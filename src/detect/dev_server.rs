//! Dev server detection: tool identity from process argv and terminal screen,
//! port from OS socket inspection (Linux) and screen URL patterns.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevServerInfo {
    pub tool: &'static str,
    pub port: u16,
}

/// Identify a dev server tool from process argv.
pub fn detect_tool_from_argv(argv: &[String]) -> Option<&'static str> {
    if argv.is_empty() {
        return None;
    }

    let joined = argv.join(" ").to_lowercase();

    for token in argv {
        let base = path_basename(token).to_lowercase();
        match base.trim_end_matches(".js").trim_end_matches(".mjs") {
            "vite" => return Some("vite"),
            "nuxt" => return Some("nuxt"),
            "astro" => return Some("astro"),
            "parcel" => return Some("parcel"),
            "expo" => return Some("expo"),
            "gatsby" if joined.contains(" develop") || joined.contains(" dev") => {
                return Some("gatsby")
            }
            "storybook" | "start-storybook" => return Some("storybook"),
            "hexo" if joined.contains(" server") || joined.contains(" s") => return Some("hexo"),
            "hugo" => return Some("hugo"),
            "jekyll" if joined.contains(" serve") || joined.contains(" s") => {
                return Some("jekyll")
            }
            "eleventy" | "@11ty/eleventy" => return Some("eleventy"),
            "http-server" => return Some("http-server"),
            "serve" if joined.contains(" serve ") || joined.ends_with(" serve") => {
                return Some("serve")
            }
            "zola" if joined.contains(" serve") => return Some("zola"),
            "mdbook" if joined.contains(" serve") || joined.contains(" watch") => {
                return Some("mdbook")
            }
            "trunk" if joined.contains(" serve") || joined.contains(" watch") => {
                return Some("trunk")
            }
            "webpack-dev-server" | "webpack-dev-server.js" => return Some("webpack"),
            "ng" if joined.contains(" serve") => return Some("angular"),
            "next" if joined.contains(" dev") || joined.contains(" start") => {
                return Some("nextjs")
            }
            "nest" if joined.contains(" start") => return Some("nestjs"),
            "remix" if joined.contains(" dev") || joined.contains(" vite") => return Some("remix"),
            "react-scripts" if joined.contains(" start") => return Some("react"),
            "svelte-kit" | "sveltekit" => return Some("sveltekit"),
            "shadow-cljs" if joined.contains(" watch") || joined.contains(" server") => {
                return Some("shadow-cljs")
            }
            "uvicorn" | "hypercorn" => return Some("uvicorn"),
            "gunicorn" => return Some("gunicorn"),
            "flask" if joined.contains(" run") => return Some("flask"),
            "litestar" | "starlette" => return Some("litestar"),
            // Go
            "air" => return Some("air"),
            // PHP
            "artisan" if joined.contains(" serve") => return Some("laravel"),
            // Rust
            "cargo"
                if joined.contains(" run")
                    || joined.contains(" watch")
                    || joined.contains(" serve") =>
            {
                return Some("cargo")
            }
            // .NET / C#
            "dotnet" if joined.contains(" run") || joined.contains(" watch") => {
                return Some("dotnet")
            }
            // Gleam
            "gleam" if joined.contains(" run") || joined.contains(" watch") => {
                return Some("gleam")
            }
            // Databases
            "postgres" | "postgresql" | "pg_ctl" | "pg_ctlcluster" => return Some("postgres"),
            "mysqld" | "mariadbd" => return Some("mysql"),
            "mongod" => return Some("mongodb"),
            "redis-server" => return Some("redis"),
            "cockroach" if joined.contains(" start") || joined.contains(" demo") => {
                return Some("cockroachdb")
            }
            // DevOps / monitoring
            "prometheus" => return Some("prometheus"),
            "grafana-server" | "grafana" => return Some("grafana"),
            "consul" if joined.contains(" agent") || joined.contains(" dev") => {
                return Some("consul")
            }
            "vault" if joined.contains(" server") || joined.contains(" dev") => {
                return Some("vault")
            }
            "etcd" => return Some("etcd"),
            "minio" => return Some("minio"),
            "jaeger-all-in-one" | "jaeger-collector" | "jaeger-agent" => return Some("jaeger"),
            "zipkin" => return Some("zipkin"),
            // Shell-level servers
            "nc" | "ncat" | "netcat" if joined.contains(" -l") => return Some("netcat"),
            "socat" => return Some("socat"),
            _ => {}
        }
    }

    // Multi-token joined patterns
    if joined.contains("manage.py") && joined.contains("runserver") {
        return Some("django");
    }
    if joined.contains("webpack") && joined.contains("serve") {
        return Some("webpack");
    }
    if joined.contains("rails")
        && (joined.contains(" server") || joined.contains(" s ") || joined.ends_with(" s"))
    {
        return Some("rails");
    }
    if joined.contains("php") && joined.contains(" -s") {
        return Some("php");
    }
    if joined.contains("mix") && joined.contains("phx.server") {
        return Some("phoenix");
    }
    if joined.contains("spring-boot:run")
        || (joined.contains("gradle") && joined.contains("bootrun"))
    {
        return Some("spring");
    }
    if joined.contains("quarkus") && joined.contains(" dev") {
        return Some("quarkus");
    }
    if joined.contains("mvn") && joined.contains("spring-boot") {
        return Some("spring");
    }
    if joined.contains("lein") && (joined.contains(" ring") || joined.contains(" figwheel")) {
        return Some("clojure");
    }
    if joined.contains("sbt") && (joined.contains(" run") || joined.contains(" ~run")) {
        return Some("scala");
    }
    // python -m http.server
    if joined.contains("python") && joined.contains(" -m http.server") {
        return Some("http.server");
    }
    // ruby -run -e httpd (built-in file server)
    if joined.contains("ruby") && joined.contains("-run") && joined.contains("-e httpd") {
        return Some("ruby-httpd");
    }
    let first_base = argv.first().map(|a| path_basename(a)).unwrap_or("");
    if first_base == "bun"
        && (joined.contains(" dev") || joined.contains(" run") || joined.contains(" start"))
    {
        return Some("bun");
    }
    if first_base == "deno"
        && (joined.contains(" serve") || joined.contains(" task") || joined.contains(" run"))
    {
        return Some("deno");
    }

    None
}

