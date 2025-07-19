use kanban::arg::*;
use kanban::kanban_run;

fn main() {
    let cli = MainArg::default();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    kanban_run(&cli);
    log::info!("{} done", env!("CARGO_PKG_NAME"));
}
