//! Opt-in, bounded deferral within the serialized 32-bit bridge.
//! This is not a general CUDA texture cache. Kernel/lifetime/unknown boundaries
//! always drain. An independent experimental flag permits data-transfer boundaries.
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub enum Boundary {
    TextureMetadata,
    Transfer,
    Synchronize,
    Kernel,
    Lifetime,
    Other,
}

pub fn can_defer(boundary: Boundary, retain_across_transfers: bool) -> bool {
    match boundary {
        Boundary::TextureMetadata => true,
        Boundary::Transfer | Boundary::Synchronize => retain_across_transfers,
        Boundary::Kernel | Boundary::Lifetime | Boundary::Other => false,
    }
}

const LIMIT: usize = 64;

pub trait Backend {
    type Error;
    fn synchronize(&mut self) -> Result<(), Self::Error>;
    fn address(&mut self, handle: u32, pointer: u32, bytes: u32) -> Result<u32, Self::Error>;
    fn flags(&mut self, handle: u32, flags: u32) -> Result<(), Self::Error>;
    fn format(&mut self, handle: u32, format: u32, channels: i32) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Default)]
struct Entry {
    binding: Option<(u32, u32, u32)>, // pointer, bytes, returned byte offset
    pending: bool,
    flags: Option<u32>,
    format: Option<(u32, i32)>,
}

#[derive(Default)]
pub struct TextureReuse {
    entries: BTreeMap<u32, Entry>,
}

impl TextureReuse {
    pub fn can_reuse(&self, handle: u32, pointer: u32, bytes: u32) -> bool {
        pointer != 0
            && self.entries.get(&handle).is_some_and(|e| {
                e.pending
                    && e.binding
                        .is_some_and(|(p, b, _)| (p, b) == (pointer, bytes))
            })
    }

    pub fn has_pending(&self) -> bool {
        self.entries.values().any(|entry| entry.pending)
    }

    fn prepare<B: Backend>(&mut self, backend: &mut B, handle: u32) -> Result<(), B::Error> {
        if !self.entries.contains_key(&handle) && self.entries.len() >= LIMIT {
            self.drain(backend)?;
            self.entries.clear();
        }
        self.entries.entry(handle).or_default();
        Ok(())
    }

    fn drain_one<B: Backend>(&mut self, backend: &mut B, handle: u32) -> Result<(), B::Error> {
        if let Some(entry) = self.entries.get_mut(&handle) {
            if entry.pending {
                // Never reuse state after a failed native operation.
                entry.pending = false;
                entry.binding = None;
                backend.address(handle, 0, 0)?;
            }
        }
        Ok(())
    }

