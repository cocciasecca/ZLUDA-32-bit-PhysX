//! Reuse owned launch bytes while preserving the existing Vec<Vec<u8>> wire type.
//! Layout is immutable function metadata. Only buffer capacity survives a call;
//! no pointer into the caller's packed argument buffer is retained.
use std::{alloc::Layout, ops::Range};

pub(crate) struct ArgumentLayout {
    ranges: Vec<Range<usize>>,
    size: usize,
}

impl ArgumentLayout {
    pub fn new(arguments: impl IntoIterator<Item = (usize, usize)>) -> Option<Self> {
        let mut layout = Layout::from_size_align(0, 1).ok()?;
        let mut ranges = Vec::new();
        let mut size = 0;
        for (bytes, alignment) in arguments {
            let argument = Layout::from_size_align(bytes, alignment).ok()?;
            let (next, offset) = layout.extend(argument).ok()?;
            size = offset.checked_add(bytes)?;
            ranges.push(offset..size);
            layout = next;
        }
        // Match the existing CUDA packed-buffer check: no final tail padding.
        Some(Self { ranges, size })
    }

    pub fn size(&self) -> usize { self.size }

    pub fn slices<'a>(&'a self, packed: &'a [u8]) -> Option<impl Iterator<Item = &'a [u8]>> {
        if packed.len() != self.size { return None; }
        Some(self.ranges.iter().map(move |range| &packed[range.clone()]))
    }
}

#[derive(Default)]
pub(crate) struct LaunchBuffers {
    active: Vec<Vec<u8>>,
    spare: Vec<Vec<u8>>,
}

impl LaunchBuffers {
    pub fn with_arguments<'a, R>(
        &mut self,
        slices: impl IntoIterator<Item = &'a [u8]>,
        call: impl FnOnce(Vec<Vec<u8>>) -> (Vec<Vec<u8>>, R),
    ) -> R {
        // Reversing preserves slot order when the next function has the same shape.
        self.spare.extend(self.active.drain(..).rev());
        for bytes in slices {
            let mut buffer = self.spare.pop().unwrap_or_default();
            buffer.clear();
            buffer.extend_from_slice(bytes);
            self.active.push(buffer);
        }
        // The caller borrows the request for serialization, then returns its buffers
        // even if IPC/native execution failed. The mutex covers this entire scope.
        let (mut buffers, result) = call(std::mem::take(&mut self.active));
        for buffer in &mut buffers { buffer.clear(); }
        self.active = buffers;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_alignment_matches_original_layout_and_excludes_padding() {
        let layout = ArgumentLayout::new([(1, 1), (4, 4), (2, 2)]).unwrap();
        assert_eq!(layout.size(), 10);
        let packed = [1, 99, 99, 99, 2, 3, 4, 5, 6, 7];
        let actual: Vec<_> = layout.slices(&packed).unwrap().map(<[u8]>::to_vec).collect();
        assert_eq!(actual, vec![vec![1], vec![2, 3, 4, 5], vec![6, 7]]);
        assert!(layout.slices(&packed[..9]).is_none());
        assert!(layout.slices(&[0; 11]).is_none());
        assert_eq!(ArgumentLayout::new([]).unwrap().size(), 0);
        assert!(ArgumentLayout::new([(1, 3)]).is_none());
        assert!(ArgumentLayout::new([(usize::MAX, 8)]).is_none());
    }

    #[test]
    fn repeated_shapes_reuse_all_inner_and_outer_allocations() {
        let mut buffers = LaunchBuffers::default();
        let slices = [b"abc".as_slice(), b"longer argument".as_slice()];
        let mut original = None;
        for _ in 0..3 {
            buffers.with_arguments(slices, |values| {
                assert_eq!(values, slices.iter().map(|s| s.to_vec()).collect::<Vec<_>>());
                let pointers = (values.as_ptr(), values[0].as_ptr(), values[1].as_ptr());
                if let Some(previous) = original { assert_eq!(previous, pointers); }
                original = Some(pointers);
                (values, ())
            });
            assert!(buffers.active.iter().all(Vec::is_empty));
        }
    }

    #[test]
    fn different_function_shapes_and_failed_calls_do_not_leak_old_arguments() {
        let mut buffers = LaunchBuffers::default();
        for count in [3, 0, 1, 8, 2, 3] {
            let source: Vec<Vec<u8>> = (0..count).map(|i| vec![i as u8; i * 33]).collect();
            let result: Result<(), u32> = buffers.with_arguments(source.iter().map(Vec::as_slice), |values| {
                assert_eq!(values, source);
                (values, Err(42))
            });
            assert_eq!(result, Err(42));
            assert!(buffers.active.iter().all(Vec::is_empty));
        }
        buffers.with_arguments([b"new".as_slice()], |values| {
            assert_eq!(values, vec![b"new".to_vec()]);
            (values, ())
        });
    }
}
