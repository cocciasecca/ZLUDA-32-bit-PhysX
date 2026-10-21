# Combined pool capture — 2026-10-21

Installed run39576382760, commitd8bf7d41ecb9577dac3eea758b94fffcf60c633c, profilepool-capture. User21FPS. Trigger armed after user confirmed scene. Completion marker present; duration60000ms;12 cumulative snapshots. No binary/settings changes by agent.

50029 binds,100069 unbinds,35084 launches; zero unknown configuration, errors, resets and nonzero offsets.41 distinct keys, not truncated. Every capacity reconciles hits+misses to50029 eligible binds.

LRU8:0hits/50029misses. LRU16:0/50029. LRU32:48822/1207 (97.5874%). LRU64 and128:49988/41 (99.9180%), zero evictions. This is optimistic descriptor-key locality, not measured native object reuse or FPS gain. No unknown/capacity/lifetime reset explains the high rate in this capture.

3000-event ring spans0.454535 seconds,18 distinct texture handles. Includes369 non-null binds (first has incomplete preceding context),368 preceded by format setter in per-handle history;369 flags setters while bound, no known changes in those flags. Thus order is not simply flags-format-address: observed unbind -> format -> address, then flags while bound, then kernel, then unbind. A pool must handle setters while active, not assume they occur before binding. Visible launch names include KernelDeformableUberCollisionPi and KernelDeformableUberSolver512. Names hex-decoded in summary.

Only predominantly same-binding regime observed in complete service windows; do not claim both workload regimes tested. Pool keys represent ptr/bytes/config/generation. Object pinning, actual descriptor ownership, unsupported proxy setters, cross-module compatibility and synchronization remain implementation requirements. Even high locality does not establish safe eviction.64 slots fit observed41keys; no promise of universal sizing.

Decision: positive evidence to implement a bounded opt-in prototype with native fallback, not another single-counter build. Preserve explicit completion/lifetime barriers, never let legacy HIP texref setters own pooled objects, refresh device reference after changed bound configuration, validate offset/layout, drain safely on resource lifetime transitions. No prototype yet installed. Source edits/push/build/install remain user-controlled as agreed.
