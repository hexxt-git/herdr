# Update agent

Rebase this fork onto upstream (`herdrdev/herdr`) and make sure the release still builds.

1. `git fetch upstream && git rebase upstream/master`.
2. Resolve conflicts. Our only change is the sidebar dev servers panel: `src/detect/dev_server.rs`,
   `src/ui/sidebar.rs`, `src/ui.rs`, `src/pane.rs`, `src/app/**`, and `listening_ports_for_pgrp` in
   `src/platform/*`. Upstream churns `ui/sidebar.rs` and `pane.rs` most — expect conflicts there.
3. Validate: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo nextest run`, and
   `cargo clippy --target x86_64-unknown-linux-musl --all-targets`. A macOS-only run will NOT catch
   Linux lints or Windows dead-code/import errors. Windows is CI-only.
4. Push (`--force-with-lease` after a rebase), then watch CI and the "Fork release" run on
   `hexxt-git/herdr`. Both must be green and the release must publish 5 assets.

Keep the diff minimal. Don't touch README, CHANGELOG, `website/`, or `docs/`. Don't add config
options — the panel is always on. Never push to upstream or open PRs/issues there.

Two tests are flaky and are not regressions: `live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`
and `reload_aborts_an_in_flight_command_task_and_its_descendants`. Re-run before investigating. Markdown-only
pushes skip the release. macOS local builds need Zig 0.15.2 plus an `xcrun --show-sdk-path` shim returning
MacOSX15.4.sdk (0.15.2 cannot link the macOS 26 SDK), or Homebrew `zig@0.15`.
