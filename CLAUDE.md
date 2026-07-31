# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```shell
# Build (workspace default-members = ["kanban"], so the other members are skipped)
cargo build -r --bin kanban

# Run
cargo run -r --bin kanban <mode> -m "message" -t 3

# Unit tests: kanban-core's layout functions, plus each mode's messages()
cargo test -p kanban -p kanban-core

# Everything the workspace can type-check, kanban-mpi included. Needs OpenMPI,
# and mpi-sys runs bindgen, so clang has to find a stddef.h:
BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include" cargo check --workspace

# Format (CI runs this on main and auto-commits "[Action] cargo fmt")
cargo fmt

# Lint (CI runs it from ./kanban)
cargo clippy


# MPI variant
cargo build -r -p kanban-mpi
```

`.github/workflows/test.yml` is a list of one-line smoke tests; run one directly, e.g.

```shell
cargo run -r --bin kanban single    -m "aaa"             -@ 2 -t 3 --tmpdir ./single
cargo run -r --bin kanban multiple2 -m "aaa" "bbb" "ccc"      -t 3 --tmpdir ./multiple2
cargo run -r --bin kanban long      -m "aaabbb"          -l 3 -t 3 --tmpdir ./long
```

Passing an explicit `--tmpdir` keeps the smoke run out of `/tmp` and makes leftovers visible.

**Exit code 0 is a weak signal.** Useful checks the smoke tests do not make:

- `--help` for every mode, diffed against a capture from before the change. Nothing
  else notices a renamed flag, a changed default or a reordered help.
- Elapsed time. Every mode should finish in about its `-t` seconds; if the parameters
  stop reaching the generated binary it falls back to 10.
- `/proc/<pid>/task` during a run, to see the thread counts a mode actually produced.
- The GPU paths need a real GPU. `srun -p maxq -w fdn21 --gres=gpu:1`, then
  `raw-gpu` first (in-process, no template build), and check `nvidia-smi` shows load
  and that the logged adapter is a `DiscreteGpu` - wgpu will quietly settle for a
  llvmpipe software rasterizer that never appears in nvtop.

## Architecture

The tool's whole trick: **the name shown by `top(1)` is what we control.** Every mode reduces to
producing a list of strings, and each string becomes one process (or thread) name.

**Per-mode logic lives in `kanban-core`**, a dependency-free crate with one function per mode
(`single` / `multiple` / `multiple2` / `long` / `vertical` / `wave`). Everything downstream is shared machinery that does not care which mode produced the list.

- `long` — the message wrapped into rows of `-l` characters
- `vertical` — transposed columns, so words read top-to-bottom in `top`
- `wave` — frames of the message scrolling through an `-l`-wide window, run one at a time
- `multiple` — N identical copies

Everything works in `char`s, not bytes. The byte-oriented originals mangled multi-byte text three
different ways, and `wave` panicked outright.

### Workspace members

| crate | what it is |
|---|---|
| `kanban-core` | the layout functions, no dependencies, no I/O |
| `kanban` | the CLI and the execution methods |
| `kanban-mpi` | MPI wrapper around `kanban_run` |

`default-members = ["kanban"]`, so a plain `cargo build` skips the rest. That is how `kanban-mpi`
came to not compile at all for a while.

### Two execution methods (`--method`, `src/method/`)

- **`compile`** (default) — writes `template/ms.rs` into a temp dir, runs `rustc -o <dir>/run/<message>`,
  then runs it. The *binary filename* is the message. Requires `rustc` on PATH at runtime.
  `Gpu` mode instead shells out to `cargo build`.
- **`procname`** — spawns `std::thread::Builder::new().name(message)` threads. The *thread name* is
  the message, visible in `htop` / `top -H`, not plain `top`.
- **`copy`** — the enum variant exists but nothing implements it; selecting it reports that and exits.

Both burn CPU deliberately so the fake processes rank high enough in `top` to be visible. The GPU
load is a WGSL compute shader re-dispatched until the deadline.

### The templates are ordinary Rust

`template/ms.rs` and `template/gpu/main.rs` are compiled at runtime in a temp dir, but they are
valid source files and `lib.rs` pulls them in under `mod template_typecheck` so `cargo check` and
clippy cover them. Parameters arrive through `KANBAN_THREAD` and `KANBAN_TIME` rather than string
substitution — that is what makes them valid Rust, and environment variables stay out of the
process name that `top` displays. Only `template/gpu/Cargo.toml` is still patched, replacing the
`kanban_gpu_template` binary name with the message.

### Trait layering (`src/method/mod.rs`)

`CommonTopMessage` (`messages`/`dir_name`/`method`/`thread`/`time`) is the data adapter each mode
implements. `CompileTopMessage` and `ProcnameTopMessage` build on it with full default
implementations — which is why `impl ProcnameTopMessage for XArg {}` is usually the entire procname
support for a mode.

`template_run` takes a `ThreadPlan` and an `ExecutionOrder`:

- `ThreadPlan::One` — one thread per process (`multiple`, `multiple2`)
- `ThreadPlan::Decreasing` — process `i` of `n` gets `n - i` threads (`long`, `vertical`).
  **Deliberate**: `top` sorts by CPU, so this keeps multi-row messages in reading order.
- `ThreadPlan::Uniform(n)` — every process the same (`wave`)
- `ExecutionOrder::Sequential` is what makes `wave` animate

### Adding a mode

1. `arg.rs` — new `XArg` with `#[clap(flatten)] pub common: CommonArgs`, a `Mode::X(XArg)` variant,
   and an arm in `Mode::common_mut` (the match is exhaustive, so the compiler will ask)
2. `src/x.rs` — `impl CommonTopMessage for XArg`, then `impl CompileTopMessage` and `impl ProcnameTopMessage`
3. `lib.rs` — `pub mod x;` and `Mode::X(arg) => dispatch(arg)`
4. `kanban-core` — the layout function, plus tests

`raw-single` / `raw-gpu` bypass all of the above: no rename, no temp dir, run in-process.

## Gotchas

- **`--method procname` names threads, not processes**, so plain `top` will not show them.
  Linux also truncates a thread name to 15 bytes; `fit_thread_name` cuts on a character boundary
  first, because the kernel does not and leaves invalid UTF-8 behind.
- **`rmdir` only deletes a directory containing `kanban.idfile`.** That marker guards against a
  user-supplied `--tmpdir` pointing at real work. Cleanup is a `Drop` guard, so it survives panics.
- **`MainArg::default()` calls `clap::Parser::parse()`** — a `Default` impl that reads argv and may
  exit the process. Never construct it in a non-CLI context.
- **`wave` has no `-t`.** Each frame lasts `SECONDS_PER_FRAME` (2s), so a run takes frames × 2.
- **`gen_dir_name` compares `--tmpdir` against `DEFAULT_TMPDIR`** to decide whether the user passed
  one. Change the constant and the default together; they are the same `const` now, but the
  comparison is still exact equality.
- Version/edition/deps are declared once in the root `Cargo.toml` `[workspace.*]` tables; member
  manifests use `.workspace = true`.
