//! Banner-text rules for naming dev servers, loaded from TOML.
//!
//! This is the fallback namer. A server whose listening socket is attributable
//! to a pane's process tree is named from the owning process argv, which needs
//! no rules; these patterns cover servers no socket scan can reach — inside a
//! container, on the far side of an SSH session.
//!
//! Rules are data rather than code so a new framework is a file edit and a
//! reload, not a rebuild. The bundled table can be replaced wholesale by an
//! override at `~/.config/herdr/dev-servers.toml`.

use std::sync::{OnceLock, RwLock};

use regex::Regex;
use serde::Deserialize;
use tracing::warn;

/// Bundled rules, compiled into the binary so detection works with no config.
const BUNDLED: &str = include_str!("dev-servers.toml");

/// Highest schema version this build understands.
const SUPPORTED_VERSION: u32 = 1;

/// Caps on an override file, so a malformed or hostile one cannot make
/// detection pathologically slow. The bundled table is well inside these.
const MAX_SERVERS: usize = 512;
const MAX_GATE_DEPTH: usize = 8;
const MAX_MATCHER_CHARS: usize = 512;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestFile {
    version: u32,
    #[serde(default)]
    servers: Vec<ServerRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerRule {
    id: String,
    #[serde(default)]
    priority: i32,
    #[serde(flatten)]
    gate: Gate,
}

/// A boolean expression over the screen text.
///
/// The flat fields are conjunctive — every `contains` string and every `regex`
/// must match. `any` / `all` / `not` nest for what that cannot express.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Gate {
    #[serde(default)]
    contains: Vec<String>,
    #[serde(default)]
    regex: Vec<String>,
    #[serde(default)]
    all: Vec<Gate>,
    #[serde(default)]
    any: Vec<Gate>,
    #[serde(default, rename = "not")]
    not_gate: Vec<Gate>,
}

#[derive(Debug)]
struct CompiledRule {
    /// Leaked so matches can hand back a `&'static str`, matching the argv
    /// namers. Rules live for the process, and a reload replaces the table
    /// rather than freeing individual ids.
    id: &'static str,
    priority: i32,
    gate: CompiledGate,
}

#[derive(Debug, Default)]
struct CompiledGate {
    contains: Vec<String>,
    regex: Vec<Regex>,
    all: Vec<CompiledGate>,
    any: Vec<CompiledGate>,
    not_gate: Vec<CompiledGate>,
}

impl CompiledGate {
    fn matches(&self, screen: &str) -> bool {
        self.contains.iter().all(|needle| screen.contains(needle))
            && self.regex.iter().all(|re| re.is_match(screen))
            && self.all.iter().all(|gate| gate.matches(screen))
            && (self.any.is_empty() || self.any.iter().any(|gate| gate.matches(screen)))
            && !self.not_gate.iter().any(|gate| gate.matches(screen))
    }

    /// True when nothing can fail, which would match every screen.
    fn is_vacuous(&self) -> bool {
        self.contains.is_empty()
            && self.regex.is_empty()
            && self.all.is_empty()
            && self.any.is_empty()
            && self.not_gate.is_empty()
    }
}

fn compile_gate(gate: &Gate, depth: usize, rule_id: &str) -> Option<CompiledGate> {
    if depth > MAX_GATE_DEPTH {
        warn!(rule = rule_id, "dev server rule nests too deeply; skipping");
        return None;
    }
    if gate
        .contains
        .iter()
        .chain(gate.regex.iter())
        .any(|matcher| matcher.chars().count() > MAX_MATCHER_CHARS)
    {
        warn!(rule = rule_id, "dev server matcher too long; skipping rule");
        return None;
    }

    let mut regex = Vec::with_capacity(gate.regex.len());
    for pattern in &gate.regex {
        match Regex::new(pattern) {
            Ok(compiled) => regex.push(compiled),
            Err(err) => {
                warn!(rule = rule_id, %err, "invalid dev server regex; skipping rule");
                return None;
            }
        }
    }

    let compile_all = |gates: &[Gate]| -> Option<Vec<CompiledGate>> {
        gates
            .iter()
            .map(|nested| compile_gate(nested, depth + 1, rule_id))
            .collect()
    };

    Some(CompiledGate {
        contains: gate.contains.clone(),
        regex,
        all: compile_all(&gate.all)?,
        any: compile_all(&gate.any)?,
        not_gate: compile_all(&gate.not_gate)?,
    })
}

