# Update agent

Rebase this fork onto upstream (`herdrdev/herdr`) and make sure the release still builds.

1. `git fetch upstream && git rebase upstream/master`.
2. Resolve conflicts. The fork owns two commits: fork packaging, and the dev servers panel.
   The panel spans detection (`src/detect/dev_server*.rs`, `src/detect/dev-servers.toml`),
   platform socket/process lookups (`listening_sockets`, `descendant_pids`,
   `process_argv_for_pid` in `src/platform/*`), the pane detection task in `src/pane.rs`,
   server state in `src/app/{state,actions,mod}.rs`, the `dev_servers` snapshot projection in
   `src/protocol/wire.rs` + `src/server/client_shell.rs`, and rendering in
   `src/client/shell/dev_server_sidebar.rs`. Upstream churns `src/pane.rs` and the
   `src/client/shell/` tree most — expect conflicts there.
3. Validate: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo nextest run`, and
   `just windows-lint`. A macOS-only run will NOT catch Linux lints or Windows dead-code and
   import errors, and `cfg(windows)` code is not compiled at all on macOS.
4. Push (`--force-with-lease` after a rebase), then watch CI and the "Fork release" run on
   `hexxt-git/herdr`. Both must be green and the release must publish 5 assets.

Keep the diff minimal: no new config options (the panel is always on), no `website/`, `docs/`, or
CHANGELOG work, and keep the fork's README title, servers-panel note, and install curl.
Never push to upstream.

Snapshot fields added for the panel must stay `#[serde(default)]`: the endpoint contract requires
a generation-1 client to keep decoding a newer snapshot.

Keep `fork-release.yml` in step with upstream's `release.yml` for the Rust and Zig versions.
Upstream bumps them without touching the fork workflow, and a stale pin fails the release.
Building `vendor/libghostty-vt` fetches Zig dependencies over the network on a cold cache.

Flaky, not regressions — re-run before investigating: `live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`
and `reload_aborts_an_in_flight_command_task_and_its_descendants`. Markdown-only pushes skip the release.
