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
/// Ordered most-specific first to avoid misattribution when one tool wraps another.
pub fn detect_tool_from_screen(screen: &str) -> Option<&'static str> {
    if screen.contains("VITE v") {
        // SvelteKit wraps Vite and shows its own banner; prefer it.
        if screen.contains("SvelteKit v") {
            return Some("sveltekit");
        }
        return Some("vite");
    }
    if screen.contains("▲ Next.js") || screen.contains("◼ Next.js") {
        return Some("nextjs");
    }
    if contains_nest_log_prefix(screen) || screen.contains("Nest application successfully started")
    {
        return Some("nestjs");
    }
    if screen.contains("[webpack-dev-server]") {
        return Some("webpack");
    }
    if (screen.contains("\u{25B2} Nuxt") || screen.contains("Nuxt ")) && screen.contains("ready") {
        return Some("nuxt");
    }
    if screen.contains("SvelteKit v") {
        return Some("sveltekit");
    }
    if screen.contains("Remix App Server") || (screen.contains("remix") && screen.contains("built"))
    {
        return Some("remix");
    }
    if screen.contains(" astro  v") || (screen.contains("astro") && screen.contains("ready in")) {
        return Some("astro");
    }
    if screen.contains("Gatsby develop")
        || (screen.contains("gatsby") && screen.contains("You can now view"))
    {
        return Some("gatsby");
    }
    if screen.contains("Storybook") && (screen.contains("started") || screen.contains("ready")) {
        return Some("storybook");
    }
    if screen.contains("Angular Live Development Server") {
        return Some("angular");
    }
    if screen.contains("Metro") && screen.contains("Expo") {
        return Some("expo");
    }
    if screen.contains("Hugo") && screen.contains("Web Server is available at") {
        return Some("hugo");
    }
    if screen.contains("Server address:")
        && screen.contains("127.0.0.1")
        && screen.contains("Server running")
    {
        return Some("jekyll");
    }
    if screen.contains("[11ty]") && (screen.contains("Watching") || screen.contains("Serving")) {
        return Some("eleventy");
    }
    if screen.contains("Starting up http-server") {
        return Some("http-server");
    }
    if screen.contains("Serving HTTP on") {
        return Some("http.server");
    }
    if screen.contains("Zola") && screen.contains("Listening for changes") {
        return Some("zola");
    }
    if screen.contains("mdBook") && screen.contains("Serving on") {
        return Some("mdbook");
    }
    if screen.contains("Uvicorn running on") {
        return Some("uvicorn");
    }
    if screen.contains("Starting development server at") {
        return Some("django");
    }
    if screen.contains("Werkzeug")
        || (screen.contains("Running on http") && screen.contains("Press CTRL+C"))
    {
        return Some("flask");
    }
    if screen.contains("LITESTAR") || (screen.contains("litestar") && screen.contains("Listening"))
    {
        return Some("litestar");
    }
    if screen.contains("Listening at:") && screen.contains("workers") {
        return Some("gunicorn");
    }
    if screen.contains("Puma ") && (screen.contains("started") || screen.contains("Listening")) {
        return Some("rails");
    }
    if screen.contains("Sinatra") && screen.contains("has taken the stage") {
        return Some("sinatra");
    }
    if screen.contains("PHP ") && screen.contains("Development Server") {
        return Some("php");
    }
    if screen.contains("INFO  Application ready!") {
        return Some("laravel");
    }
    if screen.contains("Symfony") && screen.contains("Local Web Server") {
        return Some("symfony");
    }
    if screen.contains("Tomcat started on port")
        || (screen.contains("Started ")
            && screen.contains("in ")
            && screen.contains("seconds (process running for"))
    {
        return Some("spring");
    }
    if screen.contains("Quarkus") && screen.contains("started in") {
        return Some("quarkus");
    }
    if screen.contains("Micronaut")
        && (screen.contains("Startup completed") || screen.contains("startup completed"))
    {
        return Some("micronaut");
    }
    if screen.contains("Application - Application started")
        || (screen.contains("ktor") && screen.contains("Responding at"))
    {
        return Some("ktor");
    }
    if screen.contains("Running ") && screen.contains("Endpoint") && screen.contains("with cowboy")
    {
        return Some("phoenix");
    }
    if screen.contains("Now listening on: http") && screen.contains("Press Ctrl+C to shut down.") {
        return Some("dotnet");
    }
    if screen.contains("Trunk version")
        || (screen.contains("✔ success") && screen.contains("trunk"))
    {
        return Some("trunk");
    }
    if screen.contains("watching .")
        && screen.contains("building...")
        && screen.contains("running...")
    {
        return Some("air");
    }
    if screen.contains("shadow-cljs")
        && (screen.contains("Build completed") || screen.contains("waiting for changes"))
    {
        return Some("shadow-cljs");
    }
    if screen.contains("bun run") || screen.contains("$ bun ") {
        return Some("bun");
    }
    if screen.contains("Listening on http") && screen.contains("deno") {
        return Some("deno");
    }
    // Rust web frameworks
    if screen.contains("[GIN-debug]") {
        return Some("gin");
    }
    if screen.contains("⇨ http server started") {
        return Some("echo");
    }
    if screen.contains("Rocket has launched") {
        return Some("rocket");
    }
    if screen.contains("Actix Web v") {
        return Some("actix-web");
    }
    if screen.contains("Fiber v") && screen.contains("Listen") {
        return Some("fiber");
    }
    if screen.contains("warp::server") {
        return Some("warp");
    }
    if screen.contains("axum: listening on") {
        return Some("axum");
    }
    // Gleam
    if screen.contains("gleam") && screen.contains("Listening on") {
        return Some("gleam");
    }
    // Databases
    if screen.contains("database system is ready to accept connections") {
        return Some("postgres");
    }
    if screen.contains("ready for connections")
        && (screen.contains("MySQL") || screen.contains("MariaDB"))
    {
        return Some("mysql");
    }
    if screen.contains("Waiting for connections") && screen.contains("mongod") {
        return Some("mongodb");
    }
    if screen.contains("Ready to accept connections") && screen.contains("Redis") {
        return Some("redis");
    }
    if screen.contains("CockroachDB node starting") || screen.contains("CockroachDB node ready") {
        return Some("cockroachdb");
    }
    // DevOps / monitoring
    if screen.contains("Server is ready to receive web requests.") {
        return Some("prometheus");
    }
    if (screen.contains("Grafana") || screen.contains("grafana"))
        && screen.contains("HTTP Server Listen")
    {
        return Some("grafana");
    }
    if screen.contains("Consul agent running!") {
        return Some("consul");
    }
    if screen.contains("Vault server started!")
        || (screen.contains("Vault") && screen.contains("api_address"))
    {
        return Some("vault");
    }
    if screen.contains("ready to serve client requests") && screen.contains("etcd") {
        return Some("etcd");
    }
    if screen.contains("Started Zipkin") {
        return Some("zipkin");
    }
    if screen.contains("MinIO Object Storage Server") {
        return Some("minio");
    }
    if screen.contains("Jaeger")
        && (screen.contains("all components ready") || screen.contains("Starting Jaeger"))
    {
        return Some("jaeger");
    }

    None
}