fn compile(content: &str, origin: &str) -> Result<Vec<CompiledRule>, String> {
    let parsed: ManifestFile = toml::from_str(content).map_err(|err| format!("{origin}: {err}"))?;
    if parsed.version > SUPPORTED_VERSION {
        return Err(format!(
            "{origin}: schema version {} is newer than this build supports ({SUPPORTED_VERSION})",
            parsed.version
        ));
    }
    if parsed.servers.len() > MAX_SERVERS {
        return Err(format!("{origin}: more than {MAX_SERVERS} rules"));
    }

    let mut rules = Vec::with_capacity(parsed.servers.len());
    for rule in &parsed.servers {
        let Some(gate) = compile_gate(&rule.gate, 0, &rule.id) else {
            continue;
        };
        if gate.is_vacuous() {
            warn!(
                rule = rule.id,
                "dev server rule has no matchers and would match anything; skipping"
            );
            continue;
        }
        rules.push(CompiledRule {
            id: Box::leak(rule.id.clone().into_boxed_str()),
            priority: rule.priority,
            gate,
        });
    }

    // Most specific first, so a wrapper outranks the tool it wraps. Ties break
    // on id purely to keep matching deterministic across reloads.
    rules.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(b.id)));
    Ok(rules)
}

fn override_path() -> std::path::PathBuf {
    crate::config::config_dir().join("dev-servers.toml")
}

fn build_rules() -> Vec<CompiledRule> {
    let path = override_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        match compile(&content, &path.display().to_string()) {
            Ok(rules) => return rules,
            Err(err) => warn!(%err, "dev server override is invalid; using bundled rules"),
        }
    }
    // A broken bundled table is a build error, not a runtime condition, but
    // detection must not take the process down if one ever ships.
    compile(BUNDLED, "bundled dev-servers.toml").unwrap_or_else(|err| {
        warn!(%err, "bundled dev server rules failed to compile");
        Vec::new()
    })
}

static RULES: OnceLock<RwLock<Vec<CompiledRule>>> = OnceLock::new();

fn rules() -> &'static RwLock<Vec<CompiledRule>> {
    RULES.get_or_init(|| RwLock::new(build_rules()))
}

/// Re-read the rules, picking up edits to the override file.
///
/// Shares the agent manifests' reload command, so one reload refreshes both.
pub fn reload() {
    let rebuilt = build_rules();
    let lock = rules();
    match lock.write() {
        Ok(mut guard) => *guard = rebuilt,
        Err(poisoned) => *poisoned.into_inner() = rebuilt,
    }
}

