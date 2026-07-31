// The page talks to a kanban running on the same machine.
//
// Two ways to get here. Served by `kanban serve`, everything is same-origin and
// just works. Loaded from GitHub Pages, the page has to reach across to
// 127.0.0.1, which needs the token the server prints and, in Chrome, the user's
// Local Network Access permission - so that path degrades to instructions
// rather than pretending to work.
//
// The preview always comes from the server, never from a copy of the layout
// logic reimplemented here. A second implementation drifts, and the whole point
// of the shared core crate was to stop that happening.

const LOCAL = "http://127.0.0.1:8787";

const el = (id) => document.getElementById(id);
const form = el("form");
const rows = el("rows");

// Which numeric options each mode actually reads. Showing controls a mode
// ignores is a way of lying about what the run will do.
const USES = {
  single:    { thread: true,  time: true,  length: false },
  multiple:  { thread: true,  time: true,  length: false },
  multiple2: { thread: false, time: true,  length: false },
  long:      { thread: false, time: true,  length: true  },
  vertical:  { thread: false, time: true,  length: false },
  wave:      { thread: true,  time: false, length: true  },
};

const HINTS = {
  single:    "One message on one row.",
  multiple:  "The same message on every row.",
  multiple2: "One row per word.",
  long:      "The message wrapped into rows of WIDTH characters.",
  vertical:  "Words turned on their side, reading downwards.",
  wave:      "The message scrolls through a WIDTH-wide window, one frame at a time.",
};

let link = { up: false, base: "", token: "", rustc: false, sameOrigin: false };
let busy = false;

// --- reading the form ------------------------------------------------------

function spec() {
  const mode = form.querySelector('input[name="mode"]:checked').value;
  const raw = el("message").value;
  const words = raw.split(/\s+/).filter(Boolean);
  return {
    mode,
    // multiple2 and vertical are the plural modes; the rest take one string.
    message: mode === "multiple2" || mode === "vertical" ? words : [raw],
    thread: Number(el("thread").value) || 1,
    time: Number(el("time").value) || 10,
    length: Number(el("length").value) || 12,
  };
}

function commandLine(s) {
  const q = (v) => (/^[\w.@%+=:,/-]+$/.test(v) ? v : `"${v.replace(/(["\\$`])/g, "\\$1")}"`);
  const parts = ["kanban", s.mode, "-m"];
  parts.push(s.message.map(q).join(" "));
  const uses = USES[s.mode];
  if (uses.thread) parts.push("-@", String(s.thread));
  if (uses.time) parts.push("-t", String(s.time));
  if (uses.length) parts.push("-l", String(s.length));
  return parts.join(" ");
}

// --- the fake top screen ---------------------------------------------------

function render(lines) {
  rows.replaceChildren();

  if (!lines || lines.length === 0) {
    const tr = document.createElement("tr");
    tr.className = "empty";
    const td = document.createElement("td");
    td.colSpan = 5;
    td.textContent = "Type a message and it will show up here.";
    tr.append(td);
    rows.append(tr);
    return;
  }

  const basePid = 1000 + Math.floor(Math.random() * 8000);
  const user = "you";

  lines.forEach((line, i) => {
    const tr = document.createElement("tr");
    if (i === 0) tr.className = "hot";
    // top sorts by CPU, which is why kanban gives earlier rows more threads.
    const cpu = (99.9 - i * 0.3 - Math.random() * 0.2).toFixed(1);
    for (const [text, cls] of [
      [String(basePid + i), "r"],
      [user, ""],
      [cpu, "r"],
      [(0.1 + i * 0.01).toFixed(1), "r"],
      [line === "" ? "\u00a0" : line, "w"],
    ]) {
      const td = document.createElement("td");
      td.className = cls;
      td.textContent = text;
      tr.append(td);
    }
    rows.append(tr);
  });
}

// --- server ---------------------------------------------------------------

async function post(path, body) {
  const headers = { "Content-Type": "application/json" };
  if (!link.sameOrigin && link.token) headers["X-Kanban-Token"] = link.token;
  const res = await fetch(link.base + path, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || `server said ${res.status}`);
  return data;
}

