use crate::method::*;
use clap::{Parser, Subcommand};
use rand::prelude::*;

/// Placeholder meaning "the user did not pass --tmpdir".
///
/// clap cannot tell a default apart from an identical explicit value, so
/// `gen_dir_name` compares against this exact string. It used to be written out
/// in eight places, and editing any one of them would have silently stopped
/// randomising the directory for that mode alone.
const DEFAULT_TMPDIR: &str = "/tmp/tmp_kanban_(date_randomnumber_pid)";

#[derive(Parser, Debug, Clone)]
#[clap(version, about)] //#[clap(author, version, about)]
pub struct MainArg {
    #[clap(subcommand)]
    pub mode: Mode,
}

impl Default for MainArg {
    fn default() -> Self {
        let mut main_arg = Self::parse();

        if let Some(common) = main_arg.mode.common_mut() {
            common.dir_name = gen_dir_name(&common.tmpdir);
        }

        main_arg
    }
}

fn gen_dir_name(input_name: &str) -> String {
    if input_name == DEFAULT_TMPDIR {
        let mut rng = rand::rng();
        let rand_num: u32 = rng.random();
        format!(
            "{}_{}_{}",
            chrono::Utc::now().format("/tmp/tmp_kanban_%Y%m%d%H%M%S"),
            rand_num,
            std::process::id()
        )
    } else {
        input_name.to_string()
    }
}

/// Options shared by every mode that writes to a temporary directory.
///
/// They render after each mode's own options because their display order sits
/// above anything the modes use, which is where they have always appeared.
#[derive(Debug, clap::Args, Clone)]
pub struct CommonArgs {
    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = DEFAULT_TMPDIR,
        help = "tmp directory name",
        display_order = 100
    )]
    pub tmpdir: String,

    #[clap(
        long,
        value_enum,
        default_value = "compile",
        help = "execution method",
        display_order = 101
    )]
    pub method: Method,

    /// Filled in by `MainArg::default`, never parsed from the command line.
    #[clap(skip)]
    pub dir_name: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Mode {
    /// one message on one top
    #[clap(display_order = 1)]
    Single(SingleArg),

    /// one message on many top
    #[clap(display_order = 2)]
    Multiple(MultipleArg),

    /// many message on many top
    #[clap(display_order = 3)]
    Multiple2(Multiple2Arg),

    /// one long message on many top with newline
    #[clap(display_order = 4)]
    Long(LongArg),

    /// message on many top vertically
    #[clap(display_order = 5)]
    Vertical(VerticalArg),

    /// one message on one top like electric bulletin board
    #[clap(display_order = 6)]
    Wave(WaveArg),

    /// one message on one nvtop/nvitop
    #[clap(display_order = 7)]
    Gpu(GpuArg),

    /// simple cpu execution without command rename
    #[clap(display_order = 8)]
    RawSingle(RawSingleArg),

    /// simple gpu execution without command rename
    #[clap(display_order = 9)]
    RawGpu(RawGpuArg),

    /// serve a local web UI that runs kanban on this machine
    #[clap(display_order = 10)]
    Serve(ServeArg),
}

impl Mode {
    /// The temp-directory settings, for the modes that have any.
    ///
    /// Listing the raw modes explicitly rather than falling through a wildcard
    /// is deliberate: a new mode that forgets to wire up `dir_name` is now a
    /// compile error instead of an empty path at runtime.
    fn common_mut(&mut self) -> Option<&mut CommonArgs> {
        match self {
            Mode::Single(arg) => Some(&mut arg.common),
            Mode::Multiple(arg) => Some(&mut arg.common),
            Mode::Multiple2(arg) => Some(&mut arg.common),
            Mode::Long(arg) => Some(&mut arg.common),
            Mode::Vertical(arg) => Some(&mut arg.common),
            Mode::Wave(arg) => Some(&mut arg.common),
            Mode::Gpu(arg) => Some(&mut arg.common),
            Mode::RawSingle(_) | Mode::RawGpu(_) | Mode::Serve(_) => None,
        }
    }
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct SingleArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "message that appears on top",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short = '@',
        long,
        value_name = "INT",
        default_value = "1",
        help = "thread number",
        display_order = 2
    )]
    pub thread: usize,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 3
    )]
    pub time: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct MultipleArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "message that appears on top",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short = '@',
        long,
        value_name = "INT",
        default_value = "1",
        help = "thread number",
        display_order = 2
    )]
    pub thread: usize,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 3
    )]
    pub time: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct Multiple2Arg {
    #[clap(
        short,
        long,
        value_name = "STR STR STR ... STR",
        help = "message that appears on top\n[CAUTION] number of thread used is automatically determined",
        value_parser,
        required = true,
        value_delimiter = ' ',
        num_args = 1..,
        display_order = 1
    )]
    pub message: Vec<String>,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 2
    )]
    pub time: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct LongArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "one long message that appears on top",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 2
    )]
    pub time: usize,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "12",
        help = "characters per top",
        display_order = 3
    )]
    pub length: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct VerticalArg {
    #[clap(
        short,
        long,
        value_name = "STR STR STR ... STR",
        help = "message that appears on top\n[CAUTION] number of thread used is automatically determined",
        value_parser,
        required = true,
        value_delimiter = ' ',
        num_args = 1..,
        display_order = 1
    )]
    pub message: Vec<String>,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 2
    )]
    pub time: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct WaveArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "one message on one top like electric bulletin board\n[CAUTION] execute time is automatically determined",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short = '@',
        long,
        value_name = "INT",
        default_value = "1",
        help = "thread numer",
        display_order = 2
    )]
    pub thread: usize,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "12",
        help = "characters per top",
        display_order = 3
    )]
    pub length: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct GpuArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "message that appears on top",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short = '@',
        long,
        value_name = "INT",
        default_value = "1",
        help = "thread number",
        display_order = 2
    )]
    pub thread: usize,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 3
    )]
    pub time: usize,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct RawSingleArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "message that appears on top without command rename",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short = '@',
        long,
        value_name = "INT",
        default_value = "1",
        help = "thread number",
        display_order = 2
    )]
    pub thread: usize,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 3
    )]
    pub time: usize,
}

#[derive(Debug, clap::Args, Clone)]
#[clap(version)]
pub struct ServeArg {
    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "8787",
        help = "port to listen on",
        display_order = 1
    )]
    pub port: u16,
    // Deliberately no --host. The server runs whatever it is asked to run, so
    // it is only ever bound to the loopback interface.
}

#[derive(Debug, clap::Args, Clone)]
#[clap(arg_required_else_help = true, version)]
pub struct RawGpuArg {
    #[clap(
        short,
        long,
        value_name = "STR",
        help = "message that appears on top without command rename",
        required = true,
        display_order = 1
    )]
    pub message: String,

    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 2
    )]
    pub time: usize,
}
