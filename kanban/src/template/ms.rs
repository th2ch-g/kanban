use std::sync::Arc;
use std::thread;
use std::time::Instant;

const THREAD: usize = { thread };
const TIME: u64 = { time };

fn main() {
    let start = Arc::new(Instant::now());
    let mut thrs = Vec::new();
    for _ in 0..THREAD {
        let start = Arc::clone(&start);
        thrs.push(thread::spawn(move || loop {
            if start.elapsed().as_secs() >= TIME {
                break;
            }
        }));
    }
    thrs.into_iter().for_each(|h| h.join().unwrap());
}
