# PhysX workflow

Use [PUSH_BUILD.md](PUSH_BUILD.md) for push/build/install automation. The current installer is `legion_install.py`, invoked by `deploy_legion.py`: control server, matching proxy, range cache ON, texture deferral and diagnostic logging OFF.

The previous `install_experiment.py` and `update_*.py` files are historical fixed-package installers, not interchangeable generic updaters. Keep them as provenance; use the current workflow for new deployments.

See ../../Codex History/CURRENT.md for installed state and PATH_OUTCOME.md for the closed investigation. Backups and evidence are preserved.