/// Generic runtime fallback — covers plain scripts like `node server.mjs` or
/// `python app.py` when the screen has no recognisable tool banner.
fn detect_generic_runtime_from_argv(argv: &[String]) -> Option<&'static str> {
    let first = path_basename(argv.first().map(String::as_str).unwrap_or("")).to_lowercase();
    match first.as_str() {
        "node" | "nodejs" => Some("node"),
        "python" | "python3" => Some("python"),
        "ruby" => Some("ruby"),
        "go" => Some("go"),
        "java" | "java11" | "java17" | "java21" => Some("java"),
        "elixir" | "iex" => Some("elixir"),
        "erl" => Some("erlang"),
        "gleam" => Some("gleam"),
        _ => None,
    }
}

/// Returns true when the foreground process is a shell, not an app.
///
/// Login shells prefix the name with `-` (e.g. `-zsh`, `-bash`); strip it
/// before matching so stale screen banners are not re-detected after a server exits.
fn is_shell_process(argv: &[String]) -> bool {
    let raw = path_basename(argv.first().map(String::as_str).unwrap_or("")).to_lowercase();
    let name = raw.trim_start_matches('-');
    matches!(
        name,
        "zsh" | "bash" | "sh" | "fish" | "nu" | "dash" | "ksh" | "csh" | "tcsh" | "pwsh"
    )
}

