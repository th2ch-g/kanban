pub mod arg;
pub mod gpu;
pub mod long;
pub mod method;
pub mod multiple;
pub mod multiple2;
pub mod raw_gpu;
pub mod raw_single;
pub mod serve;
pub mod single;
#[cfg(test)]
mod tests;
pub mod vertical;
pub mod wave;
use crate::arg::*;
use crate::method::{compile::*, procname::*, *};

/// The templates are written to a temporary directory and compiled at runtime,
/// so nothing in a normal build ever type-checks them - which is how the GPU
/// one silently kept a wgpu 0.18 call through a bump to 30. Pulling them in as
/// modules here makes `cargo check` and `cargo clippy` cover them.
///
/// This only works because the templates are ordinary Rust now; they used to
/// open with `const TIME: u64 = { time };`, filled in by string replacement.
#[allow(dead_code)]
mod template_typecheck {
    mod ms {
        include!("template/ms.rs");
    }
    mod gpu {
        include!("template/gpu/main.rs");
    }
}

/// Hand a mode to the execution method it asked for.
///
/// Every mode picks between the same three, so this used to be the same
/// three-arm match written out once per mode.
fn dispatch<T>(arg: &T)
where
    T: CompileTopMessage + ProcnameTopMessage + Clone,
{
    match arg.method() {
        Method::Compile => arg.clone().run_by_compile(),
        Method::Procname => arg.clone().run_by_procname(),
        // Reachable: --method copy is offered by the CLI but was never
        // implemented. Say so rather than panicking through todo!().
        Method::Copy => {
            log::error!("--method copy is not implemented yet");
            std::process::exit(1);
        }
    }
}

pub fn kanban_run(cli: &MainArg) {
    match &cli.mode {
        Mode::Single(arg) => dispatch(arg),
        Mode::Multiple(arg) => dispatch(arg),
        Mode::Multiple2(arg) => dispatch(arg),
        Mode::Long(arg) => dispatch(arg),
        Mode::Vertical(arg) => dispatch(arg),
        Mode::Wave(arg) => dispatch(arg),
        Mode::Gpu(arg) => dispatch(arg),
        Mode::RawSingle(arg) => arg.run(),
        Mode::RawGpu(arg) => arg.run(),
        Mode::Serve(arg) => serve::run(arg),
    }
}
