//! Counterfactual diagnostic only. Never calls a GPU API or controls a barrier.
//! Configuration setters conservatively terminate candidates, even if unchanged.
use std::collections::BTreeMap;
#[derive(Default)]
struct Entry {
    binding: Option<(u32, u32)>,
    pending: bool,
    crossed: bool,
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
            self.entries.clear();
            count
        }
    }
    pub fn forget(&mut self, handle: u32) -> usize {
        usize::from(
            self.entries
                .remove(&handle)
                .is_some_and(|e| e.pending && e.crossed),
        )
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
        *e = Entry {
            binding: Some((pointer, bytes)),
            pending: false,
            crossed: false,
        };
        event
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(s.forget(1), 1);
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