/// Identify a dev server tool from distinctive terminal banner text.
///
/// Rules live in `dev-servers.toml`, not here, so adding a framework is a file
/// edit and a reload rather than a rebuild. See [`crate::detect::dev_server_manifest`].
pub fn detect_tool_from_screen(screen: &str) -> Option<&'static str> {
    crate::detect::dev_server_manifest::detect_tool_from_screen(screen)
}

/// Extract a listening port from recent terminal output.
///
/// Only used when the kernel socket table has nothing for this pane — a server
/// inside a container, on the other end of an SSH session, or otherwise outside
/// any process tree we can read.
pub fn extract_port_from_screen(screen: &str) -> Option<u16> {
    for prefix in &[
        "localhost:",
        "127.0.0.1:",
        "0.0.0.0:",
        "[::1]:",
        "[::]:",
        ":::",
    ] {
        if let Some(port) = first_port_after(screen, prefix) {
            return Some(port);
        }
    }
    if let Some(after) = screen.find("tcp://") {
        // Skip past "tcp://" then find the first ':' to locate the port separator.
        // Using rfind here would pick the wrong port when multiple tcp:// URLs appear.
        let url_tail = &screen[after + 6..];
        if let Some(colon) = url_tail.find(':') {
            if let Some(port) = first_port_after(&url_tail[colon..], ":") {
                return Some(port);
            }
        }
    }
    // Banners that name the port in prose: "Tomcat started on port 8080",
    // "Serving HTTP on :: port 8000".
    if let Some(after) = screen.find(" port ") {
        let digits: String = screen[after + " port ".len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(p) = digits.parse::<u16>() {
            if p > 0 {
                return Some(p);
            }
        }
    }
    // "Listening on: 8080". Slice at the marker's own length: any other offset
    // can land mid-character and panic on multi-byte terminal output.
    if let Some(after) = screen.find(LISTENING_ON_MARKER) {
        let tail = &screen[after + LISTENING_ON_MARKER.len()..];
        let digits: String = tail
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(p) = digits.parse::<u16>() {
            if p > 0 {
                return Some(p);
            }
        }
        if let Some(port) = first_port_after(tail, ":") {
            return Some(port);
        }
    }
    None
}

const LISTENING_ON_MARKER: &str = "Listening on:";

// ---------------------------------------------------------------------------
// Shared socket scan
// ---------------------------------------------------------------------------

/// Floor on the interval between whole-system socket scans.
const MIN_SCAN_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);
/// Ceiling, so a pathologically slow scan still refreshes eventually.
const MAX_SCAN_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
/// The scan is budgeted to roughly this fraction of wall time: a scan measured
/// at 100ms buys a 2s wait, one at 500ms buys 10s. Mirrors the backoff VS Code
/// uses for the same whole-system scan.
const SCAN_DUTY_CYCLE_DIVISOR: u32 = 20;

/// A whole-system listening-socket scan, shared by every pane.
#[derive(Debug)]
pub struct SocketScan {
    pub sockets: Vec<crate::platform::ListeningSocket>,
    /// Distinguishes one scan from the next. Panes poll more often than scans
    /// are taken, so they need to tell a fresh scan from a cached one.
    pub id: u64,
    taken_at: std::time::Instant,
    /// How long this scan took, which sets how long the next one waits.
    cost: std::time::Duration,
}

impl SocketScan {
    fn next_due(&self) -> std::time::Instant {
        let interval =
            (self.cost * SCAN_DUTY_CYCLE_DIVISOR).clamp(MIN_SCAN_INTERVAL, MAX_SCAN_INTERVAL);
        self.taken_at + interval
    }
}

static SOCKET_SCAN: std::sync::Mutex<Option<std::sync::Arc<SocketScan>>> =
    std::sync::Mutex::new(None);
