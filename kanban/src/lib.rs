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
use crate::method::compile::*;

pub fn kanban_run(cli: &MainArg) {
    match &cli.mode {
        Mode::Single(arg) => arg.clone().run(),
        Mode::Multiple(arg) => arg.clone().run(),
        Mode::Multiple2(arg) => arg.clone().run(),
        Mode::Vertical(arg) => arg.clone().run(),
        Mode::Long(arg) => arg.clone().run(),
        Mode::Wave(arg) => arg.clone().run(),
        Mode::Gpu(arg) => arg.clone().run(),
        Mode::RawSingle(arg) => arg.run(),
        Mode::RawGpu(arg) => arg.run(),
    }
}