async function probe() {
  // Same origin first: if this page came from the server, it is right there.
  try {
    const res = await fetch("/health");
    if (res.ok) {
      const data = await res.json();
      link = { up: true, base: "", token: "", rustc: !!data.rustc, sameOrigin: true };
      return setLink(`connected, kanban v${data.version}`);
    }
  } catch (_) { /* not served locally; try across */ }

  const token = new URLSearchParams(location.hash.slice(1)).get("t") || "";
  try {
    const res = await fetch(LOCAL + "/health", { targetAddressSpace: "local" });
    if (res.ok) {
      const data = await res.json();
      link = { up: true, base: LOCAL, token, rustc: !!data.rustc, sameOrigin: false };
      return setLink(token
        ? `connected to 127.0.0.1:8787, kanban v${data.version}`
        : "found 127.0.0.1:8787, but no token in this link");
    }
  } catch (_) { /* nothing listening, or the browser blocked it */ }

  link = { up: false, base: "", token: "", rustc: false, sameOrigin: false };
  setLink("no kanban listening on this machine");
}

function setLink(text) {
  const status = el("link-status");
  status.dataset.state = link.up ? "up" : "down";
  el("link-text").textContent = text;

  const run = el("run");
  const canRun = link.up && (link.sameOrigin || !!link.token);
  run.disabled = !canRun;
  el("offline").hidden = canRun;

  el("watch").textContent = !canRun
    ? ""
    : link.rustc
      ? "Watch with top. Each row is its own process."
      : "No rustc here, so rows are threads: watch with top -H or htop.";

  el("caption").textContent = link.up
    ? "Preview from the local kanban. PID, %CPU and %MEM are made up; COMMAND is real."
    : "Start a local kanban to preview and run.";
}

// --- wiring ---------------------------------------------------------------

let previewTimer;

function refresh() {
  const s = spec();

  for (const key of ["thread", "time", "length"]) {
    document.querySelector(`.num[data-for="${key}"]`).hidden = !USES[s.mode][key];
  }
  el("mode-hint").textContent = HINTS[s.mode];
  el("cli").textContent = commandLine(s);

  clearTimeout(previewTimer);
  previewTimer = setTimeout(async () => {
    if (!link.up) return render([]);
    try {
      const data = await post("/preview", s);
      render(data.lines);
    } catch (e) {
      render([]);
      say(String(e.message || e), "error");
    }
  }, 120);
}

function say(text, state) {
  const out = el("runstate");
  out.textContent = text;
  if (state) out.dataset.state = state; else delete out.dataset.state;
}

form.addEventListener("input", refresh);
form.addEventListener("change", refresh);

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (busy || !link.up) return;

  const s = spec();
  busy = true;
  el("run").disabled = true;

  // wave has no -t: it runs two seconds per frame, so say how long that is.
  const seconds = s.mode === "wave" ? null : s.time;

  try {
    const data = await post("/run", s);
    render(data.lines);
    say(seconds ? `running for ${seconds}s — go look at top` : "running — go look at top", "running");
    const wait = (seconds ?? data.lines.length * 2) * 1000;
    setTimeout(() => { say("done"); busy = false; el("run").disabled = false; }, wait);
  } catch (e) {
    say(String(e.message || e), "error");
    busy = false;
    el("run").disabled = false;
  }
});

for (const [button, source] of [["copy", "oneliner"], ["copy-cli", "cli"]]) {
  el(button).addEventListener("click", async () => {
    const text = el(source).textContent.trim();
    try {
      await navigator.clipboard.writeText(text);
      el(button).textContent = "copied";
      setTimeout(() => { el(button).textContent = "copy"; }, 1200);
    } catch (_) {
      el(button).textContent = "select it manually";
    }
  });
}

fetch("/health")
  .then((r) => (r.ok ? r.json() : null))
  .then((d) => { if (d) el("version").textContent = d.version; })
  .catch(() => { el("version").textContent = "—"; });

render([]);
probe().then(refresh);
