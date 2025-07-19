use clap::{Parser, Subcommand};
use rand::prelude::*;

#[derive(Parser, Debug, Clone)]
#[clap(version, about)] //#[clap(author, version, about)]
pub struct MainArg {
    #[clap(subcommand)]
    pub mode: Mode,
}

impl Default for MainArg {
    fn default() -> Self {
        let mut main_arg = Self::parse();

        match &mut main_arg.mode {
            Mode::Single(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            Mode::Multiple(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            Mode::Multiple2(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            Mode::Long(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            Mode::Vertical(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            Mode::Wave(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            Mode::Gpu(arg) => arg.dir_name = gen_dir_name(&arg.tmpdir),
            _ => (),
        }

        main_arg
    }
}

fn gen_dir_name(input_name: &str) -> String {
    let default_tmpdir_name = String::from("/tmp/tmp_kanban_(date_randomnumber_pid)");
    if input_name == default_tmpdir_name {
        let mut rng = rand::thread_rng();
        let rand_num: u32 = rng.gen();
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

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 4
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 4
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 3
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 4
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 3
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 4
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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

    // #[clap(
    //     short = '@',
    //     long,
    //     value_name = "INT",
    //     default_value = "1",
    //     help = "thread number",
    //     display_order = 2
    // )]
    // pub thread: usize,
    #[clap(
        short,
        long,
        value_name = "INT",
        default_value = "10",
        help = "display time(s)",
        display_order = 3
    )]
    pub time: usize,

    #[clap(
        long = "tmpdir",
        value_name = "STR",
        default_value = "/tmp/tmp_kanban_(date_randomnumber_pid)",
        help = "tmp directory name",
        display_order = 4
    )]
    pub tmpdir: String,

    #[clap(skip)]
    pub dir_name: String,
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
