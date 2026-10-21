//! Bounded, read-only ABI feasibility probe. Never authorizes writes or pooling.
#[cfg(windows)]
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::Write,
};
const EXPECTED_SIZE: usize = 88;
const OBJECT_OFFSET: usize = 72;
// Matches the 64-bit textureReference layout emitted in ptx/src/pass/llvm/emit.rs.
// A match is evidence for this installation only, not a portable ABI guarantee.
fn object_matches(host: &[u8], device: &[u8]) -> bool {
    host.len() == EXPECTED_SIZE
        && device.len() == EXPECTED_SIZE
        && host[OBJECT_OFFSET..OBJECT_OFFSET + 8] != [0; 8]
        && host[OBJECT_OFFSET..OBJECT_OFFSET + 8] == device[OBJECT_OFFSET..OBJECT_OFFSET + 8]
}
#[cfg(windows)]
pub struct Probe {
    file: File,
    seen: BTreeSet<u32>,
}
#[cfg(windows)]
impl Probe {
    pub fn wants(&self, handle: u32) -> bool {
        self.seen.len() < 8 && !self.seen.contains(&handle)
    }
    pub fn open() -> Option<Self> {
        if std::env::var("ZLUDA_PHYSX_TEXTURE_GLOBAL_PROBE").as_deref() != Ok("1") {
            return None;
        }
        let mut path = std::env::var_os("ZLUDA_PHYSX_DIAGNOSTICS")?;
        path.push(format!(".texture-global.pid-{}.csv", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .ok()?;
        writeln!(
            file,
            "handle,symbol_hex,status,global_bytes,host_object,device_object"
        )
        .ok()?;
        Some(Self {
            file,
            seen: BTreeSet::new(),
        })
    }
    pub fn observe(
        &mut self,
        handle: u32,
        module: super::CUmodule,
        symbol: &[u8],
        host: *const std::ffi::c_void,
    ) {
        if symbol.len() > 512 || self.seen.len() >= 8 || !self.seen.insert(handle) {
            return;
        }
        let symbol = symbol.strip_suffix(&[0]).unwrap_or(symbol);
        let symbol_hex: String = symbol.iter().map(|b| format!("{b:02x}")).collect();
        let mut bytes = 0usize;
        let mut address = super::CUdeviceptr_v2(std::ptr::null_mut());
        let mut host_bytes = [0u8; EXPECTED_SIZE];
        let mut device_bytes = [0u8; EXPECTED_SIZE];
        let status = (|| -> String {
            let Ok(name) = std::ffi::CString::new(symbol) else {
                return "invalid_symbol".into();
            };
            if let Err(e) = unsafe {
                super::cuModuleGetGlobal_v2(&mut address, &mut bytes, module, name.as_ptr())
            } {
                return format!("lookup_error_{}", e.0.get());
            }
            if bytes != EXPECTED_SIZE {
                return "layout_size_mismatch".into();
            }
            let mut read = 0usize;
            let ok = unsafe {
                ReadProcessMemory(
                    (-1isize) as *mut _,
                    host,
                    host_bytes.as_mut_ptr().cast(),
                    EXPECTED_SIZE,
                    &mut read,
                )
            };
            if ok == 0 || read != EXPECTED_SIZE {
                return "host_read_failed".into();
            }
            if let Err(e) =
                unsafe { super::cuMemcpyDtoH_v2(device_bytes.as_mut_ptr().cast(), address, bytes) }
            {
                return format!("device_read_error_{}", e.0.get());
            }
            if object_matches(&host_bytes, &device_bytes) {
                "object_matches_candidate_layout".into()
            } else {
                "object_mismatch_or_null".into()
            }
        })();
        let h = u64::from_le_bytes(
            host_bytes[OBJECT_OFFSET..OBJECT_OFFSET + 8]
                .try_into()
                .unwrap(),
        );
        let d = u64::from_le_bytes(
            device_bytes[OBJECT_OFFSET..OBJECT_OFFSET + 8]
                .try_into()
                .unwrap(),
        );
        let _ = writeln!(
            self.file,
            "{handle},{symbol_hex},{status},{bytes},{h:x},{d:x}"
        );
        let _ = self.file.flush();
    }
}
#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn ReadProcessMemory(
        process: *mut std::ffi::c_void,
        base: *const std::ffi::c_void,
        buffer: *mut std::ffi::c_void,
        size: usize,
        read: *mut usize,
    ) -> i32;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_nonzero_object_required() {
        let mut h = [0; EXPECTED_SIZE];
        let mut d = h;
        assert!(!object_matches(&h, &d));
        h[OBJECT_OFFSET] = 3;
        d[OBJECT_OFFSET] = 3;
        assert!(object_matches(&h, &d));
        d[OBJECT_OFFSET + 7] = 1;
        assert!(!object_matches(&h, &d));
    }
    #[test]
    fn size_mismatch_is_rejected() {
        assert!(!object_matches(&[1; 87], &[1; 88]));
        assert!(!object_matches(&[1; 88], &[1; 89]));
    }
}
