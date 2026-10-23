//! CPU-only integration test: actual x86 DLL -> authenticated native protocol -> wire structs.
use std::collections::BTreeMap;
use std::io;
use std::net::TcpListener;
use std::process::Command;
use zluda_server_common::*;
fn fixed<T: for<'a, 'b> rkyv::Serialize<Serializer<'a, 'b>>>(buffer: &mut message_buffer::MessageBuffer, value: T) {
    buffer.write_header(0);
    buffer.write_body(&value).unwrap();
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err(io::Error::other("Usage: native_dll_test DLL PROBE"));
    }
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?.to_string();
    let token = "1122334455667788990011223344556677889900112233445566778899001122";
    let dll = std::fs::canonicalize(&args[1])?;
    let mut child = Command::new(&args[2])
        .args(["probe", &address])
        .env("ZLUDA_NATIVE_DLL", dll)
        .env("ZLUDA_NATIVE_TOKEN", token)
        .spawn()?;
    let result = (|| {
        listener.set_nonblocking(true)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if child.try_wait()?.is_some() || std::time::Instant::now() >= deadline {
                        return Err(io::Error::other("DLL probe did not connect"));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(e) => return Err(e),
            }
        };
        stream.set_nonblocking(false)?;
        let mut connection = native_rpc::Connection::new(stream)?;
        // The test's own build script is absent; DLL revision is supplied by CI.
        let sha = std::env::var("ZLUDA_TEST_SHA").map_err(io::Error::other)?;
        connection.authenticate_server(token, &sha)?;
        connection.send(Opcode::Startup as u32, sha.as_bytes())?;
        let mut request = message_buffer::MessageBuffer::new();
        let mut response = message_buffer::MessageBuffer::new();
        let mut arena = stumpalo::Arena::new();
        let mut allocations = BTreeMap::<u32, Vec<u8>>::new();
        let mut next = 256u32;
        loop {
            let frame = match connection.receive_request() {
                Ok(f) => f,
                // Windows may reset the TCP connection when the DLL process exits.
                // Require clean allocation teardown AND a successful child below.
                Err(e) if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset) => break,
                Err(e) => return Err(e),
            };
            request
                .replace_message(frame.header, &frame.body)
                .map_err(|_| io::Error::other("buffer"))?;
            match Opcode::from_repr(frame.header) {
                Some(Opcode::cuInit) => fixed(&mut response, cuInitOut {}),
                Some(Opcode::cuMemAlloc_v2) => {
                    let input: ArchivedcuMemAlloc_v2In = request.read_body();
                    let size = input.bytesize.to_native();
                    if size > 4 * 1024 * 1024 {
                        return Err(io::Error::other("Test allocation too large"));
                    }
                    allocations.insert(next, vec![0; size as usize]);
                    fixed(&mut response, cuMemAlloc_v2Out { dptr: next.into() });
                    next += size.next_multiple_of(256);
                }
                Some(Opcode::cuMemcpyHtoD_v2) => {
                    let input = request.deserialize_body2::<cuMemcpyHtoD_v2In>();
                    let bytes = allocations
                        .get_mut(&input.dst_device.to_native())
                        .ok_or_else(|| io::Error::other("unknown allocation"))?;
                    bytes.copy_from_slice(input.src_host.as_slice());
                    fixed(&mut response, cuMemcpyHtoD_v2Out {});
                }
                Some(Opcode::cuMemcpyDtoHAsync_v2) => {
                    let input: ArchivedcuMemcpyDtoHAsync_v2In = request.read_body();
                    let bytes = allocations
                        .get(&input.src_device.to_native())
                        .ok_or_else(|| io::Error::other("unknown allocation"))?;
                    if bytes.len() != input.byte_count.to_native() as usize {
                        return Err(io::Error::other("copy size mismatch"));
                    }
                    response.write_header(0);
                    response
                        .serialize_body(
                            &mut arena,
                            &cuMemcpyDtoHAsync_v2Out {
                                dst_host: bytes.clone(),
                            },
                        )
                        .map_err(|_| io::Error::other("serialize"))?;
                }
                Some(Opcode::cuMemFree_v2) => {
                    let input: ArchivedcuMemFree_v2In = request.read_body();
                    allocations
                        .remove(&input.dptr.to_native())
                        .ok_or_else(|| io::Error::other("unknown free"))?;
                    fixed(&mut response, cuMemFree_v2Out {});
                }
                _ => {
                    return Err(io::Error::other(format!(
                        "Unexpected test opcode {}",
                        frame.header
                    )))
                }
            }
            connection.send(response.read_header(), response.read_buffer())?;
        }
        if !allocations.is_empty() {
            return Err(io::Error::other("Leaked test allocations"));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait()?;
    result?;
    if !status.success() {
        return Err(io::Error::other("DLL probe failed"));
    }
    println!("Actual x86 DLL and complete CUDA wire format verified.");
    Ok(())
}