static SCAN_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The current whole-system socket scan, refreshing it when it is due.
///
/// Every pane shares one scan. Scanning per pane would multiply a whole-system
/// walk by the pane count; this keeps the cost proportional to the number of
/// processes on the machine instead, however many panes are open.
pub fn shared_socket_scan() -> std::sync::Arc<SocketScan> {
    let mut guard = SOCKET_SCAN.lock().unwrap_or_else(|err| err.into_inner());
    if let Some(scan) = guard.as_ref() {
        if std::time::Instant::now() < scan.next_due() {
            return std::sync::Arc::clone(scan);
        }
    }

    let started = std::time::Instant::now();
    let sockets = crate::platform::listening_sockets();
    let taken_at = std::time::Instant::now();
    let scan = std::sync::Arc::new(SocketScan {
        sockets,
        id: SCAN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        taken_at,
        cost: taken_at.saturating_duration_since(started),
    });
    *guard = Some(std::sync::Arc::clone(&scan));
    scan
}

/// Consecutive scans without a socket before a latched server is dropped.
/// Absorbs a scan that briefly fails to read the process or socket tables.
const GONE_CONFIRMATIONS: u8 = 3;

/// A listening socket owned by a process inside one pane's process tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSocket {
    pub pid: u32,
    pub port: u16,
    /// Command line of the owning process, when it could be read.
    pub argv: Option<Vec<String>>,
}

/// A single observation of a pane, fed to [`DevServerTracker::poll`].
pub struct DevServerPoll<'a> {
    /// Listening sockets attributed to this pane's process tree.
    pub sockets: &'a [PaneSocket],
    /// Identifies the socket scan these sockets came from. Scans are shared
    /// between panes and cached, so the same scan can be observed by several
    /// polls; a repeated id must not be counted as fresh evidence that a
    /// latched server is gone.
    pub scan_id: u64,
    /// Recent terminal text, consulted only when `sockets` is empty.
    pub screen: &'a str,
    /// Foreground process argv, used to tell a live process from a bare shell
    /// prompt on the screen-fallback path.
    pub foreground_argv: Option<&'a [String]>,
}

#[derive(Debug)]
struct LatchedServer {
    tool: &'static str,
    /// Owning process, or `None` for a server only the screen could see.
    pid: Option<u32>,
    gone_scans: u8,
}

/// Latching dev-server detection for one pane.
///
/// The kernel socket table is the primary signal: a port is attributed to this
/// pane when the process holding it is somewhere in the pane's process tree.
/// That covers task runners which place each child in its own process group,
/// and it reports every server in the pane rather than just one.
///
/// The screen is a fallback for servers no socket scan can attribute —
/// containers, remote hosts — and it can add information but never retract it,
/// because a banner scrolls away long before its server exits.
#[derive(Debug, Default)]
pub struct DevServerTracker {
    /// Latched servers keyed by port.
    current: std::collections::BTreeMap<u16, LatchedServer>,
    last_scan_id: Option<u64>,
}

impl DevServerTracker {
    /// Every server currently latched for this pane, ordered by port.
    pub fn servers(&self) -> Vec<DevServerInfo> {
        self.current
            .iter()
            .map(|(port, server)| DevServerInfo {
                tool: server.tool,
                port: *port,
            })
            .collect()
    }

    pub fn poll(&mut self, poll: DevServerPoll<'_>) -> Vec<DevServerInfo> {
        let scan_is_new = self.last_scan_id != Some(poll.scan_id);
        self.last_scan_id = Some(poll.scan_id);

        for socket in poll.sockets {
            let tool = socket
                .argv
                .as_deref()
                .and_then(detect_tool_from_argv)
                .or_else(|| detect_tool_from_screen(poll.screen))
                .or_else(|| {
                    socket
                        .argv
                        .as_deref()
                        .and_then(detect_generic_runtime_from_argv)
                })
                .unwrap_or(UNKNOWN_TOOL);
            self.current.insert(
                socket.port,
                LatchedServer {
                    tool,
                    pid: Some(socket.pid),
                    gone_scans: 0,
                },
            );
        }

        // Nothing attributable through the socket table. The pane may still be
        // running a server we cannot see into, so read the screen — but only
        // while a real process holds the foreground, so a stale banner is not
        // re-detected after the server exits.
        if poll.sockets.is_empty() && !self.has_socket_backed_server() {
            let foreground_runs_a_process = !poll.foreground_argv.is_some_and(is_shell_process);
            if foreground_runs_a_process {
                if let (Some(tool), Some(port)) = (
                    detect_tool_from_screen(poll.screen),
                    extract_port_from_screen(poll.screen),
                ) {
                    self.current.insert(
                        port,
                        LatchedServer {
                            tool,
                            pid: None,
                            gone_scans: 0,
                        },
                    );
                }
            }
        }

        if scan_is_new {
            self.retract_missing(poll.sockets);
        }

        self.servers()
    }

