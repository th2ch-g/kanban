use kanban::arg::*;
use kanban::kanban_run;
use mpi::traits::*;
use std::env;

fn main() {
    let cli = MainArg::default();
    // mpi
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank();
    let root_rank = 0;
    kanban_run(&cli);
    world.barrier();
    if rank == root_rank {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .init();
        kanban_run(&cli);
        log::info!("{} done", env!("CARGO_PKG_NAME"));
    }
}
