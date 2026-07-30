# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```shell
# Build (workspace default-members = ["kanban"], so kanban-mpi is skipped)
cargo build -r --bin kanban

# Run
cargo run -r --bin kanban <mode> -m "message" -t 3

# Format (CI runs this on main and auto-commits "[Action] cargo fmt")
cargo fmt

# Lint (CI runs it from ./kanban)
cargo clippy

# MPI variant: needs openmpi(4.0.3) or intel-mpi(19.0) in the environment
cargo build -r -p kanban-mpi
```

There is **no unit test suite** — `cargo test` compiles and verifies nothing. Verification is
`cargo build -r --bin kanban` plus a short smoke run. `.github/workflows/test.yml` is a list of
one-line smoke tests; run a single one directly, e.g.

```shell
cargo run -r --bin kanban single    -m "aaa"             -@ 2 -t 3 --tmpdir ./single
cargo run -r --bin kanban multiple2 -m "aaa" "bbb" "ccc"      -t 3 --tmpdir ./multiple2
cargo run -r --bin kanban long      -m "aaabbb"          -l 3 -t 3 --tmpdir ./long
```

Passing an explicit `--tmpdir` keeps the smoke run out of `/tmp` and makes leftovers visible.

## Architecture

The tool's whole trick: **the name shown by `top(1)` is what we control.** Every mode reduces to
producing a list of strings, and each string becomes one process (or thread) name.

**Per-mode logic lives entirely in `messages() -> Vec<String>`.** That is the only interesting code
in `single.rs` / `multiple.rs` / `multiple2.rs` / `long.rs` / `vertical.rs` / `wave.rs`:

- `long` — byte chunks of the message (`-l` chars per row)
- `vertical` — transposed columns, so words read top-to-bottom in `top`
- `wave` — rotated shifts of the message, executed sequentially for a marquee effect
- `multiple` — N identical copies

Everything downstream is shared machinery.

### Two execution methods (`--method`, `src/method/`)

- **`compile`** (default) — writes `template/ms.rs` into a temp dir, runs `rustc -o <dir>/run/<message>`,
  then `cd`s in and executes `./<message>`. The *binary filename* is the message. Requires `rustc`
  on PATH at runtime. `Gpu` mode instead shells out to `cargo build` (pulls wgpu from git — slow).
- **`procname`** — spawns `std::thread::Builder::new().name(message)` threads. The *thread name* is
  the message, visible in `htop` / `top -H`. No temp dir, no `rustc`.
- **`copy`** — enum variant exists but every arm in `kanban_run` is `todo!()`; selecting it panics.

Both methods burn CPU with a deliberate busy loop (`template/ms.rs` spins until `elapsed >= TIME`)
so the fake processes rank high enough in `top` to be visible. GPU load is an *infinite-loop* WGSL
compute shader (`template/gpu/shader.wgsl`) re-dispatched until the deadline.

### Trait layering (`src/method/mod.rs`)

`CommonTopMessage` (`messages`/`dir_name`/`thread`/`time`) is the data adapter each mode implements.
`CompileTopMessage` and `ProcnameTopMessage` build on it and carry full default implementations —
which is why `impl ProcnameTopMessage for XArg {}` is usually the entire procname support for a mode.

### Adding a mode

1. `arg.rs` — new `XArg` struct (`#[clap(skip)] pub dir_name: String` + a `Mode::X(XArg)` variant,
   and add it to the `MainArg::default()` match that fills `dir_name`)
2. `src/x.rs` — `impl CommonTopMessage for XArg`, then `impl CompileTopMessage` and `impl ProcnameTopMessage`
3. `lib.rs` — `pub mod x;` and a 3-arm `Method` match in `kanban_run`

`raw-single` / `raw-gpu` bypass all of the above: no rename, no temp dir, run in-process.

## Gotchas

- **`gen_dir_name` (`arg.rs:31`) matches a literal sentinel string.** The default
  `"/tmp/tmp_kanban_(date_randomnumber_pid)"` is duplicated across 7 Arg structs and compared by
  exact equality. Edit that default in one struct and randomization silently stops for that mode.
- **`MainArg::default()` calls `clap::Parser::parse()`** — a `Default` impl that reads argv and may
  exit the process. Never construct it in a non-CLI context.
- **`rmdir()` only deletes if `kanban.idfile` exists** in the target dir. That marker is the guard
  against a user-supplied `--tmpdir` pointing at a real directory. Keep it.
- **The compile path mutates process-global cwd** via `std::env::set_current_dir`, records the old
  one first, and restores it after. Anything added there must not run concurrently with other cwd users.
- **`wave` ignores `-t`** — `time()` is hardcoded to 2 seconds per frame (`wave.rs:79`).
- **Stale exploratory comments, not design notes**: `method/procname.rs` (most of the fn body),
  `wave.rs:49-79`, `long.rs:28-29` contain leftover deliberation written as questions to self.
  Do not treat them as specification.
- **`kanban-mpi/src/main.rs` calls `kanban_run` twice** (line 13 on all ranks, line 19 again on root).
  Looks unintentional.
- Version/edition/deps are declared once in the root `Cargo.toml` `[workspace.*]` tables; member
  manifests use `.workspace = true`. `wgpu` is pinned to git tag `v0.18.0`.
