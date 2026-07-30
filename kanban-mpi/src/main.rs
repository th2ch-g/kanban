use kanban::arg::*;
use kanban::kanban_run;
use mpi::traits::*;

fn main() {
    // Install the logger on every rank and before any work: it used to be set
    // up inside the root-only branch, after the first kanban_run had already
    // finished, so non-root ranks logged nothing at all and the root's first
    // run was silent.
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = MainArg::default();

    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank();
    let root_rank = 0;

    kanban_run(&cli);
    world.barrier();

    if rank == root_rank {
        log::info!("{} done", env!("CARGO_PKG_NAME"));
    }
}
