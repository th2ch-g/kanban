// The program kanban compiles once per message. Its *filename* is the message,
// which is what top displays; the body just has to keep a core busy long enough
// to stay near the top of the list.
//
// Thread count and duration arrive through the environment. They used to be
// pasted into this file as text before compiling, which left it invalid Rust and
// therefore invisible to cargo, clippy and rustfmt. Environment variables also
// stay out of the process name, unlike command line arguments.

use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn param<T: std::str::FromStr>(key: &str, fallback: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

fn main() {
    let threads: usize = param("KANBAN_THREAD", 1);
    let time: u64 = param("KANBAN_TIME", 10);

    let start = Arc::new(Instant::now());
    let mut thrs = Vec::new();
    for _ in 0..threads {
        let start = Arc::clone(&start);
        thrs.push(thread::spawn(move || loop {
            if start.elapsed().as_secs() >= time {
                break;
            }
        }));
    }
    thrs.into_iter().for_each(|h| h.join().unwrap());
}
