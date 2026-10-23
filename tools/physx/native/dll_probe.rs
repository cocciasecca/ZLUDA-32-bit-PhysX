//! A Windows x86 executable that exercises the actual nvcuda.dll exports.
use std::ffi::{c_char, c_void, CString};
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}
struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0);
        }
    }
}
impl Library {
    unsafe fn function<T: Copy>(&self, name: &str) -> io::Result<T> {
        let name = CString::new(name).unwrap();
        let address = GetProcAddress(self.0, name.as_ptr());
        if address.is_null() {
            return Err(io::Error::other(format!("Missing DLL export: {name:?}")));
        }
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of_val(&address));
        Ok(std::mem::transmute_copy(&address))
    }
}
fn check(status: u32) -> io::Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(io::Error::other(format!("CUDA status {status}")))
    }
}
fn main() -> io::Result<()> {
    if std::mem::size_of::<usize>() != 4 {
        return Err(io::Error::other("Build the probe for Windows x86"));
    }
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "probe" {
        return Err(io::Error::other(
            "Usage: dll-probe.exe probe 127.0.0.1:port",
        ));
    }
    std::env::set_var("ZLUDA_NATIVE_BACKEND", "1");
    std::env::set_var("ZLUDA_NATIVE_ADDR", &args[2]);
    let path = PathBuf::from(
        std::env::var_os("ZLUDA_NATIVE_DLL")
            .ok_or_else(|| io::Error::other("Set ZLUDA_NATIVE_DLL to nvcuda.dll"))?,
    )
    .canonicalize()?;
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let library = Library(LoadLibraryW(name.as_ptr()));
        if library.0.is_null() {
            std::mem::forget(library);
            return Err(io::Error::last_os_error());
        }
        let init = library.function::<unsafe extern "system" fn(u32) -> u32>("cuInit")?;
        let create = library
            .function::<unsafe extern "system" fn(*mut usize, u32, i32) -> u32>("cuCtxCreate_v2")?;
        let destroy =
            library.function::<unsafe extern "system" fn(usize) -> u32>("cuCtxDestroy_v2")?;
        let alloc = library
            .function::<unsafe extern "system" fn(*mut usize, usize) -> u32>("cuMemAlloc_v2")?;
        let free = library.function::<unsafe extern "system" fn(usize) -> u32>("cuMemFree_v2")?;
        let upload = library
            .function::<unsafe extern "system" fn(usize, *const u32, usize) -> u32>(
                "cuMemcpyHtoD_v2",
            )?;
        let download = library
            .function::<unsafe extern "system" fn(*mut u32, usize, usize, usize) -> u32>(
                "cuMemcpyDtoHAsync_v2",
            )?;
        check(init(0))?;
        let mut ctx = 0;
        check(create(&mut ctx, 0, 0))?;
        let result = (|| {
            let mut pointer = 0;
            check(alloc(&mut pointer, 4))?;
            let copies = (|| {
                for value in [42u32, 100] {
                    check(upload(pointer, &value, 4))?;
                    let mut output = 0;
                    check(download(&mut output, pointer, 4, 0))?;
                    if output != value {
                        return Err(io::Error::other("DLL copy roundtrip mismatch"));
                    }
                    println!("Windows DLL -> native server -> DLL: {output}");
                }
                Ok(())
            })();
            let kernel = if copies.is_ok() && std::env::var("ZLUDA_PROBE_GPU").as_deref() == Ok("1")
            {
                gpu_probe(&library, pointer)
            } else {
                Ok(())
            };
            let release = check(free(pointer));
            copies.and(kernel).and(release)
        })();
        let teardown = check(destroy(ctx));
        result.and(teardown)
    }
}

unsafe fn gpu_probe(library: &Library, pointer: usize) -> io::Result<()> {
    let load = library
        .function::<unsafe extern "system" fn(*mut usize, *const u8) -> u32>("cuModuleLoadData")?;
    let unload = library.function::<unsafe extern "system" fn(usize) -> u32>("cuModuleUnload")?;
    let get = library.function::<unsafe extern "system" fn(*mut usize, usize, *const u8) -> u32>(
        "cuModuleGetFunction",
    )?;
    let launch = library.function::<unsafe extern "system" fn(
        usize,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        usize,
        *mut *mut c_void,
        *mut *mut c_void,
    ) -> u32>("cuLaunchKernel")?;
    let sync = library.function::<unsafe extern "system" fn() -> u32>("cuCtxSynchronize")?;
    let download = library
        .function::<unsafe extern "system" fn(*mut u32, usize, usize, usize) -> u32>(
            "cuMemcpyDtoHAsync_v2",
        )?;
    let ptx = b".version 6.0\n.target sm_30\n.address_size 32\n.visible .entry increment(.param .u32 data) {\n.reg .b32 %r<4>;\nld.param.u32 %r0, [data];\nld.global.u32 %r1, [%r0];\nadd.u32 %r2, %r1, 1;\nst.global.u32 [%r0], %r2;\nret;\n}\n\0";
    let mut module = 0;
    check(load(&mut module, ptx.as_ptr()))?;
    let result = (|| {
        let mut function = 0;
        check(get(&mut function, module, b"increment\0".as_ptr()))?;
        let mut argument = pointer as u32;
        let mut size = 4usize;
        let mut extra = [
            1usize as *mut c_void,
            (&mut argument as *mut u32).cast(),
            2usize as *mut c_void,
            (&mut size as *mut usize).cast(),
            std::ptr::null_mut(),
        ];
        check(launch(
            function,
            1,
            1,
            1,
            1,
            1,
            1,
            0,
            0,
            std::ptr::null_mut(),
            extra.as_mut_ptr(),
        ))?;
        check(sync())?;
        let mut output = 0;
        check(download(&mut output, pointer, 4, 0))?;
        if output != 101 {
            return Err(io::Error::other(format!(
                "PTX32 kernel result {output}, expected 101"
            )));
        }
        println!("Actual DLL -> Linux ZLUDA -> PTX32 kernel verified: 100 -> 101");
        Ok(())
    })();
    let release = check(unload(module));
    result.and(release)
}
