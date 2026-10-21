//! Counterfactual diagnostic only. Never calls a GPU API or controls a barrier.
//! Unchanged known configuration preserves bindings; changed/unknown values invalidate them.
use std::collections::BTreeMap;
#[derive(Default)]
struct Entry {
    binding: Option<(u32, u32)>,
    pending: bool,
    crossed: bool,
    flags: Option<u32>,
    format: Option<(u32, i32)>,
}
#[derive(Default)]
pub struct Shadow {
    entries: BTreeMap<u32, Entry>,
}
impl Shadow {
    pub fn boundary(&mut self, candidate: bool) -> usize {
        if candidate {
            for e in self.entries.values_mut() {
                if e.pending {
                    e.crossed = true;
                }
            }
            0
        } else {
            let count = self
                .entries
                .values()
                .filter(|e| e.pending && e.crossed)
                .count();
            for e in self.entries.values_mut() {
                if e.pending {
                    e.binding = None;
                    e.pending = false;
                    e.crossed = false;
                }
            }
            count
        }
    }
    pub fn reset(&mut self) -> usize {
        let count = self.boundary(false);
        self.entries.clear();
        count
    }
    fn prepare(&mut self, handle: u32) -> bool {
        if !self.entries.contains_key(&handle) && self.entries.len() >= 64 {
            self.entries.clear();
            return false;
        }
        self.entries.entry(handle).or_default();
        true
    }
    fn invalidate_binding(e: &mut Entry) -> Option<&'static str> {
        let event = (e.pending && e.crossed).then_some("shadow/configuration_end");
        e.binding = None;
        e.pending = false;
        e.crossed = false;
        event
    }
    // Observed before dispatch; any failed request resets all state before the next request.
    pub fn flags(&mut self, handle: u32, flags: u32) -> Option<&'static str> {
        if !self.prepare(handle) {
            return Some("shadow/capacity_reset");
        }
        let e = self.entries.get_mut(&handle).unwrap();
        let event = if e.flags != Some(flags) {
            Self::invalidate_binding(e)
        } else {
            None
        };
        e.flags = Some(flags);
        event
    }
    pub fn format(&mut self, handle: u32, format: u32, channels: i32) -> Option<&'static str> {
        if !self.prepare(handle) {
            return Some("shadow/capacity_reset");
        }
        let e = self.entries.get_mut(&handle).unwrap();
        let event = if e.format != Some((format, channels)) {
            Self::invalidate_binding(e)
        } else {
            None
        };
        e.format = Some((format, channels));
        event
    }
    pub fn pending(&self) -> usize {
        self.entries.values().filter(|e| e.pending).count()
    }
    // Only successful address requests enter the model. Errors reset the model.
    pub fn address(&mut self, handle: u32, pointer: u32, bytes: u32) -> Option<&'static str> {
        if !self.entries.contains_key(&handle) && self.entries.len() >= 64 {
            self.entries.clear();
            return Some("shadow/capacity_reset");
        }
        let e = self.entries.entry(handle).or_default();
        if pointer == 0 {
            e.pending = e.binding.is_some();
            return None;
        }
        let event = if e.pending && e.crossed {
            Some(if e.binding == Some((pointer, bytes)) {
                "shadow/same_binding_after_candidate"
            } else {
                "shadow/changed_binding_after_candidate"
            })
        } else {
            None
        };
        e.binding = Some((pointer, bytes));
        e.pending = false;
        e.crossed = false;
        event
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn texture_kernel_keeps_active_binding_until_later_unbind() {
        let mut s = Shadow::default();
        s.address(1, 100, 32);
        assert_eq!(s.boundary(false), 0); // Texture kernel uses active binding.
        s.address(1, 0, 0);
        s.boundary(true);
        assert_eq!(
            s.address(1, 100, 32),
            Some("shadow/same_binding_after_candidate")
        );
        s.reset(); // Lifetime/error invalidates active bindings too.
        s.address(1, 0, 0);
        s.boundary(true);
        assert_eq!(s.address(1, 100, 32), None);
    }
    #[test]
    fn same_changed_and_repeated_unbind() {
        let mut s = Shadow::default();
        s.address(1, 100, 32);
        s.address(1, 0, 0);
        s.boundary(true);
        s.address(1, 0, 0);
        assert_eq!(
            s.address(1, 100, 32),
            Some("shadow/same_binding_after_candidate")
        );
        s.address(1, 0, 0);
        s.boundary(true);
        assert_eq!(
            s.address(1, 100, 64),
            Some("shadow/changed_binding_after_candidate")
        );
    }
    #[test]
    fn real_boundary_and_configuration_discard_candidates() {
        let mut s = Shadow::default();
        s.address(1, 100, 32);
        s.address(1, 0, 0);
        s.boundary(true);
        assert_eq!(s.boundary(false), 1);
        assert_eq!(s.address(1, 100, 32), None);
        s.address(1, 0, 0);
        s.boundary(true);
        assert_eq!(s.flags(1, 7), Some("shadow/configuration_end"));
        assert_eq!(s.address(1, 100, 32), None);
    }
    #[test]
    fn no_candidate_no_hit_and_bounded_capacity() {
        let mut s = Shadow::default();
        s.address(1, 100, 32);
        s.address(1, 0, 0);
        assert_eq!(s.address(1, 100, 32), None);
        for i in 2..=64 {
            s.address(i, 100, 32);
        }
        assert_eq!(s.address(65, 100, 32), Some("shadow/capacity_reset"));
        assert!(s.entries.is_empty());
    }
}

