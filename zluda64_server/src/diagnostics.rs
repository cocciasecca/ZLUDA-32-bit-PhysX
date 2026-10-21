//! Request service time, deliberately excluding SignalObjectAndWait/client idle time.
//! Compile with physx-diagnostics and set ZLUDA_PHYSX_DIAGNOSTICS to a NEW CSV path.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = RefCell::new(Recorder::open());
}

#[derive(Default)]
struct Stats {
    calls: u64,
    errors: u64,
    total_ns: u128,
    max_ns: u128,
}

impl Stats {
    fn record(&mut self, ns: u128, status: u32) {
        self.calls += 1;
        self.errors += u64::from(status != 0);
        self.total_ns += ns;
        self.max_ns = self.max_ns.max(ns);
    }
}

struct Recorder {
    file: File,
    origin: Instant,
    window: Instant,
    stats: BTreeMap<&'static str, Stats>,
}

// A launcher and the game can inherit the same environment but create separate servers.
// Keep create_new protection while giving each server its own output file.
fn process_output_path(mut base: std::ffi::OsString, pid: u32) -> PathBuf {
    base.push(format!(".pid-{pid}.csv"));
    PathBuf::from(base)
}

impl Recorder {
    fn open() -> Option<Self> {
        let path = process_output_path(
            std::env::var_os("ZLUDA_PHYSX_DIAGNOSTICS")?,
            std::process::id(),
        );
        // Never truncate existing data. An invalid path disables diagnostics.
        let mut file = OpenOptions::new().write(true).create_new(true).open(path).ok()?;
        let epoch = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_nanos();
        writeln!(file, "# pid={},unix_ns={epoch},git={}", std::process::id(),
            option_env!("VERGEN_GIT_SHA").unwrap_or("standalone-test")).ok()?;
        writeln!(file, "window_start_ns,window_end_ns,request,calls,errors,total_ns,max_ns").ok()?;
        let origin = Instant::now();
        Some(Self { file, origin, window: origin, stats: BTreeMap::new() })
    }

    fn record(&mut self, name: &'static str, elapsed: Duration, status: u32, now: Instant) -> io::Result<()> {
        self.stats.entry(name).or_default().record(elapsed.as_nanos(), status);
        if now.duration_since(self.window) >= Duration::from_secs(5) {
            let start = self.window.duration_since(self.origin).as_nanos();
            let end = now.duration_since(self.origin).as_nanos();
            // At most one row per compiled request type per window; no event log.
            let mut batch = Vec::new();
            for (name, stats) in &self.stats {
                writeln!(batch, "{start},{end},{name},{},{},{},{}",
                    stats.calls, stats.errors, stats.total_ns, stats.max_ns)?;
            }
            self.file.write_all(&batch)?;
            self.stats.clear();
            self.window = now;
        }
        Ok(())
    }
}

pub struct Timer {
    name: &'static str,
    start: Option<Instant>,
}

impl Timer {
    pub fn start(name: &'static str) -> Self {
        let start = RECORDER.with(|recorder| {
            if recorder.borrow().is_some() { Some(Instant::now()) } else { None }
        });
        Self { name, start }
    }

    pub fn finish(self, status: u32) {
        let Some(start) = self.start else { return };
        let now = Instant::now();
        RECORDER.with(|recorder| {
            let mut recorder = recorder.borrow_mut();
            if let Some(active) = recorder.as_mut() {
                if active.record(self.name, now.duration_since(start), status, now).is_err() {
                    // Diagnostics must not change CUDA error propagation or panic on I/O failure.
                    *recorder = None;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_output_path_does_not_collide_between_servers() {
        let base = std::env::temp_dir().join(format!("zluda-multiprocess-{}", std::process::id()));
        let launcher = process_output_path(base.clone().into_os_string(), 416);
        let game = process_output_path(base.into_os_string(), 1428);
        assert_ne!(launcher, game);
        for path in [&launcher, &game] {
            let mut file = OpenOptions::new().write(true).create_new(true).open(path).unwrap();
            file.write_all(b"preserve-existing-data").unwrap();
        }
        assert!(OpenOptions::new().write(true).create_new(true).open(&launcher).is_err());
        assert_eq!(std::fs::read(&launcher).unwrap(), b"preserve-existing-data");
        std::fs::remove_file(launcher).unwrap();
        std::fs::remove_file(game).unwrap();
    }

    #[test]
    fn aggregates_success_failure_and_tail_latency() {
        let mut stats = Stats::default();
        stats.record(10, 0);
        stats.record(900, 1);
        stats.record(20, 0);
        assert_eq!((stats.calls, stats.errors, stats.total_ns, stats.max_ns), (3, 1, 930, 900));
    }

    #[test]
    fn window_output_separates_request_types_and_resets() {
        let path = std::env::temp_dir().join(format!("zluda-diagnostic-test-{}.csv", std::process::id()));
        let file = OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        let origin = Instant::now();
        let mut recorder = Recorder { file, origin, window: origin, stats: BTreeMap::new() };
        recorder.record("copy", Duration::from_nanos(100), 0, origin).unwrap();
        recorder.record("sync", Duration::from_nanos(900), 1, origin + Duration::from_secs(5)).unwrap();
        let output = std::fs::read_to_string(&path).unwrap();
        assert!(output.contains("0,5000000000,copy,1,0,100,100\n"));
        assert!(output.contains("0,5000000000,sync,1,1,900,900\n"));
        assert!(recorder.stats.is_empty());
        drop(recorder);
        std::fs::remove_file(path).unwrap();
    }
}
