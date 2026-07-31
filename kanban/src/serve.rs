//! A tiny web UI that runs kanban on the machine you opened it from.
//!
//! A browser cannot name a process, so it cannot do what kanban does. What it
//! can do is ask a local program to. `kanban serve` binds 127.0.0.1, serves the
//! UI from the binary itself, and runs the modes in-process on request.
//!
//! Serving the page from the same origin the requests go to is what keeps this
//! simple: no CORS, no mixed content, and none of Chrome's Local Network Access
//! prompting. A page hosted elsewhere - GitHub Pages, say - can still reach the
//! server, but only cross-origin with the token printed at startup.

use crate::arg::*;
use crate::method::procname::*;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tiny_http::{Header, Method, Request, Response, Server};

const INDEX_HTML: &str = include_str!("../../web/index.html");
const APP_JS: &str = include_str!("../../web/app.js");
const STYLE_CSS: &str = include_str!("../../web/style.css");

/// Longest message we will accept, in bytes.
///
/// Not an arbitrary cap: `--method compile` names the binary after the message,
/// and Linux limits one path component to NAME_MAX bytes. Past that, rustc
/// fails with ENAMETOOLONG - which used to leave the run flag stuck on, so
/// every later run answered 409 forever.
const MAX_MESSAGE_LEN: usize = 255;

/// Longest run we will accept, in seconds.
const MAX_TIME: usize = 300;

/// Most processes or threads a single request may spawn.
const MAX_THREAD: usize = 64;

pub fn run(arg: &ServeArg) {
    let token = gen_token();
    let addr = format!("127.0.0.1:{}", arg.port);

    let server = match Server::http(&addr) {
        Ok(server) => server,
        Err(e) => {
            log::error!("failed to bind {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    let has_rustc = which_rustc();
    log::info!("kanban is serving at http://{}/", addr);
    if !has_rustc {
        log::info!("rustc not found: only --method procname is offered");
    }
    log::info!("cross-origin token: {}", token);

    // One run at a time. Every mode saturates the CPU on purpose, so overlapping
    // requests would just make each other's output unreadable.
    let busy = Arc::new(AtomicBool::new(false));

    for request in server.incoming_requests() {
        let busy = Arc::clone(&busy);
        let token = token.clone();
        if let Err(e) = handle(request, &token, arg.port, has_rustc, busy) {
            log::warn!("request failed: {}", e);
        }
    }
}

fn handle(
    mut request: Request,
    token: &str,
    port: u16,
    has_rustc: bool,
    busy: Arc<AtomicBool>,
) -> std::io::Result<()> {
    let url = request.url().split('?').next().unwrap_or("/").to_string();
    let method = request.method().clone();

    // Preflight for the cross-origin case. Chrome additionally requires the
    // user to grant Local Network Access before it even gets here.
    if method == Method::Options {
        return request.respond(cors(Response::empty(204)));
    }

    match (&method, url.as_str()) {
        (Method::Get, "/") => request.respond(html(INDEX_HTML)),
        (Method::Get, "/app.js") => request.respond(asset(APP_JS, "text/javascript")),
        (Method::Get, "/style.css") => request.respond(asset(STYLE_CSS, "text/css")),
        (Method::Get, "/health") => request.respond(cors(json(&format!(
            "{{\"ok\":true,\"rustc\":{},\"version\":\"{}\"}}",
            has_rustc,
            env!("CARGO_PKG_VERSION")
        )))),
        (Method::Post, "/preview") | (Method::Post, "/run") => {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body)?;

            if !authorised(&request, token, port) {
                return request.respond(
                    cors(json("{\"error\":\"bad or missing token\"}")).with_status_code(403),
                );
            }

            let spec = match RunSpec::parse(&body) {
                Ok(spec) => spec,
                Err(e) => {
                    return request.respond(
                        cors(json(&format!("{{\"error\":{}}}", quote(&e)))).with_status_code(400),
                    )
                }
            };

            let lines = spec.preview();
            if url == "/preview" {
                return request.respond(cors(json(&lines_json(&lines))));
            }

            if busy.swap(true, Ordering::SeqCst) {
                return request
                    .respond(cors(json("{\"error\":\"already running\"}")).with_status_code(409));
            }
            let result = request.respond(cors(json(&lines_json(&lines))));
            // Answer, then work on a thread of its own. Running inline blocks
            // the accept loop for the whole duration, so /health and /preview
            // stop answering and the page looks frozen.
            std::thread::spawn(move || {
                // A panic in here unwinds only this thread, so the server lives
                // on - but it would skip the reset below and wedge every later
                // run at 409. Catch it and report instead.
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    spec.execute(has_rustc)
                }));
                if outcome.is_err() {
                    log::error!("run failed; the server is still up");
                }
                busy.store(false, Ordering::SeqCst);
            });
            result
        }
        _ => request.respond(Response::from_string("not found").with_status_code(404)),
    }
}

