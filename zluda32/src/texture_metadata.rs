//! Successful texture metadata replies for the serialized, single-device bridge.
//! These are host-side setters: an identical value does not refresh the HIP object.
//! This never caches address binds, offsets, synchronization or failed setters.
//! The IPC owner must clear this cache at lifetime/unknown-operation boundaries.
use std::collections::BTreeMap;

const LIMIT: usize = 64;

#[derive(Default)]
struct Entry {
    flags: Option<u32>,
    format: Option<(u32, i32)>,
}

#[derive(Default)]
pub(crate) struct TextureMetadata {
    entries: BTreeMap<u32, Entry>,
}

impl TextureMetadata {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn flags_match(&self, handle: u32, value: u32) -> bool {
        self.entries.get(&handle).is_some_and(|e| e.flags == Some(value))
    }

    pub fn format_matches(&self, handle: u32, format: u32, channels: i32) -> bool {
        self.entries.get(&handle).is_some_and(|e| e.format == Some((format, channels)))
    }

    fn entry(&mut self, handle: u32) -> &mut Entry {
        if self.entries.len() >= LIMIT && !self.entries.contains_key(&handle) {
            self.clear();
        }
        self.entries.entry(handle).or_default()
    }

    pub fn flags_result(&mut self, handle: u32, flags: u32, success: bool) {
        if success {
            self.entry(handle).flags = Some(flags);
        } else {
            // The setter may have changed host state before reporting an error.
            self.clear();
        }
    }

    pub fn format_result(&mut self, handle: u32, format: u32, channels: i32, success: bool) {
        if success {
            self.entry(handle).format = Some((format, channels));
        } else {
            self.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_exact_successful_values_hit() {
        let mut cache = TextureMetadata::default();
        assert!(!cache.flags_match(10, 0));
        assert!(!cache.format_matches(10, 0, 0));
        cache.flags_result(10, 1, true);
        cache.format_result(10, 32, 4, true);
        assert!(cache.flags_match(10, 1));
        assert!(cache.format_matches(10, 32, 4));
        assert!(!cache.flags_match(11, 1));
        assert!(!cache.flags_match(10, 0));
        assert!(!cache.format_matches(10, 32, 2));
        assert!(!cache.format_matches(10, 16, 4));
        cache.flags_result(10, 2, true);
        assert!(!cache.flags_match(10, 1));
        assert!(cache.flags_match(10, 2));
        assert!(cache.format_matches(10, 32, 4));
    }

    #[test]
    fn errors_clear_both_fields_and_other_handles() {
        let mut cache = TextureMetadata::default();
        cache.flags_result(10, 1, true);
        cache.format_result(11, 32, 4, true);
        cache.flags_result(10, 2, false);
        assert!(!cache.flags_match(10, 1));
        assert!(!cache.flags_match(10, 2));
        assert!(!cache.format_matches(11, 32, 4));
        cache.flags_result(10, 1, true);
        cache.format_result(11, 32, 4, false);
        assert!(!cache.flags_match(10, 1));
        assert!(!cache.format_matches(11, 32, 4));
    }

    #[test]
    fn lifetime_reset_prevents_recycled_handle_hits() {
        let mut cache = TextureMetadata::default();
        cache.flags_result(10, 1, true);
        cache.format_result(10, 32, 4, true);
        cache.clear();
        assert!(!cache.flags_match(10, 1));
        assert!(!cache.format_matches(10, 32, 4));
        cache.format_result(10, 16, 1, true);
        assert!(!cache.flags_match(10, 1));
        assert!(cache.format_matches(10, 16, 1));
    }

    #[test]
    fn bounded_cache_keeps_existing_entries_until_capacity_is_exceeded() {
        let mut cache = TextureMetadata::default();
        for handle in 0..LIMIT as u32 {
            cache.flags_result(handle, 1, true);
        }
        cache.format_result(0, 32, 4, true);
        assert!(cache.flags_match(0, 1));
        assert_eq!(cache.entries.len(), LIMIT);
        cache.flags_result(LIMIT as u32, 2, true);
        assert_eq!(cache.entries.len(), 1);
        assert!(!cache.flags_match(0, 1));
    }
}
