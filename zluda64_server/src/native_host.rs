//! Linux process supervisor and dynamic ZLUDA loading. Each client owns a process.
use crate::*;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use zluda_server_common::native_rpc::Connection;
pub fn entry(run: fn(Endpoint, Endpoint) -> io::Result<()>) -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--connection-fd" {
        let fd: i32 = args[2].parse().map_err(io::Error::other)?;
        if fd < 3 {
            return Err(io::Error::other("Invalid inherited socket"));
        }
        let mut connection = Connection::new(unsafe { TcpStream::from_raw_fd(fd) })?;
        let token = std::env::var("ZLUDA_NATIVE_TOKEN").map_err(io::Error::other)?;
        connection.authenticate_server(&token, env!("VERGEN_GIT_SHA"))?;
        let local = Endpoint {
            shared_memory: SharedMemory::new(),
            connection: None,
            disconnected: false,
        };
        let remote = Endpoint {
            shared_memory: SharedMemory::new(),
            connection: Some(connection),
            disconnected: false,
        };
        return run(local, remote);
    }
    if args.len() != 3 || args[1] != "--listen" {
        return Err(io::Error::other(
            "Usage: zluda64_server --listen 127.0.0.1:0",
        ));
    }
    let address: SocketAddr = args[2].parse().map_err(io::Error::other)?;
    if !address.ip().is_loopback() {
        return Err(io::Error::other("Native server only listens on loopback"));
    }
    let token = std::env::var("ZLUDA_NATIVE_TOKEN").map_err(io::Error::other)?;
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::other("Invalid session token"));
    }
    let listener = TcpListener::bind(address)?;
    println!("{}", listener.local_addr()?);
    io::stdout().flush()?;
    let mut children: Vec<std::process::Child> = Vec::new();
    for stream in listener.incoming() {
        let stream = stream?;
        children.retain_mut(|child| matches!(child.try_wait(), Ok(None)));
        if children.len() >= 32 {
            drop(stream);
            continue;
        }
        let fd = stream.as_raw_fd();
        let parent = unsafe { libc::getpid() };
        let mut command = Command::new(std::env::current_exe()?);
        command
            .args(["--connection-fd", &fd.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null());
        unsafe {
            command.pre_exec(move || {
                if libc::fcntl(fd, libc::F_SETFD, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) == -1 {
                    return Err(io::Error::last_os_error());
                }
                if libc::getppid() != parent {
                    return Err(io::Error::other("Supervisor exited"));
                }
                Ok(())
            });
        }
        children.push(command.spawn()?);
    }
    Ok(())
}
pub fn receive(local: &mut Endpoint, remote: &mut Endpoint) {
    let result = remote
        .connection
        .as_mut()
        .unwrap()
        .receive_request()
        .and_then(|frame| {
            local
                .shared_memory
                .replace_message(frame.header, &frame.body)
                .map_err(|e| io::Error::other(format!("CUDA buffer error {e:?}")))
        });
    if let Err(error) = result {
        if error.kind() != io::ErrorKind::UnexpectedEof {
            eprintln!("Native CUDA connection ended: {error}");
        }
        local.disconnected = true;
    }
}
pub fn reply(local: &mut Endpoint, remote: &mut Endpoint) {
    let header = remote.shared_memory.read_header();
    let body = if header == 0 {
        remote.shared_memory.read_buffer()
    } else {
        &[]
    };
    if let Err(error) = remote.connection.as_mut().unwrap().send(header, body) {
        eprintln!("Native CUDA reply failed: {error}");
        local.disconnected = true;
        return;
    }
    receive(local, remote);
}
// Library remains loaded for the lifetime of the process, including all CUDA handles.
static LIBRARY: OnceLock<Result<usize, String>> = OnceLock::new();
pub unsafe fn symbol(name: &'static [u8]) -> Option<usize> {
    let library = LIBRARY
        .get_or_init(|| {
            let path = std::env::var("ZLUDA_NATIVE_LIBRARY")
                .map_err(|_| "ZLUDA_NATIVE_LIBRARY missing".to_string())?;
            let path = std::ffi::CString::new(path).map_err(|e| e.to_string())?;
            let handle = libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL);
            if handle.is_null() {
                let error = libc::dlerror();
                let detail = if error.is_null() {
                    "dlopen failed".into()
                } else {
                    CStr::from_ptr(error).to_string_lossy().into_owned()
                };
                eprintln!("Native ZLUDA load failed: {detail}");
                Err(detail)
            } else {
                Ok(handle as usize)
            }
        })
        .as_ref()
        .ok()?;
    let address = libc::dlsym(*library as *mut c_void, name.as_ptr().cast());
    (!address.is_null()).then_some(address as usize)
}