/// Same-origin requests carry no token; the browser's origin policy already
/// gates them. Anything that announces a different origin must present one.
fn authorised(request: &Request, token: &str, port: u16) -> bool {
    match header(request, "Origin") {
        None => true,
        Some(origin) if is_own_origin(&origin, port) => true,
        Some(_) => header(request, "X-Kanban-Token").as_deref() == Some(token),
    }
}

/// Whether an Origin header names this very server.
///
/// Exact comparison, never a prefix: `starts_with("http://127.0.0.1")` also
/// accepts `http://127.0.0.1.example.com`, a domain anyone can register, which
/// would have waved that origin through as local.
fn is_own_origin(origin: &str, port: u16) -> bool {
    let mut expected = vec![
        format!("http://127.0.0.1:{}", port),
        format!("http://localhost:{}", port),
        format!("http://[::1]:{}", port),
    ];
    if port == 80 {
        // Browsers leave the port out when it is the scheme's default.
        expected.extend([
            "http://127.0.0.1".to_string(),
            "http://localhost".to_string(),
            "http://[::1]".to_string(),
        ]);
    }
    expected.iter().any(|e| e == origin)
}

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .iter()
        // equiv() wants a 'static name; compare the text instead.
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

/// What a request may ask for. Only these fields, all validated, and never a
/// command line: the message ends up as a filename, so a stray `/` or `..`
/// would write outside the temporary directory.
struct RunSpec {
    mode: String,
    message: Vec<String>,
    thread: usize,
    time: usize,
    length: usize,
}

impl RunSpec {
    fn parse(body: &str) -> Result<Self, String> {
        let mode = field(body, "mode").ok_or("missing mode")?;
        let message: Vec<String> = list(body, "message");

        if message.is_empty() || message.iter().all(|m| m.is_empty()) {
            return Err("message is empty".into());
        }
        for m in &message {
            check_message(m)?;
        }

        Ok(RunSpec {
            mode,
            message,
            thread: number(body, "thread", 1).clamp(1, MAX_THREAD),
            time: number(body, "time", 10).clamp(1, MAX_TIME),
            length: number(body, "length", 12).clamp(1, MAX_MESSAGE_LEN),
        })
    }

    /// What the request would put on screen, without running anything.
    fn preview(&self) -> Vec<String> {
        let first = self.message.first().cloned().unwrap_or_default();
        match self.mode.as_str() {
            "multiple" => kanban_core::multiple(&first, self.thread),
            "multiple2" => kanban_core::multiple2(&self.message),
            "long" => kanban_core::long(&first, self.length),
            "vertical" => kanban_core::vertical(&self.message),
            "wave" => kanban_core::wave(&first, self.length),
            _ => kanban_core::single(&first),
        }
    }

    fn execute(&self, has_rustc: bool) {
        // procname unless the machine can compile: it needs no rustc, writes
        // nothing to disk, and runs no subprocesses.
        let method = if has_rustc {
            crate::method::Method::Compile
        } else {
            crate::method::Method::Procname
        };
        // dir_name is normally filled in by MainArg::default from the parsed
        // command line; there is none here, so generate one directly.
        let common = CommonArgs {
            tmpdir: String::new(),
            method,
            dir_name: temp_dir_name(),
        };

        let first = self.message.first().cloned().unwrap_or_default();
        let mode = match self.mode.as_str() {
            "multiple" => Mode::Multiple(MultipleArg {
                message: first,
                thread: self.thread,
                time: self.time,
                common,
            }),
            "multiple2" => Mode::Multiple2(Multiple2Arg {
                message: self.message.clone(),
                time: self.time,
                common,
            }),
            "long" => Mode::Long(LongArg {
                message: first,
                time: self.time,
                length: self.length,
                common,
            }),
            "vertical" => Mode::Vertical(VerticalArg {
                message: self.message.clone(),
                time: self.time,
                common,
            }),
            "wave" => Mode::Wave(WaveArg {
                message: first,
                thread: self.thread,
                length: self.length,
                common,
            }),
            _ => Mode::Single(SingleArg {
                message: first,
                thread: self.thread,
                time: self.time,
                common,
            }),
        };

        crate::kanban_run(&MainArg { mode });
    }
}