/// Extract a listening port from recent terminal output.
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
        let digits: String = screen[after + 6..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(p) = digits.parse::<u16>() {
            if p > 0 {
                return Some(p);
            }
        }
    }
    if let Some(after) = screen.find("Listening on:") {
        if let Some(port2) = first_port_after(&screen[after + 14..], ":") {
            return Some(port2);
        }
    }
    None
}

/// Combine argv-based and screen-based tool detection.
///
/// Priority: argv-specific > screen-specific > argv-generic (e.g. "node").
fn detect_tool(argv: Option<&[String]>, screen: &str) -> Option<&'static str> {
    match argv {
        Some(a) => detect_tool_from_argv(a)
            .or_else(|| detect_tool_from_screen(screen))
            .or_else(|| detect_generic_runtime_from_argv(a)),
        None => detect_tool_from_screen(screen),
    }
}

/// Consecutive polls without a live foreground process before a latched server
/// is dropped. Absorbs transient failures to read the process table.
const GONE_CONFIRMATIONS: u8 = 3;

/// A single observation of a pane, fed to [`DevServerTracker::poll`].
pub struct DevServerPoll<'a> {
    /// Foreground process group of the pane, if it could be read.
    pub pgid: Option<u32>,
    pub argv: Option<&'a [String]>,
    pub screen: &'a str,
    pub listening_ports: &'a [u16],
}

/// Latching dev-server detection.
///
/// A one-shot detection has to re-derive everything from the visible screen on
/// every poll, so a server drops out the moment its banner scrolls past the
/// text window or the process table read hiccups. This keeps a detected server
/// until the foreground process group actually changes or goes away: the screen
/// can add information but never retracts it.
#[derive(Debug, Default)]
pub struct DevServerTracker {
    current: Option<DevServerInfo>,
    /// Foreground process group the latched server was detected on.
    pgid: Option<u32>,
    gone_polls: u8,
}

impl DevServerTracker {
    /// Whether an OS port lookup is still worth performing.
    ///
    /// Once a server is latched its port is known, and re-reading kernel socket
    /// state every second per pane buys nothing.
    pub fn needs_port_lookup(&self) -> bool {
        self.current.is_none()
    }

