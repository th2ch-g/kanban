use std::sync::Arc;
use std::thread::Builder;
use std::time::Instant;
use crate::method::*;

pub trait ProcnameTopMessage: CommonTopMessage
where
    Self: Sized,
{
    fn run_by_procname(self) {
        let start = Arc::new(Instant::now());
        let time_t = Arc::new(self.time());
        let mut thrs = Vec::new();

        for _ in 0..self.thread() {
            let start = Arc::clone(&start);
            let time_r = Arc::clone(&time_t);
            let message = self.messages()[0].to_string();
            let builder = Builder::new().name(message);

            thrs.push(
                builder
                    .spawn(move || loop {
                        if start.elapsed().as_secs() >= *time_r as u64 {
                            break;
                        }
                    })
                    .unwrap(),
            );
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());
    }
}
