# Probe OFF capture and comparison

Verified installation manifest profile diagnostic, split_unbind false, settings checksum, runtime19714c9. No probe rows; no reported API errors. User24–25FPS. Five complete windows25.0022309 seconds.

Bound unbind:24590 calls,16.2125772 sec, mean659.316 us,81.88% of19.7992937 sec server service. Already-unbound:24577 calls, mean0.405us. Prior probe ON: bound mean691.813us; already-unbound18.330us (includes added synchronization). ON residual unbind15.381s after1.096s explicit sync confirms dominant cost persists after pre-sync, but no exact internal breakdown is possible.

Workload is NOT matched: OFF23977 kernel launches (~959/sec) vs ON16134 (~645/sec). OFF20383 changed address/size binds and4194 same rebinds; ON23238 same rebinds and no changed-bind row. Same camera location does not ensure same physics workload. Cannot attribute user FPS difference to probe or claim optimization. No recommendation to leave extra sync enabled; keep it off.

Next repo-only investigation should account for binding size/address churn before proposing object pooling or texture representation changes. No demonstrated small safe resource-lifetime optimization currently. Agent performed read-only remote collection only; user owns push/build/install.