    pub fn poll(&mut self, poll: DevServerPoll<'_>) -> Option<DevServerInfo> {
        let foreground_runs_a_process =
            poll.pgid.is_some() && !poll.argv.is_some_and(is_shell_process);

        let (tool, port) = if foreground_runs_a_process {
            (
                detect_tool(poll.argv, poll.screen),
                poll.listening_ports
                    .first()
                    .copied()
                    .or_else(|| extract_port_from_screen(poll.screen)),
            )
        } else {
            (None, None)
        };

        // A complete detection on another process group is a restart: adopt it
        // straight away rather than waiting out the old one.
        if let (Some(tool), Some(port)) = (tool, port) {
            if self.pgid != poll.pgid || self.current.is_none() {
                self.pgid = poll.pgid;
                self.gone_polls = 0;
                self.current = Some(DevServerInfo { tool, port });
                return self.current.clone();
            }
        }

        // Anything else on a foreign or absent process group is evidence the
        // latched server ended, but a single poll can lie: the process table
        // read can fail, and a shell can briefly own the terminal. Only a run
        // of them retracts a detection.
        if !foreground_runs_a_process || self.pgid != poll.pgid {
            self.gone_polls = self.gone_polls.saturating_add(1);
            if self.gone_polls >= GONE_CONFIRMATIONS {
                self.clear();
            }
            return self.current.clone();
        }
        self.gone_polls = 0;

        // Same process group: the screen can sharpen what we know but never
        // retract it, since the banner scrolls away long before the server exits.
        if let Some(current) = &mut self.current {
            if let Some(tool) = tool {
                current.tool = tool;
            }
            if let Some(port) = port {
                current.port = port;
            }
        }

        self.current.clone()
    }

    fn clear(&mut self) {
        self.current = None;
        self.pgid = None;
        self.gone_polls = 0;
    }
}

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

