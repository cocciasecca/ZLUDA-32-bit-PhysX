//! File-backed shared RAM. The socket transfers ownership of the single slot.
use std::{fs::{File, OpenOptions}, io, path::{Path, PathBuf}, ptr::NonNull};
pub const CAPACITY: usize = 8 * 1024 * 1024;
pub struct Mapping {
    address: NonNull<u8>,
    _file: File,
    owned_path: Option<PathBuf>,
}
// Access is serialized by Connection; no references escape a transport operation.
unsafe impl Send for Mapping {}
impl Mapping {
    pub fn create(directory: &Path) -> io::Result<(Self, String)> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = format!("cuda-{}-{}.shm", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
        let path = directory.join(&name);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)] {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)?;
        let result = (|| { file.set_len(CAPACITY as u64)?; Self::map(file, Some(path.clone())) })();
        if result.is_err() { let _ = std::fs::remove_file(path); }
        result.map(|mapping| (mapping, name))
    }
    pub fn open(directory: &Path, name: &str) -> io::Result<Self> {
        if !name.starts_with("cuda-") || !name.ends_with(".shm") || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.') {
            return Err(io::Error::other("Invalid shared-memory name"));
        }
        let file = OpenOptions::new().read(true).write(true).open(directory.join(name))?;
        if file.metadata()?.len() != CAPACITY as u64 { return Err(io::Error::other("Invalid shared-memory size")); }
        Self::map(file, None)
    }
    fn map(file: File, owned_path: Option<PathBuf>) -> io::Result<Self> {
        #[cfg(unix)]
        let address = unsafe {
            use std::os::fd::AsRawFd;
            unsafe extern "C" { fn mmap(p: *mut std::ffi::c_void, n: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut std::ffi::c_void; }
            let p = mmap(std::ptr::null_mut(), CAPACITY, 3, 1, file.as_raw_fd(), 0);
            if p as isize == -1 { return Err(io::Error::last_os_error()); }
            p.cast::<u8>()
        };
        #[cfg(windows)]
        let address = unsafe {
            use std::os::windows::io::AsRawHandle;
            #[link(name="kernel32")]
            unsafe extern "system" {
                fn CreateFileMappingW(file: *mut std::ffi::c_void, attrs: *const std::ffi::c_void, protect: u32, hi: u32, lo: u32, name: *const u16) -> *mut std::ffi::c_void;
                fn MapViewOfFile(mapping: *mut std::ffi::c_void, access: u32, hi: u32, lo: u32, size: usize) -> *mut std::ffi::c_void;
                fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
            }
            let handle = CreateFileMappingW(file.as_raw_handle(), std::ptr::null(), 4, 0, 0, std::ptr::null());
            if handle.is_null() { return Err(io::Error::last_os_error()); }
            let p = MapViewOfFile(handle, 6, 0, 0, CAPACITY);
            let error = io::Error::last_os_error();
            CloseHandle(handle);
            if p.is_null() { return Err(error); }
            p.cast::<u8>()
        };
        Ok(Self { address: NonNull::new(address).ok_or_else(|| io::Error::other("Null mapping"))?, _file: file, owned_path })
    }
    // Caller must own the slot (received a socket notification or is replying).
    pub fn write(&mut self, header: u32, sequence: u64, body: &[u8]) {
        assert!(body.len() <= CAPACITY - 16);
        unsafe {
            let slot = std::slice::from_raw_parts_mut(self.address.as_ptr(), CAPACITY);
            slot[..4].copy_from_slice(&header.to_le_bytes());
            slot[4..8].copy_from_slice(&(body.len() as u32).to_le_bytes());
            slot[8..16].copy_from_slice(&sequence.to_le_bytes());
            slot[16..16+body.len()].copy_from_slice(body);
        }
        std::sync::atomic::fence(std::sync::atomic::Ordering::Release);
    }
    pub fn read(&self) -> io::Result<(u32, u64, Vec<u8>)> {
        std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        unsafe {
            let slot = std::slice::from_raw_parts(self.address.as_ptr(), CAPACITY);
            let length = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
            if length > CAPACITY - 16 { return Err(io::Error::other("Shared-memory frame exceeds slot")); }
            Ok((u32::from_le_bytes(slot[..4].try_into().unwrap()), u64::from_le_bytes(slot[8..16].try_into().unwrap()), slot[16..16+length].to_vec()))
        }
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            #[cfg(unix)] {
                unsafe extern "C" { fn munmap(p: *mut std::ffi::c_void, n: usize) -> i32; }
                munmap(self.address.as_ptr().cast(), CAPACITY);
            }
            #[cfg(windows)] {
                #[link(name="kernel32")]
                unsafe extern "system" { fn UnmapViewOfFile(p: *const std::ffi::c_void) -> i32; }
                UnmapViewOfFile(self.address.as_ptr().cast());
            }
        }
        // Windows may still hold the mapping; session-directory cleanup is a second guard.
        if let Some(path) = &self.owned_path { let _ = std::fs::remove_file(path); }
    }
}
