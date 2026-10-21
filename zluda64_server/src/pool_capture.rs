//! One bounded capture: request sequence + five LRU models. Never changes GPU calls.
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs::OpenOptions,
    io::Write,
    time::Instant,
};
const CAPS: [usize; 5] = [8, 16, 32, 64, 128];
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Bind(u32, u32, u32),
    Flags(u32, u32),
    Format(u32, u32, i32),
    Kernel(u32),
    Reset,
    Offset(u32),
    Other(u32),
}
#[derive(Clone, Copy, Default)]
struct Config {
    flags: Option<u32>,
    format: Option<(u32, i32)>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    generation: u64,
    pointer: u32,
    bytes: u32,
    flags: u32,
    format: u32,
    channels: i32,
}
#[derive(Default, Clone)]
struct Stats {
    hits: u64,
    misses: u64,
    evictions: u64,
}
#[derive(Default)]
struct Model {
    config: BTreeMap<u32, Config>,
    generation: u64,
    pools: [VecDeque<Key>; 5],
    stats: [Stats; 5],
    binds: u64,
    unknown: u64,
    unbinds: u64,
    launches: u64,
    errors: u64,
    resets: u64,
    distinct: BTreeSet<(u64, u32, u32, u32, u32, i32)>,
    distinct_truncated: bool,
    nonzero_offsets: u64,
    skip_binding: bool,
}
impl Model {
    fn apply(&mut self, event: Event, status: u32) {
        if status != 0 {
            self.errors += 1;
            self.reset();
            return;
        }
        match event {
            Event::Offset(offset) => {
                self.skip_binding = offset != 0;
                if offset != 0 {
                    self.nonzero_offsets += 1;
                }
            }
            Event::Other(opcode) => {
                let _ = opcode;
            }
            Event::Reset => self.reset(),
            Event::Kernel(handle) => {
                let _ = handle;
                self.launches += 1;
            }
            Event::Flags(h, v) => {
                self.prepare(h);
                self.config.entry(h).or_default().flags = Some(v);
            }
            Event::Format(h, f, c) => {
                self.prepare(h);
                self.config.entry(h).or_default().format = Some((f, c));
            }
            Event::Bind(_, 0, _) => self.unbinds += 1,
            Event::Bind(h, p, b) => {
                self.binds += 1;
                if self.skip_binding {
                    self.unknown += 1;
                    self.skip_binding = false;
                    return;
                }
                let Some(Config {
                    flags: Some(flags),
                    format: Some((format, channels)),
                }) = self.config.get(&h).copied()
                else {
                    self.unknown += 1;
                    return;
                };
                let k = Key {
                    generation: self.generation,
                    pointer: p,
                    bytes: b,
                    flags,
                    format,
                    channels,
                };
                if self.distinct.len() < 4096 {
                    self.distinct
                        .insert((self.generation, p, b, flags, format, channels));
                } else {
                    self.distinct_truncated = true;
                }
                for (i, cap) in CAPS.iter().enumerate() {
                    if let Some(pos) = self.pools[i].iter().position(|x| *x == k) {
                        self.stats[i].hits += 1;
                        self.pools[i].remove(pos);
                    } else {
                        self.stats[i].misses += 1;
                        if self.pools[i].len() == *cap {
                            self.pools[i].pop_front();
                            self.stats[i].evictions += 1;
                        }
                    }
                    self.pools[i].push_back(k);
                }
            }
        }
    }
    fn prepare(&mut self, h: u32) {
        if !self.config.contains_key(&h) && self.config.len() >= 512 {
            self.reset();
        }
    }
    fn reset(&mut self) {
        self.generation += 1;
        self.resets += 1;
        self.config.clear();
        for p in &mut self.pools {
            p.clear();
        }
    }
}
pub struct Capture {
    model: Model,
    file: std::fs::File,
    start: Option<Instant>,
    window: u64,
    ring: VecDeque<(u128, Event, u32)>,
    names: BTreeMap<(bool, u32), String>,
    done: bool,
    trigger: std::path::PathBuf,
    created: std::time::SystemTime,
    last_check: Instant,
}
impl Capture {
    pub fn open() -> Option<Self> {
        if std::env::var("ZLUDA_PHYSX_POOL_CAPTURE").as_deref() != Ok("1") {
            return None;
        }
        let mut path = std::env::var_os("ZLUDA_PHYSX_DIAGNOSTICS")?;
        let trigger = std::path::PathBuf::from(&path)
            .parent()?
            .join("pool-capture.start");
        path.push(format!(".pool.pid-{}.txt", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .ok()?;
        writeln!(file,"# Cumulative 5s LRU snapshots, 60s after pool-capture.start trigger; last 3000 completed events. Optimistic descriptor-key reuse, NOT safe eviction or speedup proof.").ok()?;
        Some(Self {
            model: Model::default(),
            file,
            start: None,
            window: 0,
            ring: VecDeque::new(),
            names: BTreeMap::new(),
            done: false,
            trigger,
            created: std::time::SystemTime::now(),
            last_check: Instant::now(),
        })
    }
    pub fn name(&mut self, kernel: bool, h: u32, name: &[u8]) {
        if !self.done && self.names.len() < 512 {
            self.names.insert(
                (kernel, h),
                name.iter().take(512).map(|b| format!("{b:02x}")).collect(),
            );
        }
    }
    pub fn observe(&mut self, event: Event, status: u32) {
        if self.done {
            return;
        }
        if self.start.is_none() {
            self.model.apply(event, status);
            if self.last_check.elapsed().as_secs() >= 1 {
                self.last_check = Instant::now();
                let armed = std::fs::metadata(&self.trigger)
                    .and_then(|m| m.modified())
                    .is_ok_and(|t| t >= self.created);
                if armed {
                    let config = std::mem::take(&mut self.model.config);
                    self.model = Model::default();
                    self.model.config = config;
                    self.start = Some(Instant::now());
                    let _ = writeln!(
                        self.file,
                        "started_unix_ns,{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    );
                    let _ = self.file.flush();
                }
            }
            return;
        }
        self.model.apply(event, status);
        let start = self.start.unwrap();
        let elapsed = start.elapsed();
        if self.ring.len() == 3000 {
            self.ring.pop_front();
        }
        self.ring.push_back((elapsed.as_micros(), event, status));
        if elapsed.as_secs() / 5 > self.window {
            self.window = elapsed.as_secs() / 5;
            let m = &self.model;
            let _ = writeln!(
                self.file,
                "coverage,{},{},{},{},{},{},{},{},{}",
                elapsed.as_millis(),
                m.binds,
                m.unbinds,
                m.launches,
                m.unknown,
                m.errors,
                m.resets,
                m.distinct.len(),
                m.distinct_truncated
            );
            let _ = writeln!(self.file, "offset_nonzero,{}", m.nonzero_offsets);
            for (i, cap) in CAPS.iter().enumerate() {
                let s = &m.stats[i];
                let _ = writeln!(
                    self.file,
                    "lru,{},{cap},{},{},{}",
                    elapsed.as_millis(),
                    s.hits,
                    s.misses,
                    s.evictions
                );
            }
            let _ = self.file.flush();
        }
        if elapsed.as_secs() >= 60 {
            for ((kernel, h), name) in &self.names {
                let _ = writeln!(self.file, "name,{kernel},{h},{name}");
            }
            for (t, event, status) in &self.ring {
                let _ = writeln!(self.file, "event,{t},{event:?},{status}");
            }
            let _ = writeln!(self.file, "complete");
            let _ = self.file.flush();
            self.done = true;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config(m: &mut Model, h: u32) {
        m.apply(Event::Flags(h, 0), 0);
        m.apply(Event::Format(h, 1, 4), 0);
    }
    #[test]
    fn full_sequence_reuses_keys_across_handles() {
        let mut m = Model::default();
        config(&mut m, 1);
        config(&mut m, 2);
        m.apply(Event::Bind(1, 100, 64), 0);
        m.apply(Event::Kernel(3), 0);
        m.apply(Event::Bind(1, 0, 0), 0);
        m.apply(Event::Bind(2, 100, 64), 0);
        assert_eq!(m.stats[0].hits, 1);
        assert_eq!(m.stats[0].misses, 1);
    }
    #[test]
    fn changes_unknown_errors_and_lifetime_are_not_hits() {
        let mut m = Model::default();
        m.apply(Event::Bind(1, 100, 64), 0);
        assert_eq!(m.unknown, 1);
        config(&mut m, 1);
        m.apply(Event::Bind(1, 100, 64), 0);
        m.apply(Event::Bind(1, 100, 65), 0);
        m.apply(Event::Reset, 0);
        config(&mut m, 1);
        m.apply(Event::Bind(1, 100, 64), 0);
        assert_eq!(m.stats[0].hits, 0);
        m.apply(Event::Bind(1, 100, 64), 1);
        assert!(m.pools[0].is_empty());
    }
    #[test]
    fn capacities_and_lru_eviction() {
        let mut m = Model::default();
        config(&mut m, 1);
        for p in 1..=9 {
            m.apply(Event::Bind(1, p, 64), 0);
        }
        m.apply(Event::Bind(1, 1, 64), 0);
        assert_eq!(m.stats[0].hits, 0);
        assert_eq!(m.stats[0].evictions, 2);
        assert_eq!(m.stats[1].hits, 1);
        for p in 10..1000 {
            m.apply(Event::Bind(1, p, 64), 0);
        }
        for (i, cap) in CAPS.iter().enumerate() {
            assert!(m.pools[i].len() <= *cap);
        }
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    #[test]
    fn nonzero_offset_is_excluded() {
        let mut m = Model::default();
        m.apply(Event::Flags(1, 0), 0);
        m.apply(Event::Format(1, 1, 4), 0);
        m.apply(Event::Offset(8), 0);
        m.apply(Event::Bind(1, 100, 64), 0);
        assert_eq!(m.unknown, 1);
        assert_eq!(m.stats[0].misses, 0);
        m.apply(Event::Offset(0), 0);
        m.apply(Event::Bind(1, 100, 64), 0);
        assert_eq!(m.stats[0].misses, 1);
    }
    #[test]
    fn lru_matches_independent_recent_history() {
        let mut m = Model::default();
        m.apply(Event::Flags(1, 0), 0);
        m.apply(Event::Format(1, 1, 4), 0);
        let mut history = Vec::new();
        let mut hits = [0u64; 5];
        let mut rng = 1u64;
        for _ in 0..2000 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = ((rng >> 32) % 200 + 1) as u32;
            for (i, cap) in CAPS.iter().enumerate() {
                let recent: BTreeSet<_> = history
                    .iter()
                    .rev()
                    .copied()
                    .scan(BTreeSet::new(), |seen, x| {
                        Some(if seen.insert(x) { Some(x) } else { None })
                    })
                    .flatten()
                    .take(*cap)
                    .collect();
                if recent.contains(&p) {
                    hits[i] += 1;
                }
            }
            m.apply(Event::Bind(1, p, 64), 0);
            history.push(p);
        }
        for (i, h) in hits.iter().enumerate() {
            assert_eq!(m.stats[i].hits, *h);
            assert_eq!(m.stats[i].hits + m.stats[i].misses, 2000);
        }
    }
    #[test]
    fn capture_finishes_once_and_ring_is_bounded() {
        let path = std::env::temp_dir().join(format!(
            "pool-test-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let mut c = Capture {
            model: Model::default(),
            file,
            start: Some(Instant::now()),
            window: 0,
            ring: VecDeque::new(),
            names: BTreeMap::new(),
            done: false,
            trigger: path.with_extension("start"),
            created: std::time::SystemTime::now(),
            last_check: Instant::now(),
        };
        for _ in 0..4000 {
            c.observe(Event::Kernel(1), 0);
        }
        assert_eq!(c.ring.len(), 3000);
        c.start = Some(Instant::now() - std::time::Duration::from_secs(61));
        c.observe(Event::Kernel(1), 0);
        assert!(c.done);
        let before = std::fs::read(&path).unwrap();
        c.observe(Event::Kernel(1), 0);
        assert_eq!(before, std::fs::read(&path).unwrap());
        drop(c);
        std::fs::remove_file(path).unwrap();
    }
}
