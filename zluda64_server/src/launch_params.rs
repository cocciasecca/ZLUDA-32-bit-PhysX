//! Scratch storage for the serialized server's kernel argument preparation.
//! Only capacity survives a launch. Argument values/pointers are rebuilt each time,
//! and native cuLaunchKernel must consume the parameter array before returning.
use std::{ffi::c_void, ptr};

#[derive(Default)]
pub(crate) struct LaunchParams {
    params: Vec<*mut c_void>,
    globals: Vec<u32>,
}

impl LaunchParams {
    pub fn with_arguments<R>(
        &mut self,
        arguments: impl IntoIterator<Item = *mut c_void>,
        mut arena_base: *mut c_void,
        globals: impl IntoIterator<Item = u32>,
        launch: impl FnOnce(&mut [*mut c_void]) -> R,
    ) -> R {
        self.params.clear();
        self.globals.clear();
        // Finish growing the values vector before taking any pointers into it.
        self.globals.extend(globals);
        self.params.extend(arguments);
        self.params.push(ptr::from_mut(&mut arena_base).cast());
        self.params.extend(
            self.globals.iter().map(|value| ptr::from_ref(value).cast_mut().cast()),
        );
        let result = launch(&mut self.params);
        // This also runs for a native Result::Err. No pointer from this request
        // remains logically present when the shared IPC buffer is reused.
        self.params.clear();
        self.globals.clear();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argument_order_widths_and_values_match_native_layout() {
        let mut scratch = LaunchParams::default();
        let mut arg = 0x1234_u64;
        let arg_ptr = ptr::from_mut(&mut arg).cast();
        let arena = 0x1234567890_usize as *mut c_void;
        scratch.with_arguments([arg_ptr], arena, [256, u32::MAX], |params| {
            assert_eq!(params.len(), 4);
            assert_eq!(params[0], arg_ptr);
            unsafe {
                assert_eq!(*params[0].cast::<u64>(), arg);
                assert_eq!(*params[1].cast::<*mut c_void>(), arena);
                assert_eq!(*params[2].cast::<u32>(), 256);
                assert_eq!(*params[3].cast::<u32>(), u32::MAX);
            }
        });
        assert!(scratch.params.is_empty());
        assert!(scratch.globals.is_empty());
    }

    #[test]
    fn growth_then_smaller_launch_reuses_capacity_without_stale_values() {
        let mut scratch = LaunchParams::default();
        let mut capacity = None;
        for count in [0, 1024, 3, 0, 1024] {
            let values: Vec<u32> = (0..count).map(|i| i * 256 + 7).collect();
            scratch.with_arguments([], ptr::null_mut(), values.iter().copied(), |params| {
                assert_eq!(params.len(), count as usize + 1);
                for (parameter, expected) in params[1..].iter().zip(&values) {
                    assert_eq!(unsafe { *parameter.cast::<u32>() }, *expected);
                }
            });
            if let Some((params, globals)) = capacity {
                assert_eq!(scratch.params.capacity(), params);
                assert_eq!(scratch.globals.capacity(), globals);
            }
            if count == 1024 {
                capacity = Some((scratch.params.capacity(), scratch.globals.capacity()));
            }
        }
    }

    #[test]
    fn failed_launch_does_not_retain_arguments_for_next_function() {
        let mut scratch = LaunchParams::default();
        let result: Result<(), u32> = scratch.with_arguments([], ptr::null_mut(), [123], |_| Err(42));
        assert_eq!(result, Err(42));
        assert!(scratch.params.is_empty());
        assert!(scratch.globals.is_empty());
        scratch.with_arguments([], 123_usize as *mut c_void, [], |params| {
            assert_eq!(params.len(), 1);
            assert_eq!(unsafe { *params[0].cast::<*mut c_void>() }, 123_usize as *mut c_void);
        });
    }
}