    pub fn drain<B: Backend>(&mut self, backend: &mut B) -> Result<(), B::Error> {
        let pending: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(&id, entry)| entry.pending.then_some(id))
            .collect();
        for id in pending {
            self.drain_one(backend, id)?;
        }
        Ok(())
    }

    // Caller must drain successfully BEFORE invalidating or releasing resources.
    pub fn invalidate(&mut self) {
        assert!(!self.has_pending());
        self.entries.clear();
    }

    pub fn address<B: Backend>(
        &mut self,
        backend: &mut B,
        handle: u32,
        pointer: u32,
        bytes: u32,
    ) -> Result<u32, B::Error> {
        self.prepare(backend, handle)?;
        let old = self.entries[&handle];
        if pointer == 0 && old.binding.is_some() {
            // HIP's current texture free waits on streams. Retain an explicit
            // context-wide barrier; never trade away completion for performance.
            backend.synchronize()?;
            self.entries.get_mut(&handle).unwrap().pending = true;
            return Ok(0);
        }
        if pointer != 0 && old.pending {
            if let Some((old_pointer, old_bytes, offset)) = old.binding {
                if (pointer, bytes) == (old_pointer, old_bytes) {
                    // Preserve ordering of the synchronous native rebind too.
                    backend.synchronize()?;
                    self.entries.get_mut(&handle).unwrap().pending = false;
                    return Ok(offset);
                }
            }
        }
        self.drain_one(backend, handle)?;
        self.entries.get_mut(&handle).unwrap().binding = None;
        let offset = backend.address(handle, pointer, bytes)?;
        if pointer != 0 {
            self.entries.get_mut(&handle).unwrap().binding = Some((pointer, bytes, offset));
        }
        Ok(offset)
    }

    pub fn flags<B: Backend>(
        &mut self,
        backend: &mut B,
        handle: u32,
        flags: u32,
    ) -> Result<(), B::Error> {
        self.prepare(backend, handle)?;
        if self.entries[&handle].flags != Some(flags) {
            self.drain_one(backend, handle)?;
            // A setter can rebuild a currently bound HIP object. Require a fresh
            // successful address call before trusting its cached byte offset.
            self.entries.get_mut(&handle).unwrap().binding = None;
        }
        if let Err(error) = backend.flags(handle, flags) {
            self.entries.get_mut(&handle).unwrap().binding = None;
            return Err(error);
        }
        self.entries.get_mut(&handle).unwrap().flags = Some(flags);
        Ok(())
    }

    pub fn format<B: Backend>(
        &mut self,
        backend: &mut B,
        handle: u32,
        format: u32,
        channels: i32,
    ) -> Result<(), B::Error> {
        self.prepare(backend, handle)?;
        if self.entries[&handle].format != Some((format, channels)) {
            self.drain_one(backend, handle)?;
            self.entries.get_mut(&handle).unwrap().binding = None;
        }
        if let Err(error) = backend.format(handle, format, channels) {
            self.entries.get_mut(&handle).unwrap().binding = None;
            return Err(error);
        }
        self.entries.get_mut(&handle).unwrap().format = Some((format, channels));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_policy_never_bypasses_kernel_lifetime_or_unknown() {
        for enabled in [false, true] {
            for boundary in [Boundary::Kernel, Boundary::Lifetime, Boundary::Other] {
                assert!(!can_defer(boundary, enabled));
            }
            assert!(can_defer(Boundary::TextureMetadata, enabled));
            assert_eq!(can_defer(Boundary::Transfer, enabled), enabled);
            assert_eq!(can_defer(Boundary::Synchronize, enabled), enabled);
        }
    }

    #[test]
    fn transfer_between_unbind_and_rebind_retains_resource_but_not_barriers() {
        for enabled in [false, true] {
            let mut cache = TextureReuse::default();
            let mut b = Mock::default();
            let offset = cache.address(&mut b, 1, 256, 64).unwrap();
            cache.address(&mut b, 1, 0, 0).unwrap();
            if !can_defer(Boundary::Transfer, enabled) {
                cache.drain(&mut b).unwrap();
            }
            // A transfer uses allocation addresses, not texture binding state.
            b.calls.push("transfer".into());
            let returned = cache.address(&mut b, 1, 256, 64).unwrap();
            assert_eq!(returned, offset);
            let unbinds = b
                .calls
                .iter()
                .filter(|c| c.as_str() == "address:1:0:0")
                .count();
            assert_eq!(unbinds, if enabled { 0 } else { 1 });
            assert_eq!(
                b.calls.iter().filter(|c| c.as_str() == "sync").count(),
                if enabled { 2 } else { 1 }
            );
            cache.address(&mut b, 1, 0, 0).unwrap();
            assert!(!can_defer(Boundary::Kernel, enabled));
            cache.drain(&mut b).unwrap();
            assert!(b.bindings.is_empty());
        }
    }

    #[derive(Default)]
    struct Mock {
        calls: Vec<String>,
        fail: Option<&'static str>,
        bindings: BTreeMap<u32, (u32, u32)>,
    }
    impl Mock {
        fn record(&mut self, kind: &'static str, call: String) -> Result<(), &'static str> {
            self.calls.push(call);
            if self.fail == Some(kind) {
                self.fail = None;
                Err(kind)
            } else {
                Ok(())
            }
        }
    }
    impl Backend for Mock {
        type Error = &'static str;
        fn synchronize(&mut self) -> Result<(), Self::Error> {
            self.record("sync", "sync".into())
        }
        fn address(&mut self, h: u32, p: u32, b: u32) -> Result<u32, Self::Error> {
            self.record("address", format!("address:{h}:{p}:{b}"))?;
            if p == 0 {
                self.bindings.remove(&h);
            } else {
                self.bindings.insert(h, (p, b));
            }
            Ok(if p == 0 { 0 } else { 16 })
        }
        fn flags(&mut self, h: u32, f: u32) -> Result<(), Self::Error> {
            self.record("flags", format!("flags:{h}:{f}"))
        }
        fn format(&mut self, h: u32, f: u32, c: i32) -> Result<(), Self::Error> {
            self.record("format", format!("format:{h}:{f}:{c}"))
        }
    }

    #[test]
    fn reuses_only_matching_binding_and_preserves_both_barriers_and_offset() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        cache.flags(&mut b, 1, 0).unwrap();
        cache.format(&mut b, 1, 1, 4).unwrap();
        assert_eq!(cache.address(&mut b, 1, 256, 64), Ok(16));
        b.calls.clear();
        assert_eq!(cache.address(&mut b, 1, 0, 0), Ok(0));
        cache.flags(&mut b, 1, 0).unwrap();
        cache.format(&mut b, 1, 1, 4).unwrap();
        assert_eq!(cache.address(&mut b, 1, 256, 64), Ok(16));
        assert_eq!(b.calls, ["sync", "flags:1:0", "format:1:1:4", "sync"]);
        assert!(!cache.has_pending());
        // A direct repeated bind (no unbind) must still execute the native call.
        cache.address(&mut b, 1, 256, 64).unwrap();
        assert_eq!(b.calls.last().unwrap(), "address:1:256:64");
    }

    #[test]
    fn changed_address_size_or_configuration_drains_before_native_operation() {
        for mode in 0..4 {
            let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
            cache.flags(&mut b, 1, 0).unwrap();
            cache.format(&mut b, 1, 1, 4).unwrap();
            cache.address(&mut b, 1, 256, 64).unwrap();
            cache.address(&mut b, 1, 0, 0).unwrap();
            b.calls.clear();
            match mode {
                0 => {
                    cache.address(&mut b, 1, 512, 64).unwrap();
                }
                1 => {
                    cache.address(&mut b, 1, 256, 128).unwrap();
                }
                2 => {
                    cache.flags(&mut b, 1, 1).unwrap();
                }
                _ => {
                    cache.format(&mut b, 1, 1, 2).unwrap();
                }
            }
            assert_eq!(b.calls.len(), 2);
            assert_eq!(b.calls[0], "address:1:0:0");
        }
    }

    #[test]
    fn drain_before_launch_or_resource_free_restores_native_unbound_state() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        for id in [1, 2] {
            cache.address(&mut b, id, 256, 64).unwrap();
            cache.address(&mut b, id, 0, 0).unwrap();
        }
        b.calls.clear();
        cache.drain(&mut b).unwrap();
        cache.invalidate();
        assert_eq!(b.calls, ["address:1:0:0", "address:2:0:0"]);
        cache.address(&mut b, 1, 256, 64).unwrap();
        assert_eq!(b.calls.last().unwrap(), "address:1:256:64");
    }

    #[test]
    fn synchronization_failure_does_not_report_success_or_cancel_unbind() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        cache.address(&mut b, 1, 256, 64).unwrap();
        b.fail = Some("sync");
        assert_eq!(cache.address(&mut b, 1, 0, 0), Err("sync"));
        assert!(!cache.has_pending());
        cache.address(&mut b, 1, 0, 0).unwrap();
        b.fail = Some("sync");
        assert_eq!(cache.address(&mut b, 1, 256, 64), Err("sync"));
        assert!(cache.has_pending());
    }

    #[test]
    fn failed_native_operations_never_leave_a_reusable_binding() {
        for kind in ["address", "flags", "format"] {
            let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
            cache.flags(&mut b, 1, 0).unwrap();
            cache.format(&mut b, 1, 1, 4).unwrap();
            cache.address(&mut b, 1, 256, 64).unwrap();
            cache.address(&mut b, 1, 0, 0).unwrap();
            b.fail = Some(kind);
            let failed = match kind {
                "address" => cache.address(&mut b, 1, 512, 64).map(|_| ()),
                "flags" => cache.flags(&mut b, 1, 0),
                _ => cache.format(&mut b, 1, 1, 4),
            };
            assert_eq!(failed, Err(kind));
            b.calls.clear();
            cache.address(&mut b, 1, 256, 64).unwrap();
            assert_eq!(b.calls.last().unwrap(), "address:1:256:64");
        }
    }

    #[test]
    fn capacity_overflow_drains_resources_before_forgetting_handles() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        for id in 0..LIMIT as u32 {
            cache.address(&mut b, id, 256, 64).unwrap();
            cache.address(&mut b, id, 0, 0).unwrap();
        }
        b.calls.clear();
        cache.address(&mut b, LIMIT as u32, 256, 64).unwrap();
        assert_eq!(b.calls.len(), LIMIT + 1);
        assert_eq!(cache.entries.len(), 1);
        assert!(!cache.has_pending());
    }
    #[test]
    fn unbound_other_texture_is_drained_before_a_launch_after_one_cache_hit() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        for id in [1, 2] {
            cache.address(&mut b, id, 256, 64).unwrap();
            cache.address(&mut b, id, 0, 0).unwrap();
        }
        cache.address(&mut b, 1, 256, 64).unwrap();
        b.calls.clear();
        cache.drain(&mut b).unwrap();
        assert_eq!(b.calls, ["address:2:0:0"]);
        assert!(!cache.has_pending());
    }

    #[test]
    fn unknown_configuration_cannot_reuse_a_deferred_binding() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        cache.address(&mut b, 1, 256, 64).unwrap();
        cache.address(&mut b, 1, 0, 0).unwrap();
        b.calls.clear();
        cache.flags(&mut b, 1, 0).unwrap();
        cache.address(&mut b, 1, 256, 64).unwrap();
        assert_eq!(b.calls, ["address:1:0:0", "flags:1:0", "address:1:256:64"]);
    }

    #[test]
    fn repeated_unbind_retains_barrier_and_failed_drain_blocks_progress() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        for id in [1, 2] {
            cache.address(&mut b, id, 256, 64).unwrap();
            cache.address(&mut b, id, 0, 0).unwrap();
        }
        b.calls.clear();
        cache.address(&mut b, 1, 0, 0).unwrap();
        assert_eq!(b.calls, ["sync"]);
        b.fail = Some("address");
        assert_eq!(cache.drain(&mut b), Err("address"));
        assert!(cache.has_pending());
        b.calls.clear();
        cache.drain(&mut b).unwrap();
        assert_eq!(b.calls, ["address:2:0:0"]);
        cache.address(&mut b, 1, 256, 64).unwrap();
        assert_eq!(b.calls.last().unwrap(), "address:1:256:64");
    }
    #[test]
    fn changing_configuration_while_bound_invalidates_cached_offset() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        cache.flags(&mut b, 1, 0).unwrap();
        cache.address(&mut b, 1, 256, 64).unwrap();
        cache.flags(&mut b, 1, 1).unwrap();
        b.calls.clear();
        cache.address(&mut b, 1, 0, 0).unwrap();
        cache.address(&mut b, 1, 256, 64).unwrap();
        assert_eq!(b.calls, ["address:1:0:0", "address:1:256:64"]);
    }
    #[test]
    fn observable_bindings_match_uncached_model_at_every_external_boundary() {
        let (mut cache, mut b) = (TextureReuse::default(), Mock::default());
        let mut expected = BTreeMap::new();
        let mut seed = 7u32;
        for _ in 0..4096 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let handle = (seed >> 8) % 8;
            match seed % 6 {
                0 | 1 => {
                    let pointer = 256 * (1 + ((seed >> 16) % 4));
                    let bytes = 64 * (1 + ((seed >> 24) % 2));
                    cache.address(&mut b, handle, pointer, bytes).unwrap();
                    expected.insert(handle, (pointer, bytes));
                }
                2 => {
                    cache.address(&mut b, handle, 0, 0).unwrap();
                    expected.remove(&handle);
                }
                3 => {
                    cache.flags(&mut b, handle, (seed >> 16) % 2).unwrap();
                }
                4 => {
                    cache
                        .format(&mut b, handle, 1, 1 + ((seed >> 16) % 4) as i32)
                        .unwrap();
                }
                _ => {
                    cache.drain(&mut b).unwrap();
                    assert_eq!(b.bindings, expected);
                    if seed & 0x10000 != 0 {
                        cache.invalidate();
                    }
                }
            }
        }
        cache.drain(&mut b).unwrap();
        assert_eq!(b.bindings, expected);
    }
}