/// Identify a dev server from terminal banner text.
pub fn detect_tool_from_screen(screen: &str) -> Option<&'static str> {
    if screen.is_empty() {
        return None;
    }
    let guard = match rules().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard
        .iter()
        .find(|rule| rule.gate.matches(screen))
        .map(|rule| rule.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compiled(toml: &str) -> Vec<CompiledRule> {
        compile(toml, "test").expect("compile")
    }

    #[test]
    fn bundled_rules_compile() {
        let rules = compile(BUNDLED, "bundled").expect("bundled rules must compile");
        assert!(
            rules.len() > 40,
            "expected the full table, got {}",
            rules.len()
        );
    }

    #[test]
    fn bundled_rule_ids_are_unique() {
        let rules = compile(BUNDLED, "bundled").expect("compile");
        let mut ids: Vec<_> = rules.iter().map(|rule| rule.id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate rule id in bundled table");
    }

    #[test]
    fn contains_fields_are_conjunctive() {
        let rules = compiled(
            r#"
version = 1
[[servers]]
id = "both"
contains = ["alpha", "beta"]
"#,
        );
        assert!(rules[0].gate.matches("alpha and beta"));
        assert!(!rules[0].gate.matches("alpha only"));
    }

    #[test]
    fn any_gate_is_disjunctive_and_not_gate_excludes() {
        let rules = compiled(
            r#"
version = 1
[[servers]]
id = "either"
contains = ["base"]
any = [{ contains = ["alpha"] }, { contains = ["beta"] }]
not = [{ contains = ["halt"] }]
"#,
        );
        assert!(rules[0].gate.matches("base alpha"));
        assert!(rules[0].gate.matches("base beta"));
        assert!(!rules[0].gate.matches("base nothing else"));
        assert!(!rules[0].gate.matches("base alpha halt"));
    }

    #[test]
    fn higher_priority_wins_over_a_tool_it_wraps() {
        // SvelteKit runs on Vite and both banners are on screen at once.
        let screen = "SvelteKit v2.0.0\n  VITE v5.2.0  ready in 324 ms";
        assert_eq!(detect_tool_from_screen(screen), Some("sveltekit"));
        assert_eq!(
            detect_tool_from_screen("  VITE v5.2.0  ready"),
            Some("vite")
        );
    }

    #[test]
    fn a_vacuous_rule_is_dropped_rather_than_matching_everything() {
        let rules = compiled(
            r#"
version = 1
[[servers]]
id = "empty"
"#,
        );
        assert!(rules.is_empty());
    }

    #[test]
    fn an_invalid_regex_drops_only_its_own_rule() {
        let rules = compiled(
            r#"
version = 1
[[servers]]
id = "broken"
regex = ["("]
[[servers]]
id = "fine"
contains = ["ok"]
"#,
        );
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "fine");
    }

    #[test]
    fn a_future_schema_version_is_refused_rather_than_half_read() {
        let err = compile("version = 99", "test").expect_err("must refuse");
        assert!(err.contains("newer than this build supports"), "{err}");
    }

    #[test]
    fn unknown_fields_are_refused_so_typos_are_not_silently_ignored() {
        assert!(compile(
            r#"
version = 1
[[servers]]
id = "x"
containss = ["typo"]
"#,
            "test"
        )
        .is_err());
    }

    #[test]
    fn nesting_beyond_the_depth_cap_drops_the_rule() {
        let mut nested = String::from("{ contains = [\"deep\"] }");
        for _ in 0..MAX_GATE_DEPTH + 2 {
            nested = format!("{{ all = [{nested}] }}");
        }
        let rules = compiled(&format!(
            "version = 1\n[[servers]]\nid = \"deep\"\nall = [{nested}]\n"
        ));
        assert!(rules.is_empty());
    }

    /// Every id the bundled table can produce, so the panel never shows a name
    /// that only exists in a rule file.
    #[test]
    fn every_bundled_rule_matches_a_representative_banner() {
        let cases: &[(&str, &str)] = &[
            ("vite", "  VITE v5.2.0  ready in 324 ms"),
            ("sveltekit", "SvelteKit v2.0.0 ready"),
            ("nextjs", "  ▲ Next.js 14.1.0"),
            ("nestjs", "[Nest] 51234  - 08/20/2026"),
            ("webpack", "[webpack-dev-server] Project is running at:"),
            ("nuxt", "▲ Nuxt 3.10.0 ready in 812 ms"),
            ("remix", "Remix App Server started at http://localhost:3000"),
            ("astro", " astro  v4.5.0 ready in 1214 ms"),
            ("gatsby", "Gatsby develop process started"),
            ("storybook", "Storybook 8.0.0 for react-vite started"),
            (
                "angular",
                "** Angular Live Development Server is listening **",
            ),
            ("expo", "Starting Metro Bundler for Expo"),
            ("bun", "$ bun run dev"),
            ("deno", "Listening on http://localhost:8000/ deno"),
            (
                "hugo",
                "Hugo Web Server is available at http://localhost:1313/",
            ),
            (
                "jekyll",
                "Server address: http://127.0.0.1:4000/\nServer running... press ctrl-c to stop.",
            ),
            ("eleventy", "[11ty] Watching for file changes"),
            ("zola", "Zola: Listening for changes in /site"),
            ("mdbook", "mdBook Serving on: http://localhost:3000"),
            ("http-server", "Starting up http-server, serving ./"),
            ("http.server", "Serving HTTP on 0.0.0.0 port 8000"),
            (
                "uvicorn",
                "INFO:     Uvicorn running on http://127.0.0.1:8000",
            ),
            (
                "django",
                "Starting development server at http://127.0.0.1:8000/",
            ),
            ("flask", "WARNING: This is a development server. Werkzeug"),
            ("litestar", "LITESTAR starting"),
            (
                "gunicorn",
                "[INFO] Listening at: http://0.0.0.0:8000 (4 workers)",
            ),
            (
                "rails",
                "Puma starting in single mode\n* Listening on http://0.0.0.0:3000",
            ),
            ("sinatra", "Sinatra has taken the stage on 4567"),
            (
                "php",
                "PHP 8.3.0 Development Server (http://localhost:8000) started",
            ),
            ("laravel", "  INFO  Application ready!"),
            ("symfony", "[OK] Symfony Local Web Server listening"),
            ("spring", "Tomcat started on port 8080 (http)"),
            ("quarkus", "Quarkus 3.8.0 started in 1.234s"),
            ("micronaut", "Micronaut (v4.3.0) Startup completed in 812ms"),
            ("ktor", "Application - Application started in 0.31 seconds."),
            ("phoenix", "Running MyAppWeb.Endpoint with cowboy 2.10.0"),
            (
                "dotnet",
                "Now listening on: http://localhost:5000\nPress Ctrl+C to shut down.",
            ),
            ("trunk", "Trunk version 0.19.0"),
            ("air", "watching .\nbuilding...\nrunning..."),
            ("shadow-cljs", "shadow-cljs - Build completed."),
            ("gleam", "gleam Listening on http://localhost:3000"),
            ("gin", "[GIN-debug] Listening and serving HTTP on :8080"),
            ("echo", "⇨ http server started on [::]:1323"),
            ("fiber", "Fiber v2.52.0  Listen on :3000"),
            ("rocket", "Rocket has launched from http://127.0.0.1:8000"),
            ("actix-web", "Actix Web v4.5.1 starting"),
            ("warp", "warp::server: listening on 127.0.0.1:3030"),
            ("axum", "axum: listening on 0.0.0.0:3000"),
            (
                "postgres",
                "LOG:  database system is ready to accept connections",
            ),
            ("mysql", "MySQL: ready for connections. Version 8.0"),
            ("mongodb", "mongod Waiting for connections on port 27017"),
            ("redis", "Redis: Ready to accept connections tcp"),
            ("cockroachdb", "CockroachDB node starting at 2026-08-20"),
            ("prometheus", "Server is ready to receive web requests."),
            ("grafana", "Grafana HTTP Server Listen address=[::]:3000"),
            ("consul", "Consul agent running!"),
            (
                "vault",
                "Vault server started! Log data will stream in below",
            ),
            ("etcd", "etcd ready to serve client requests"),
            ("zipkin", "Started Zipkin in 3.2 seconds"),
            ("minio", "MinIO Object Storage Server"),
            ("jaeger", "Starting Jaeger all-in-one"),
        ];

        for (expected, banner) in cases {
            assert_eq!(
                detect_tool_from_screen(banner),
                Some(*expected),
                "banner for {expected} did not resolve to it: {banner:?}"
            );
        }

        // Every rule in the table is covered by a case above.
        let rules = compile(BUNDLED, "bundled").expect("compile");
        let covered: std::collections::HashSet<&str> = cases.iter().map(|(id, _)| *id).collect();
        let uncovered: Vec<&str> = rules
            .iter()
            .map(|rule| rule.id)
            .filter(|id| !covered.contains(id))
            .collect();
        assert!(
            uncovered.is_empty(),
            "rules without a test banner: {uncovered:?}"
        );
    }
}
