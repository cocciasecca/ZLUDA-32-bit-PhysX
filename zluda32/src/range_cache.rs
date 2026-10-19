//! Successful server replies only; never infer rounded allocation sizes locally.
use std::collections::BTreeMap;
#[derive(Default)]
pub(crate) struct RangeCache {
    entries: BTreeMap<u32, u32>,
}
impl RangeCache {
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn get(&self, pointer: u32) -> Option<(u32, u32)> {
        let (&base, &size) = self.entries.range(..=pointer).next_back()?;
        ((pointer as u64) < base as u64 + size as u64).then_some((base, size))
    }
    pub fn insert(&mut self, pointer: u32, base: u32, size: u32) {
        let end = base as u64 + size as u64;
        if size == 0 || end > (1u64 << 32) || pointer < base || pointer as u64 >= end {
            return;
        }
        // Replies should never overlap. Forget old metadata if they do.
        if self.entries.len() >= 256
            || self
                .entries
                .iter()
                .any(|(&b, &s)| (b as u64) < end && (base as u64) < b as u64 + s as u64)
        {
            self.clear();
        }
        self.entries.insert(base, size);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interior_and_end() {
        let mut c = RangeCache::default();
        c.insert(300, 256, 256);
        assert_eq!(c.get(256), Some((256, 256)));
        assert_eq!(c.get(511), Some((256, 256)));
        assert_eq!(c.get(255), None);
        assert_eq!(c.get(512), None);
    }
    #[test]
    fn invalid_replies_and_top_address() {
        let mut c = RangeCache::default();
        c.insert(1, 2, 4);
        c.insert(0, 0, 0);
        c.insert(u32::MAX, u32::MAX, 2);
        assert!(c.entries.is_empty());
        c.insert(u32::MAX, u32::MAX, 1);
        assert_eq!(c.get(u32::MAX), Some((u32::MAX, 1)));
    }
    #[test]
    fn lifetime_and_address_reuse() {
        let mut c = RangeCache::default();
        c.insert(256, 256, 512);
        c.clear();
        assert_eq!(c.get(256), None);
        c.insert(256, 256, 256);
        assert_eq!(c.get(600), None);
    }
    #[test]
    fn capacity_is_bounded() {
        let mut c = RangeCache::default();
        for i in 0..1024 {
            c.insert(i * 256, i * 256, 256);
            assert!(c.entries.len() <= 256);
        }
        assert!(c.get(0).is_none());
    }
    #[test]
    fn overlapping_reply_forgets_stale_range() {
        let mut c = RangeCache::default();
        c.insert(256, 256, 512);
        c.insert(512, 512, 512);
        assert_eq!(c.get(256), None);
        assert_eq!(c.get(512), Some((512, 512)));
    }
    #[test]
    fn model_queries_across_lifetimes() {
        let mut c = RangeCache::default();
        for generation in 0..100 {
            c.clear();
            let base = 256 * (generation + 1);
            let size = 256 * (1 + generation % 5);
            c.insert(base, base, size);
            for p in [base - 1, base, base + size - 1, base + size] {
                assert_eq!(
                    c.get(p),
                    if p >= base && p < base + size {
                        Some((base, size))
                    } else {
                        None
                    }
                );
            }
        }
    }
}
