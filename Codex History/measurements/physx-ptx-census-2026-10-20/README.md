# Installed PhysX PTX census — 2026-10-20

Read-only extraction from the installed PhysXCore.dll, SHA256 recorded in census.json. 18 compressed PTX modules were extracted locally. This is a lexical census, not a parser-backed proof of safety. Vendor DLL and PTX are not published.

14 modules have no texture declarations; four contain textures. No call instructions or texture-symbol references outside declarations and tex instructions were detected by the lexical scan. These are useful candidates for stricter analysis, not permission to skip a barrier. The runtime aggregate does not identify which named kernels account for cleanup time.

Next experiment: a bounded shadow model of pending texture bindings across candidate kernel boundaries. Count whether the next relevant operation would reuse the same binding, require a changed binding, or force cleanup for a lifetime/unknown operation. Keep actual cleanup unchanged. Only implement selective retention if the model finds useful reuse and parser-based descriptor-access checks establish eligibility. Preserve context synchronization and resource-lifetime invalidations. HIP remains unchanged.

No new runtime was built or installed during this investigation. User's last observation: 24–25 FPS; no controlled improvement demonstrated.
