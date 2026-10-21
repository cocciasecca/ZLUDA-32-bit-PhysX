//! Diagnostic perturbation, not an optimization or exact decomposition of native costs.
pub fn run<T, E>(
    enabled: bool,
    mut sync: impl FnMut() -> Result<(), E>,
    mut unbind: impl FnMut() -> Result<T, E>,
) -> Result<T, E> {
    if enabled {
        sync()?;
    }
    unbind()
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[test]
    fn preserves_order_and_result() {
        let events = RefCell::new(Vec::new());
        let result = run(
            true,
            || {
                events.borrow_mut().push("sync");
                Ok::<_, u32>(())
            },
            || {
                events.borrow_mut().push("unbind");
                Ok(42)
            },
        );
        assert_eq!(result, Ok(42));
        assert_eq!(*events.borrow(), ["sync", "unbind"]);
    }
    #[test]
    fn sync_failure_stops_unbind() {
        assert_eq!(
            run(
                true,
                || Err(7),
                || -> Result<(), u32> { panic!("unbind after failed sync") }
            ),
            Err(7)
        );
    }
    #[test]
    fn disabled_probe_skips_sync_and_preserves_error() {
        assert_eq!(
            run(
                false,
                || -> Result<(), u32> { panic!("unexpected sync") },
                || Err::<(), _>(9)
            ),
            Err(9)
        );
    }
}
