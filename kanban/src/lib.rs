pub mod arg;
pub mod gpu;
pub mod long;
pub mod method;
pub mod multiple;
pub mod multiple2;
pub mod raw_gpu;
pub mod raw_single;
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

pub fn kanban_run(cli: &MainArg) {
    match &cli.mode {
        Mode::Single(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::Multiple(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::Multiple2(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::Long(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::Vertical(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::Wave(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::Gpu(arg) => match arg.method {
            Method::Procname => arg.clone().run_by_procname(),
            Method::Compile => arg.clone().run_by_compile(),
            Method::Copy => todo!(),
        },
        Mode::RawSingle(arg) => arg.run(),
        Mode::RawGpu(arg) => arg.run(),
    }
}
