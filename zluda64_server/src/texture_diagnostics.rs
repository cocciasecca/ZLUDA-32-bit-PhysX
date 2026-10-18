//! Observes successful address changes; never suppresses or alters CUDA calls.
//! "Same" means equal address and size only, NOT equivalent texture/resource state.
use std::collections::BTreeMap;

const LIMIT: usize = 4096;

#[derive(Default)]
pub struct TextureHistory {
    entries: BTreeMap<u32, Binding>,
}

#[derive(Clone, Copy)]
struct Binding {
    last: Option<(u32, u32)>,
    bound: bool,
}

impl TextureHistory {
    pub fn classify(&self, handle: u32, pointer: u32, bytes: u32) -> &'static str {
        let old = self.entries.get(&handle);
        if pointer == 0 {
            return match old {
                Some(Binding { bound: true, .. }) => "texture_address/unbind_bound",
                Some(_) => "texture_address/unbind_already_unbound",
                None => "texture_address/unbind_unknown",
            };
        }
        match old {
            None => "texture_address/bind_unknown",
            Some(binding) if binding.last == Some((pointer, bytes)) => {
                if binding.bound {
                    "texture_address/rebind_same_address_size"
                } else {
                    "texture_address/bind_same_after_unbind"
                }
            }
            Some(_) => "texture_address/bind_changed_address_size",
        }
    }

    // Call only after CUDA success. Capacity reset is conservative: next call is unknown.
    pub fn succeeded(&mut self, handle: u32, pointer: u32, bytes: u32) {
        if !self.entries.contains_key(&handle) && self.entries.len() >= LIMIT {
            self.clear();
        }
        let entry = self.entries.entry(handle).or_insert(Binding {
            last: None,
            bound: false,
        });
        entry.bound = pointer != 0;
        if entry.bound {
            entry.last = Some((pointer, bytes));
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_unbind_rebind_sequence_and_size_change() {
        let mut h = TextureHistory::default();
        assert_eq!(h.classify(1, 256, 64), "texture_address/bind_unknown");
        h.succeeded(1, 256, 64);
        assert_eq!(
            h.classify(1, 256, 64),
            "texture_address/rebind_same_address_size"
        );
        assert_eq!(
            h.classify(1, 256, 128),
            "texture_address/bind_changed_address_size"
        );
        assert_eq!(h.classify(1, 0, 0), "texture_address/unbind_bound");
        h.succeeded(1, 0, 0);
        assert_eq!(
            h.classify(1, 0, 0),
            "texture_address/unbind_already_unbound"
        );
        assert_eq!(
            h.classify(1, 256, 64),
            "texture_address/bind_same_after_unbind"
        );
        assert_eq!(h.classify(2, 0, 0), "texture_address/unbind_unknown");
    }

    #[test]
    fn classification_does_not_change_history_on_failed_calls() {
        let mut h = TextureHistory::default();
        h.succeeded(1, 256, 64);
        let _ = h.classify(1, 512, 128); // Failed CUDA call: do not call succeeded.
        assert_eq!(
            h.classify(1, 256, 64),
            "texture_address/rebind_same_address_size"
        );
        h.clear();
        assert_eq!(h.classify(1, 256, 64), "texture_address/bind_unknown");
    }

    #[test]
    fn memory_is_bounded_and_overflow_forgets_stale_handles() {
        let mut h = TextureHistory::default();
        for id in 0..=LIMIT as u32 {
            h.succeeded(id, 256, 64);
        }
        assert_eq!(h.entries.len(), 1);
        assert_eq!(h.classify(0, 256, 64), "texture_address/bind_unknown");
    }
}
