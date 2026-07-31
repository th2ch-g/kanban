use kanban::arg::*;
use kanban::kanban_run;

fn main() {
    // parse_default_env keeps RUST_LOG working; Builder::new() alone ignores
    // the environment entirely, pinning everyone to info.
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    let cli = MainArg::default();
    kanban_run(&cli);
    log::info!("{} done", env!("CARGO_PKG_NAME"));
}
