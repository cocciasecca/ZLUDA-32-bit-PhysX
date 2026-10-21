# Validated shadow coverage — 2026-10-21

Runtime 3e9ea162d124d3e7e118465f8f920e007d9d162f verified. User reports 29 FPS after advancing in game and returning to same position. Not a controlled comparison with prior sessions.

Five complete windows: 25.0171545 seconds. Coverage passes: 76962 observed address calls equal all texture address classifications; 51312 observed unbinds equal unbind classifications; 17810 observed launches equal launch requests. No reported API errors. 14960 candidate kernels, 179520 pending-at-candidate occurrences (not unique resources), 17100 kernel-ended candidate binding episodes. No same/changed rebind-after-candidate events, configuration ends, capacity resets or error resets in selected windows.

Within the modeled policy (defer only past NoTextureInstructions kernels, preserve known identical configuration, conservative reset at other boundaries), no reuse opportunities were observed. Candidate pending bindings reached a noncandidate kernel before reuse. Aggregate kernel_end does not distinguish texture-using from unknown; no claim of exact named kernel sequence. Outstanding episodes at capture boundaries are censored. This supports stopping this specific deferral-only optimization for this scene; it does not rule out other reuse strategies.

14.6557519 sec cleanup / 19.377307 sec measured service (~75.6%). Not frame time. Shader warmup is plausible but unmeasured; no graphics compiler trace captured. Current version changes diagnostics, not cleanup policy. No FPS gain attributable to this change.