#[cfg(test)]
mod sequence_tests {
    use super::*;
    // Compare against the existing reuse state machine with a CPU-only backend.
    // This validates bookkeeping, not HIP semantics or GPU safety.
    use super::reference_reuse as reuse;
    struct Backend;
    impl reuse::Backend for Backend {
        type Error = ();
        fn synchronize(&mut self) -> Result<(), ()> {
            Ok(())
        }
        fn address(&mut self, _: u32, _: u32, _: u32) -> Result<u32, ()> {
            Ok(0)
        }
        fn flags(&mut self, _: u32, _: u32) -> Result<(), ()> {
            Ok(())
        }
        fn format(&mut self, _: u32, _: u32, _: i32) -> Result<(), ()> {
            Ok(())
        }
    }
    #[test]
    fn full_cycle_preserves_identical_configuration_in_both_orders() {
        for reverse in [false, true] {
            let mut s = Shadow::default();
            for cycle in 0..100 {
                if reverse {
                    s.format(1, 32, 4);
                    s.flags(1, 2);
                } else {
                    s.flags(1, 2);
                    s.format(1, 32, 4);
                }
                // PhysX may issue an extra unbind immediately before binding.
                s.address(1, 0, 0);
                assert_eq!(
                    s.address(1, 100, 64),
                    if cycle == 0 {
                        None
                    } else {
                        Some("shadow/same_binding_after_candidate")
                    }
                );
                s.boundary(false); // Texture kernel must preserve the ACTIVE binding.
                s.flags(1, 2);
                s.format(1, 32, 4);
                s.address(1, 0, 0);
                assert_eq!(s.pending(), 1);
                s.boundary(true);
                s.boundary(true); // More than one candidate does not create extra reuse.
            }
        }
    }
    #[test]
    fn changed_config_and_failure_reset_cannot_produce_hits() {
        for mode in 0..4 {
            let mut s = Shadow::default();
            s.flags(1, 2);
            s.format(1, 32, 4);
            s.address(1, 100, 64);
            s.address(1, 0, 0);
            s.boundary(true);
            match mode {
                0 => assert_eq!(s.flags(1, 3), Some("shadow/configuration_end")),
                1 => assert_eq!(s.format(1, 33, 4), Some("shadow/configuration_end")),
                2 => assert_eq!(s.format(1, 32, 2), Some("shadow/configuration_end")),
                _ => {
                    s.flags(1, 2);
                    s.reset();
                } // Failed setter, or lifetime boundary.
            }
            assert_eq!(s.address(1, 100, 64), None);
        }
    }
    #[test]
    fn generated_sequences_match_reuse_state_machine() {
        for seed in 1..=256u64 {
            let mut rng = seed;
            let mut s = Shadow::default();
            let mut reference = reuse::TextureReuse::default();
            let mut backend = Backend;
            for step in 0..256 {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let handle = ((rng >> 32) % 3) as u32;
                let op = (rng >> 40) % 10;
                match op {
                    0..=2 => {
                        let p = [0, 100, 200][op as usize];
                        reference.address(&mut backend, handle, p, 64).unwrap();
                        s.address(handle, p, 64);
                    }
                    3..=4 => {
                        reference.flags(&mut backend, handle, op as u32).unwrap();
                        s.flags(handle, op as u32);
                    }
                    5..=6 => {
                        reference
                            .format(&mut backend, handle, 32, op as i32)
                            .unwrap();
                        s.format(handle, 32, op as i32);
                    }
                    7 => {
                        reference.drain(&mut backend).unwrap();
                        s.boundary(false);
                    }
                    8 => {
                        s.boundary(true);
                    }
                    _ => {
                        reference.drain(&mut backend).unwrap();
                        reference.invalidate();
                        s.reset();
                    }
                }
                assert_eq!(
                    s.pending() != 0,
                    reference.has_pending(),
                    "seed {seed}, step {step}"
                );
                for h in 0..3 {
                    for p in [100, 200] {
                        let predicted = s
                            .entries
                            .get(&h)
                            .is_some_and(|e| e.pending && e.binding == Some((p, 64)));
                        assert_eq!(
                            predicted,
                            reference.can_reuse(h, p, 64),
                            "seed {seed}, step {step}, handle {h}"
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[path = "texture_reuse.rs"]
mod reference_reuse;
