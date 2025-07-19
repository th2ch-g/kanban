use crate::arg::*;

impl RawSingleArg {
    pub fn run(&self) {
        let start = std::sync::Arc::new(std::time::Instant::now());
        let time_t = std::sync::Arc::new(self.time);
        let mut thrs = Vec::new();
        for _ in 0..self.thread {
            let start = std::sync::Arc::clone(&start);
            let time_r = std::sync::Arc::clone(&time_t);
            thrs.push(std::thread::spawn(move || loop {
                if start.elapsed().as_secs() >= *time_r as u64 {
                    break;
                }
            }));
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());
    }
}