    /// Age out latched servers the newest scan no longer reports.
    ///
    /// Screen-only servers have no socket to disappear from, so they are held
    /// until the pane's foreground goes quiet, which `poll` handles by simply
    /// not refreshing them.
    fn retract_missing(&mut self, sockets: &[PaneSocket]) {
        self.current.retain(|port, server| {
            if sockets.iter().any(|socket| socket.port == *port) {
                server.gone_scans = 0;
                return true;
            }
            if server.pid.is_none() {
                // Screen-only: retract on the same schedule, since nothing else
                // will ever confirm it.
                server.gone_scans = server.gone_scans.saturating_add(1);
                return server.gone_scans < GONE_CONFIRMATIONS;
            }
            server.gone_scans = server.gone_scans.saturating_add(1);
            server.gone_scans < GONE_CONFIRMATIONS
        });
    }

    fn has_socket_backed_server(&self) -> bool {
        self.current.values().any(|server| server.pid.is_some())
    }
}

/// Fallback name for a listening process we cannot identify. It is still a
/// real server on a real port, so it is worth showing.
const UNKNOWN_TOOL: &str = "server";

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Last path component of an argv entry.
///
/// Handles both separators regardless of host: argv can carry Windows paths
/// (`C:\Program Files\nodejs\node.exe`) as well as POSIX ones.
fn path_basename(s: &str) -> &str {
    s.trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(s)
}