fn contains_nest_log_prefix(screen: &str) -> bool {
    let mut rest = screen;
    while let Some(idx) = rest.find("[Nest]") {
        let after = idx + 6;
        let tail = rest[after..].trim_start();
        if tail.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
        if after >= rest.len() {
            break;
        }
        rest = &rest[after..];
    }
    false
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
    fn detect_reads_windows_style_argv_paths() {
        // Detection runs on every platform, and only Linux reports listening
        // ports, so a Windows pane has to resolve both the tool name from a
        // backslash path and the port from screen output.
        let argv = vec![
            "C:\\Program Files\\nodejs\\node.exe".into(),
            "C:\\proj\\node_modules\\.bin\\vite".into(),
        ];
        let screen = "VITE v5.0  ready\n  ➜  Local: http://localhost:5173/";
        assert_eq!(
            detect_once(Some(&argv), screen, &[]),
            Some(DevServerInfo {
                tool: "vite",
                port: 5173
            })
        );
    }

    #[test]
    fn detect_prefers_os_port() {
        let argv = vec!["node".into(), "/proj/.bin/vite".into()];
        let screen = "VITE v5.0  ready\n  ➜  Local: http://localhost:5173/";
        assert_eq!(
            detect_once(Some(&argv), screen, &[4321]),
            Some(DevServerInfo {
                tool: "vite",
                port: 4321
            })
        );
    }

    #[test]
    fn detect_returns_none_without_port() {
        let argv = vec!["node".into(), "/proj/.bin/vite".into()];
        assert_eq!(detect_once(Some(&argv), "VITE v5 ready", &[]), None);
    }

    #[test]
    fn login_shell_clears_detection() {
        let shell_argv = vec!["-zsh".into()];
        let screen = "  VITE v6.4.3  ready in 164 ms\n  ➜  Local: http://localhost:5173/";
        assert_eq!(detect_once(Some(&shell_argv), screen, &[5173]), None);
    }

    #[test]
    fn bash_login_shell_clears_detection() {
        let shell_argv = vec!["-bash".into()];
        let screen = "Nest application successfully started";
        assert_eq!(detect_once(Some(&shell_argv), screen, &[3000]), None);
    }

    #[test]
    fn node_script_detected_as_node() {
        let argv = vec![
            "/usr/local/bin/node".into(),
            "/home/user/project/server.mjs".into(),
        ];
        let screen = "example server listening on http://localhost:3030\n";
        assert_eq!(
            detect_once(Some(&argv), screen, &[]),
            Some(DevServerInfo {
                tool: "node",
                port: 3030
            })
        );
    }

    #[test]
    fn detect_python_http_server_from_argv_and_banner() {
        let argv: Vec<String> = vec!["python3".into(), "-m".into(), "http.server".into()];
        let screen = "Serving HTTP on :: port 8757 (http://[::]:8757/) ...";
        assert_eq!(
            detect_once(Some(&argv), screen, &[]),
            Some(DevServerInfo {
                tool: "http.server",
                port: 8757,
            })
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

    /// One poll of a fresh tracker, for tests about a single observation.
    fn detect_once(
        argv: Option<&[String]>,
        screen: &str,
        listening_ports: &[u16],
    ) -> Option<DevServerInfo> {
        DevServerTracker::default().poll(DevServerPoll {
            pgid: Some(1),
            argv,
            screen,
            listening_ports,
        })
    }

    fn poll(
        tracker: &mut DevServerTracker,
        pgid: Option<u32>,
        argv: Option<&[String]>,
        screen: &str,
    ) -> Option<DevServerInfo> {
        tracker.poll(DevServerPoll {
            pgid,
            argv,
            screen,
            listening_ports: &[],
        })
    }

    #[test]
    fn tracker_keeps_server_after_banner_scrolls_away() {
        let argv: Vec<String> = vec!["python3".into(), "-m".into(), "http.server".into()];
        let mut tracker = DevServerTracker::default();

        let banner = "Serving HTTP on :: port 8757 (http://[::]:8757/) ...";
        assert_eq!(
            poll(&mut tracker, Some(42), Some(&argv), banner),
            Some(DevServerInfo {
                tool: "http.server",
                port: 8757,
            })
        );

        // Request logs have pushed the banner out of the text window.
        for _ in 0..10 {
            assert_eq!(
                poll(&mut tracker, Some(42), Some(&argv), "GET / HTTP/1.1 200 -"),
                Some(DevServerInfo {
                    tool: "http.server",
                    port: 8757,
                })
            );
        }
    }

    #[test]
    fn tracker_survives_transient_process_read_failure() {
        let argv: Vec<String> = vec!["python3".into(), "-m".into(), "http.server".into()];
        let banner = "Serving HTTP on :: port 8757 (http://[::]:8757/) ...";
        let mut tracker = DevServerTracker::default();
        poll(&mut tracker, Some(42), Some(&argv), banner);

        assert!(poll(&mut tracker, None, None, "").is_some());
        assert!(poll(&mut tracker, None, None, "").is_some());
        // Recovering resets the countdown.
        assert!(poll(&mut tracker, Some(42), Some(&argv), "").is_some());
        assert!(poll(&mut tracker, None, None, "").is_some());
    }

    #[test]
    fn tracker_drops_server_after_sustained_shell_foreground() {
        let argv: Vec<String> = vec!["python3".into(), "-m".into(), "http.server".into()];
        let shell: Vec<String> = vec!["-zsh".into()];
        let banner = "Serving HTTP on :: port 8757 (http://[::]:8757/) ...";
        let mut tracker = DevServerTracker::default();
        poll(&mut tracker, Some(42), Some(&argv), banner);

        assert!(poll(&mut tracker, Some(7), Some(&shell), banner).is_some());
        assert!(poll(&mut tracker, Some(7), Some(&shell), banner).is_some());
        assert_eq!(poll(&mut tracker, Some(7), Some(&shell), banner), None);
    }

    #[test]
    fn tracker_drops_server_when_foreground_group_changes() {
        let argv: Vec<String> = vec!["python3".into(), "-m".into(), "http.server".into()];
        let other: Vec<String> = vec!["vim".into()];
        let banner = "Serving HTTP on :: port 8757 (http://[::]:8757/) ...";
        let mut tracker = DevServerTracker::default();
        poll(&mut tracker, Some(42), Some(&argv), banner);

        // Another process owns the pane and the banner has scrolled away.
        assert!(poll(&mut tracker, Some(99), Some(&other), "~ ~ ~").is_some());
        assert!(poll(&mut tracker, Some(99), Some(&other), "~ ~ ~").is_some());
        assert_eq!(poll(&mut tracker, Some(99), Some(&other), "~ ~ ~"), None);
    }

    #[test]
    fn tracker_updates_port_on_restart() {
        let argv: Vec<String> = vec!["python3".into(), "-m".into(), "http.server".into()];
        let mut tracker = DevServerTracker::default();
        poll(
            &mut tracker,
            Some(42),
            Some(&argv),
            "Serving HTTP on :: port 8000 (http://[::]:8000/) ...",
        );
        let restarted = poll(
            &mut tracker,
            Some(43),
            Some(&argv),
            "Serving HTTP on :: port 9000 (http://[::]:9000/) ...",
        );
        assert_eq!(restarted.map(|s| s.port), Some(9000));
    }

    #[test]
    fn npm_wrapper_defers_to_screen_banner() {
        let argv = vec![
            "node".into(),
            "/usr/lib/node_modules/npm/bin/npm-cli.js".into(),
            "run".into(),
            "dev".into(),
        ];
        let screen = " astro  v6.4.5 ready in 1214 ms\n┃ Local    http://localhost:4321/\n";
        assert_eq!(
            detect_once(Some(&argv), screen, &[]),
            Some(DevServerInfo {
                tool: "astro",
                port: 4321
            })
        );
    }
}