/// Reject anything that would escape the temporary directory once it becomes a
/// filename, plus control characters, which have no business in a process name.
fn check_message(message: &str) -> Result<(), String> {
    if message.len() > MAX_MESSAGE_LEN {
        return Err(format!("message longer than {} bytes", MAX_MESSAGE_LEN));
    }
    if message.contains('/') || message.contains('\\') {
        return Err("message may not contain a path separator".into());
    }
    if message.split_whitespace().any(|w| w == "..") || message == ".." || message == "." {
        return Err("message may not be a path component".into());
    }
    if message.chars().any(|c| c.is_control()) {
        return Err("message may not contain control characters".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Minimal JSON handling. The bodies are three known keys of known shape, so a
// serde dependency would outweigh what it saves.
// ---------------------------------------------------------------------------

fn field(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let colon = rest.find(':')? + 1;
    let rest = rest[colon..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    unquote(&rest[1..])
}

fn list(body: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{}\"", key);
    let Some(start) = body.find(&needle) else {
        return Vec::new();
    };
    let rest = &body[start + needle.len()..];
    let Some(colon) = rest.find(':') else {
        return Vec::new();
    };
    let rest = rest[colon + 1..].trim_start();

    if let Some(inner) = rest.strip_prefix('[') {
        let Some(end) = inner.find(']') else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut chunk = &inner[..end];
        while let Some(open) = chunk.find('"') {
            let Some(value) = unquote(&chunk[open + 1..]) else {
                break;
            };
            let consumed = open + 1 + raw_len(&chunk[open + 1..]);
            out.push(value);
            chunk = &chunk[consumed..];
        }
        return out;
    }

    match field(body, key) {
        Some(single) => vec![single],
        None => Vec::new(),
    }
}

fn number(body: &str, key: &str, fallback: usize) -> usize {
    let needle = format!("\"{}\"", key);
    let Some(start) = body.find(&needle) else {
        return fallback;
    };
    let rest = &body[start + needle.len()..];
    let Some(colon) = rest.find(':') else {
        return fallback;
    };
    rest[colon + 1..]
        .trim_start()
        .trim_start_matches('"')
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(fallback)
}

/// Read a JSON string body up to its closing quote, honouring backslash escapes.
fn unquote(rest: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}

/// Bytes the raw form of the next JSON string occupies, closing quote included.
fn raw_len(rest: &str) -> usize {
    let mut len = 0;
    let mut escaped = false;
    for c in rest.chars() {
        len += c.len_utf8();
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            break;
        }
    }
    len
}

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn lines_json(lines: &[String]) -> String {
    let items: Vec<String> = lines.iter().map(|l| quote(l)).collect();
    format!("{{\"lines\":[{}]}}", items.join(","))
}

// ---------------------------------------------------------------------------

type Bytes = Response<Cursor<Vec<u8>>>;

fn asset(body: &str, mime: &str) -> Bytes {
    Response::from_string(body).with_header(content_type(mime))
}

fn html(body: &str) -> Bytes {
    asset(body, "text/html; charset=utf-8")
}

fn json(body: &str) -> Bytes {
    asset(body, "application/json")
}

fn content_type(mime: &str) -> Header {
    Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).expect("static header")
}

/// Allow the published copy of this same UI to talk to a local server. Exact
/// origin only, and requests from it still need the token.
fn cors<R: std::io::Read>(response: Response<R>) -> Response<R> {
    let allow = [
        ("Access-Control-Allow-Origin", "https://th2ch-g.github.io"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        (
            "Access-Control-Allow-Headers",
            "Content-Type, X-Kanban-Token",
        ),
        // Chrome's Local Network Access preflight.
        ("Access-Control-Allow-Private-Network", "true"),
    ];
    allow.iter().fold(response, |r, (k, v)| {
        match Header::from_bytes(k.as_bytes(), v.as_bytes()) {
            Ok(header) => r.with_header(header),
            Err(_) => r,
        }
    })
}

/// A token a page from another origin must present to run anything.
///
/// This gates execution, so it comes from the OS-seeded generator rather than
/// anything derived from the clock and the pid - those are both observable, and
/// a token an attacker can predict is no token at all. It is not a secret from
/// someone who can already read this process's memory; it stops a web page you
/// happened to open from driving the server.
fn gen_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    format!("{:016x}{:016x}", rng.random::<u64>(), rng.random::<u64>())
}

fn temp_dir_name() -> String {
    use rand::Rng;
    format!(
        "{}_{}_{}",
        chrono::Utc::now().format("/tmp/tmp_kanban_%Y%m%d%H%M%S"),
        rand::rng().random::<u32>(),
        std::process::id()
    )
}

fn which_rustc() -> bool {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Thread names are what `procname` shows, and the kernel keeps only 15 bytes
/// of them. The UI needs to say so, so expose the same cut it will apply.
pub fn preview_thread_name(name: &str) -> String {
    fit_thread_name(name)
}