fn first_port_after(text: &str, prefix: &str) -> Option<u16> {
    let mut search = text;
    loop {
        let idx = search.find(prefix)?;
        let after = idx + prefix.len();
        let digits: String = search[after..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(p) = digits.parse::<u16>() {
            if p > 0 {
                return Some(p);
            }
        }
        if after >= search.len() {
            return None;
        }
        search = &search[after..];
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_vite_from_argv() {
        let argv = vec!["node".into(), "/proj/node_modules/.bin/vite".into()];
        assert_eq!(detect_tool_from_argv(&argv), Some("vite"));
    }

    #[test]
    fn detect_nextjs_from_argv() {
        let argv = vec![
            "node".into(),
            "/proj/node_modules/.bin/next".into(),
            "dev".into(),
        ];
        assert_eq!(detect_tool_from_argv(&argv), Some("nextjs"));
    }

    #[test]
    fn detect_nestjs_from_argv() {
        let argv = vec![
            "node".into(),
            "/proj/node_modules/.bin/nest".into(),
            "start".into(),
            "--watch".into(),
        ];
        assert_eq!(detect_tool_from_argv(&argv), Some("nestjs"));
    }

    #[test]
    fn detect_rails_from_argv() {
        let argv = vec!["ruby".into(), "bin/rails".into(), "server".into()];
        assert_eq!(detect_tool_from_argv(&argv), Some("rails"));
    }

    #[test]
    fn detect_django_from_argv() {
        let argv = vec!["python3".into(), "manage.py".into(), "runserver".into()];
        assert_eq!(detect_tool_from_argv(&argv), Some("django"));
    }

    #[test]
    fn detect_laravel_from_argv() {
        let argv = vec!["php".into(), "artisan".into(), "serve".into()];
        assert_eq!(detect_tool_from_argv(&argv), Some("laravel"));
    }

    #[test]
    fn detect_vite_from_screen() {
        let screen = "  VITE v5.2.0  ready in 324 ms\n\n  ➜  Local:   http://localhost:5173/";
        assert_eq!(detect_tool_from_screen(screen), Some("vite"));
    }

    #[test]
    fn detect_sveltekit_preferred_over_vite() {
        let screen = "SvelteKit v2.0.0\nVITE v5.0.0\n  Local: http://localhost:5173/";
        assert_eq!(detect_tool_from_screen(screen), Some("sveltekit"));
    }

    #[test]
    fn detect_nestjs_from_screen_prefix() {
        let screen = "[Nest] 12345  - 01/01/2024, 12:00:00 AM     LOG [NestFactory] Starting Nest application...";
        assert_eq!(detect_tool_from_screen(screen), Some("nestjs"));
    }

    #[test]
    fn extract_port_from_vite_banner() {
        let screen = "  VITE v5.2.0  ready in 324 ms\n\n  ➜  Local:   http://localhost:5173/";
        assert_eq!(extract_port_from_screen(screen), Some(5173));
    }

    #[test]
    fn extract_port_from_spring_boot_banner() {
        let screen = "Tomcat started on port 8080 with context path ''";
        assert_eq!(extract_port_from_screen(screen), Some(8080));
    }

    #[test]
    fn extract_port_from_rails_puma_banner() {
        let screen = "Puma starting...\nListening on tcp://127.0.0.1:3000";
        assert_eq!(extract_port_from_screen(screen), Some(3000));
    }

    #[test]
    fn extract_port_from_tcp_url_picks_first_not_last() {
        // rfind would return 3001; find must return 3000
        let screen = "tcp://0.0.0.0:3000 tcp://0.0.0.0:3001";
        assert_eq!(extract_port_from_screen(screen), Some(3000));
    }

    #[test]
    fn windows_style_argv_paths_resolve_the_tool() {
        // Detection runs on every platform, so a Windows pane has to resolve
        // the tool name from a backslash path.
        let argv: Vec<String> = vec![
            "C:\\Program Files\\nodejs\\node.exe".into(),
            "C:\\proj\\node_modules\\.bin\\vite".into(),
        ];
        assert_eq!(detect_tool_from_argv(&argv), Some("vite"));
    }

    #[test]
    fn a_wrapper_argv_falls_through_to_the_screen_banner() {
        // `npm run dev` says nothing about what it started; the banner does.
        let argv: Vec<String> = vec![
            "node".into(),
            "/usr/lib/node_modules/npm/bin/npm-cli.js".into(),
            "run".into(),
            "dev".into(),
        ];
        assert_eq!(detect_tool_from_argv(&argv), None);

        let screen = " astro  v6.4.5 ready in 1214 ms\n┃ Local    http://localhost:4321/\n";
        let sockets = [PaneSocket {
            pid: 5,
            port: 4321,
            argv: Some(argv),
        }];
        let mut tracker = DevServerTracker::default();
        assert_eq!(
            tracker.poll(DevServerPoll {
                sockets: &sockets,
                scan_id: 1,
                screen,
                foreground_argv: None,
            }),
            vec![DevServerInfo {
                tool: "astro",
                port: 4321
            }]
        );
    }

    #[test]
    fn extract_port_from_ipv6_url() {
        assert_eq!(
            extract_port_from_screen("Listening on http://[::1]:4321/"),
            Some(4321)
        );
    }

    #[test]
    fn detect_http_server_from_screen_alone() {
        assert_eq!(
            detect_tool_from_screen("Serving HTTP on 0.0.0.0 port 8000"),
            Some("http.server")
        );
    }
    // -----------------------------------------------------------------------
    // Tracker
    // -----------------------------------------------------------------------

    fn socket(pid: u32, port: u16, argv: &[&str]) -> PaneSocket {
        PaneSocket {
            pid,
            port,
            argv: Some(argv.iter().map(|a| a.to_string()).collect()),
        }
    }

    /// Poll with a fresh scan id each time, as a live pane would once the
    /// shared scan refreshes.
    fn poll_scan(
        tracker: &mut DevServerTracker,
        scan_id: u64,
        sockets: &[PaneSocket],
    ) -> Vec<DevServerInfo> {
        tracker.poll(DevServerPoll {
            sockets,
            scan_id,
            screen: "",
            foreground_argv: None,
        })
    }

    #[test]
    fn reports_every_server_a_task_runner_started() {
        // turbo places each task in its own process group; attribution is by
        // process tree, so all three are still this pane's.
        let sockets = [
            socket(201, 3000, &["node", "/repo/node_modules/.bin/next", "dev"]),
            socket(202, 3001, &["node", "/repo/node_modules/.bin/astro", "dev"]),
            socket(203, 4000, &["python3", "-m", "uvicorn", "app:api"]),
        ];
        let mut tracker = DevServerTracker::default();

        assert_eq!(
            poll_scan(&mut tracker, 1, &sockets),
            vec![
                DevServerInfo {
                    tool: "nextjs",
                    port: 3000
                },
                DevServerInfo {
                    tool: "astro",
                    port: 3001
                },
                DevServerInfo {
                    tool: "uvicorn",
                    port: 4000
                },
            ]
        );
    }

    #[test]
    fn unrecognised_listener_is_still_reported() {
        let sockets = [socket(9, 7654, &["/opt/custom/bin/thing", "--serve"])];
        let mut tracker = DevServerTracker::default();
        assert_eq!(
            poll_scan(&mut tracker, 1, &sockets),
            vec![DevServerInfo {
                tool: "server",
                port: 7654
            }]
        );
    }

    #[test]
    fn generic_runtime_names_a_plain_script() {
        let sockets = [socket(9, 5000, &["node", "/repo/server.mjs"])];
        let mut tracker = DevServerTracker::default();
        assert_eq!(poll_scan(&mut tracker, 1, &sockets)[0].tool, "node");
    }

    #[test]
    fn one_server_stopping_leaves_the_others() {
        let all = [
            socket(201, 3000, &["node", "/repo/node_modules/.bin/next", "dev"]),
            socket(202, 3001, &["node", "/repo/node_modules/.bin/astro", "dev"]),
        ];
        let remaining = [all[1].clone()];
        let mut tracker = DevServerTracker::default();
        poll_scan(&mut tracker, 1, &all);

        for scan in 2..=GONE_CONFIRMATIONS as u64 {
            let live = poll_scan(&mut tracker, scan, &remaining);
            assert!(live.iter().any(|s| s.port == 3000), "retracted too early");
        }
        let live = poll_scan(&mut tracker, GONE_CONFIRMATIONS as u64 + 1, &remaining);
        assert_eq!(
            live,
            vec![DevServerInfo {
                tool: "astro",
                port: 3001
            }]
        );
    }

    #[test]
    fn a_repeated_scan_is_not_fresh_evidence() {
        let sockets = [socket(
            201,
            3000,
            &["node", "/repo/node_modules/.bin/next", "dev"],
        )];
        let mut tracker = DevServerTracker::default();
        poll_scan(&mut tracker, 1, &sockets);

        // Panes poll faster than the shared scan refreshes, so the same scan is
        // observed repeatedly. That must not age the latch out.
        for _ in 0..10 {
            assert_eq!(poll_scan(&mut tracker, 1, &[]).len(), 1);
        }
    }

    #[test]
    fn a_transient_empty_scan_does_not_retract_immediately() {
        let sockets = [socket(
            201,
            3000,
            &["node", "/repo/node_modules/.bin/next", "dev"],
        )];
        let mut tracker = DevServerTracker::default();
        poll_scan(&mut tracker, 1, &sockets);

        assert_eq!(poll_scan(&mut tracker, 2, &[]).len(), 1);
        // Recovering resets the countdown.
        assert_eq!(poll_scan(&mut tracker, 3, &sockets).len(), 1);
        assert_eq!(poll_scan(&mut tracker, 4, &[]).len(), 1);
        assert_eq!(poll_scan(&mut tracker, 5, &[]).len(), 1);
    }

    #[test]
    fn a_restart_on_a_new_port_replaces_the_old_entry() {
        let before = [socket(
            201,
            3000,
            &["node", "/repo/node_modules/.bin/next", "dev"],
        )];
        let after = [socket(
            444,
            3002,
            &["node", "/repo/node_modules/.bin/next", "dev"],
        )];
        let mut tracker = DevServerTracker::default();
        poll_scan(&mut tracker, 1, &before);

        for scan in 2..=(GONE_CONFIRMATIONS as u64 + 1) {
            poll_scan(&mut tracker, scan, &after);
        }
        assert_eq!(
            tracker.servers(),
            vec![DevServerInfo {
                tool: "nextjs",
                port: 3002
            }]
        );
    }

    // -----------------------------------------------------------------------
    // Screen fallback: containers and remote hosts, where no socket is ours
    // -----------------------------------------------------------------------

    fn poll_screen(
        tracker: &mut DevServerTracker,
        scan_id: u64,
        screen: &str,
        foreground_argv: Option<&[String]>,
    ) -> Vec<DevServerInfo> {
        tracker.poll(DevServerPoll {
            sockets: &[],
            scan_id,
            screen,
            foreground_argv,
        })
    }

    #[test]
    fn screen_fallback_covers_a_server_no_socket_scan_can_see() {
        let argv: Vec<String> = vec!["docker".into(), "compose".into(), "up".into()];
        let screen = " VITE v5.2.0  ready in 324 ms\n  ➜  Local:   http://localhost:5173/";
        let mut tracker = DevServerTracker::default();
        assert_eq!(
            poll_screen(&mut tracker, 1, screen, Some(&argv)),
            vec![DevServerInfo {
                tool: "vite",
                port: 5173
            }]
        );
    }

    #[test]
    fn a_stale_banner_at_a_shell_prompt_is_not_detected() {
        let shell: Vec<String> = vec!["-zsh".into()];
        let screen = " VITE v5.2.0  ready in 324 ms\n  ➜  Local:   http://localhost:5173/";
        let mut tracker = DevServerTracker::default();
        assert!(poll_screen(&mut tracker, 1, screen, Some(&shell)).is_empty());
    }

    #[test]
    fn a_socket_backed_server_is_not_second_guessed_by_the_screen() {
        // The screen still shows an old banner on a different port; the socket
        // table is authoritative once it has attributed something.
        let sockets = [socket(
            201,
            3000,
            &["node", "/repo/node_modules/.bin/next", "dev"],
        )];
        let mut tracker = DevServerTracker::default();
        poll_scan(&mut tracker, 1, &sockets);

        let stale = "  ➜  Local:   http://localhost:5173/";
        let live = tracker.poll(DevServerPoll {
            sockets: &[],
            scan_id: 2,
            screen: stale,
            foreground_argv: None,
        });
        assert_eq!(live.iter().map(|s| s.port).collect::<Vec<_>>(), vec![3000]);
    }

    // -----------------------------------------------------------------------
    // Port extraction
    // -----------------------------------------------------------------------

    #[test]
    fn listening_on_marker_reads_a_bare_port() {
        assert_eq!(extract_port_from_screen("Listening on: 8080"), Some(8080));
    }

    #[test]
    fn port_extraction_survives_multibyte_text_after_the_marker() {
        // Slicing at a fixed offset past the marker used to land mid-character
        // and panic on output like this.
        assert_eq!(extract_port_from_screen("Listening on:➜"), None);
        assert_eq!(extract_port_from_screen("Listening on:"), None);
        assert_eq!(
            extract_port_from_screen("Listening on: ➜ http://localhost:9111/"),
            Some(9111)
        );
    }
}
