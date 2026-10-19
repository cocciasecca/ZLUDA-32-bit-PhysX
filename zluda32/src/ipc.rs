use cuda_types::cuda::CUerror;
use rkyv::rancor::{Failure, Strategy};
use rkyv::{Archive, Deserialize, Portable, Serialize};
use std::env;
use std::num::NonZeroU32;
use std::os::windows::io::{AsHandle, AsRawHandle};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use windows::core::Error;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use zluda_server_common::{Opcode, Serializer};

pub(crate) struct Server {
    local: zluda_server_common::Endpoint,
    remote: zluda_server_common::Endpoint,
    _child: Child,
    arena: stumpalo::Arena,
    range_cache: crate::range_cache::RangeCache,
    range_cache_enabled: bool,
    range_queries: u64,
    range_hits: u64,
    range_log: Option<std::fs::File>,
}

impl Server {
    pub unsafe fn start() -> Result<Self, Error> {
        let local = zluda_server_common::Endpoint::new()?;
        let remote = zluda_server_common::Endpoint::new()?;
        let spawn_server = |path: &PathBuf| {
            Command::new(path)
                .args([
                    &local.event_name,
                    &local.shared_memory.name,
                    &remote.event_name,
                    &remote.shared_memory.name,
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .current_dir(path.parent().unwrap())
                .spawn()
        };
        let mut primary_path = zluda_common::os::self_path().ok_or(Error::new(
            E_FAIL,
            "Could not get path to the executing module",
        ))?;
        primary_path.pop();
        if cfg!(debug_assertions) {
            primary_path.push("../../debug/zluda64_server.exe");
        } else {
            primary_path.push("../zluda64_server.exe");
        };
        let fallback_path = env::var("ZLUDA64_PATH").ok().map(PathBuf::from);
        let child = match (spawn_server(&primary_path), fallback_path) {
            (Ok(c), _) => c,
            (Err(_), Some(mut fallback_path)) => {
                fallback_path.push("zluda64_server.exe");
                spawn_server(&fallback_path)?
            }
            (Err(e), None) => return Err(e.into()),
        };
        zluda_windows::kill_child_on_process_exit(child.as_handle().as_raw_handle())?;
        read_git_version(&local)?;
        let arena = stumpalo::Arena::new();
        let range_cache_enabled = env::var("ZLUDA_PHYSX_RANGE_CACHE").as_deref() == Ok("1");
        let range_log = env::var("ZLUDA_PHYSX_DIAGNOSTICS").ok().and_then(|base| {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(format!("{base}.range-cache.pid-{}.csv", std::process::id()))
                .ok()?;
            writeln!(
                file,
                "# git={},enabled={}\nqueries,hits,misses",
                env!("VERGEN_GIT_SHA"),
                range_cache_enabled
            )
            .ok()?;
            Some(file)
        });
        Ok(Server {
            local,
            remote,
            _child: child,
            arena,
            range_cache: Default::default(),
            range_cache_enabled,
            range_queries: 0,
            range_hits: 0,
            range_log,
        })
    }

    pub(crate) fn invalidate_ranges(&mut self) {
        self.range_cache.clear();
    }

    // Unknown/future operations invalidate by default. Only operations audited
    // not to change the bridge allocation map can retain cached replies.
    fn prepare_range_cache(&mut self, opcode: &Opcode) {
        if !matches!(
            opcode,
            Opcode::cuMemGetAddressRange_v2
                | Opcode::cuTexRefSetAddress_v2
                | Opcode::cuTexRefSetFlags
                | Opcode::cuTexRefSetFormat
                | Opcode::cuMemcpyHtoD_v2
                | Opcode::cuMemcpyHtoDAsync_v2
                | Opcode::cuMemcpyDtoHAsync_v2
                | Opcode::cuMemcpyDtoD_v2
                | Opcode::cuMemcpyDtoDAsync_v2
                | Opcode::cuMemsetD8_v2
                | Opcode::cuLaunchKernel
                | Opcode::cuCtxSynchronize
                | Opcode::cuEventRecord
                | Opcode::cuEventQuery
                | Opcode::cuEventSynchronize
                | Opcode::cuStreamQuery
        ) {
            self.invalidate_ranges();
        }
    }

    pub(crate) fn address_range(&mut self, pointer: u32) -> Result<(u32, u32), CUerror> {
        self.range_queries += 1;
        let cached = self
            .range_cache_enabled
            .then(|| self.range_cache.get(pointer))
            .flatten();
        let result = if let Some(reply) = cached {
            self.range_hits += 1;
            Ok(reply)
        } else {
            self.remote_call_zero_copy::<zluda_server_common::cuMemGetAddressRange_v2Out>(
                Opcode::cuMemGetAddressRange_v2,
                zluda_server_common::cuMemGetAddressRange_v2In {
                    dptr: pointer.into(),
                },
            )
            .map(|r| (r.pbase.to_native(), r.psize.to_native()))
        };
        match result {
            Ok((base, size)) if self.range_cache_enabled && cached.is_none() => {
                self.range_cache.insert(pointer, base, size)
            }
            Err(_) => self.invalidate_ranges(),
            _ => (),
        }
        // Bounded aggregate output, no per-request disk I/O. Final partial batch
        // is intentionally not flushed on process termination.
        if self.range_queries % 4096 == 0 {
            use std::io::Write;
            if let Some(file) = self.range_log.as_mut() {
                if writeln!(
                    file,
                    "{},{},{}",
                    self.range_queries,
                    self.range_hits,
                    self.range_queries - self.range_hits
                )
                .is_err()
                {
                    self.range_log = None;
                }
            }
        }
        result
    }

    pub(crate) fn remote_call_zero_copy<Out: Portable + Clone>(
        &mut self,
        opcode: Opcode,
        data: impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Out, CUerror> {
        self.prepare_range_cache(&opcode);
        self.remote.shared_memory.write_header(opcode as u32);
        self.remote.shared_memory.write_body(&data);
        unsafe { SignalObjectAndWait(*self.remote.event, *self.local.event, INFINITE, false) };
        let return_value = self.local.shared_memory.read_header();
        match NonZeroU32::new(return_value) {
            None => Ok(()),
            Some(code) => {
                self.invalidate_ranges();
                Err(CUerror(code))
            }
        }?;
        Ok(self.local.shared_memory.read_body())
    }

    pub(crate) fn remote_call_framed_in<Out: Portable + Clone>(
        &mut self,
        opcode: Opcode,
        data: impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Out, CUerror> {
        self.prepare_range_cache(&opcode);
        let opcode = opcode as u32;
        self.remote.shared_memory.write_header(opcode);
        let old_shmem = self
            .remote
            .shared_memory
            .serialize_body(&mut self.arena, &data)?;
        if let Some(mut old_shmem) = old_shmem {
            self.remote.shared_memory.write_header(opcode);
            old_shmem.write_header(u32::MAX);
            old_shmem.write_buffer(self.remote.shared_memory.name.as_bytes());
        }
        unsafe { SignalObjectAndWait(*self.remote.event, *self.local.event, INFINITE, false) };
        let return_value = self.local.shared_memory.read_header();
        match NonZeroU32::new(return_value) {
            None => Ok(()),
            Some(code) => {
                self.invalidate_ranges();
                Err(CUerror(code))
            }
        }?;
        Ok(self.local.shared_memory.read_body())
    }

    pub(crate) fn remote_call_framed_out<Out: Archive>(
        &mut self,
        opcode: Opcode,
        data: impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Out, CUerror>
    where
        <Out as Archive>::Archived: Deserialize<Out, Strategy<(), Failure>>,
    {
        self.prepare_range_cache(&opcode);
        self.remote.shared_memory.write_header(opcode as u32);
        self.remote.shared_memory.write_body(&data);
        unsafe { SignalObjectAndWait(*self.remote.event, *self.local.event, INFINITE, false) };
        let mut return_value = self.local.shared_memory.read_header();
        if return_value == u32::MAX {
            let new_shmem_name = unsafe {
                String::from_utf8_unchecked(self.local.shared_memory.read_buffer().to_vec())
            };
            let new_shmem =
                unsafe { zluda_server_common::SharedMemory::open(new_shmem_name, None) }
                    .map_err(|_| CUerror::MAP_FAILED)?;
            self.local.shared_memory = new_shmem;
            return_value = self.local.shared_memory.read_header();
        }
        match NonZeroU32::new(return_value) {
            None => Ok(()),
            Some(code) => {
                self.invalidate_ranges();
                Err(CUerror(code))
            }
        }?;
        self.local.shared_memory.deserialize_body()
    }
}

fn read_git_version(local: &zluda_server_common::Endpoint) -> Result<(), Error> {
    unsafe { WaitForSingleObject(*local.event, INFINITE) };
    let opcode = local.shared_memory.read_header();
    if opcode != Opcode::Startup as u32 {
        return Err(Error::empty());
    }
    let git_sha = env!("VERGEN_GIT_SHA");
    let git_version = local.shared_memory.read_buffer();
    if git_version != git_sha.as_bytes() {
        return Err(Error::new(
            E_FAIL,
            "Git version mismatch between zluda and zluda64_server",
        ));
    }
    Ok(())
}
