pub mod arg;
pub mod gpu;
pub mod long;
pub mod method;
pub mod multiple;
pub mod multiple2;
pub mod raw_gpu;
pub mod raw_single;
pub mod single;
pub mod vertical;
pub mod wave;
use crate::arg::*;
use crate::method::{compile::*, procname::*, *};

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
