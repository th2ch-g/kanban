use crate::arg::*;
use crate::method::{compile::*, procname::*, *};
use std::sync::Arc;
use std::thread::Builder;
use std::time::Instant;

impl CommonTopMessage for SingleArg {
    fn messages(&self) -> Vec<String> {
        kanban_core::single(&self.message)
    }

    fn dir_name(&self) -> &str {
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
    }

    fn thread(&self) -> usize {
        self.thread
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for SingleArg {
    fn run_by_compile(self) {
        let _guard = self.temp_dir();

        self.create_mainfile(self.dir_name());

        let message = &self.messages()[0];
        self.compile(self.dir_name(), message);
        self.execute(self.dir_name(), message, self.thread(), self.time());
    }
}

impl ProcnameTopMessage for SingleArg {
    fn run_by_procname(self) {
        let start = Arc::new(Instant::now());
        let time_t = Arc::new(self.time());
        let mut thrs = Vec::new();

        for _ in 0..self.thread() {
            let start = Arc::clone(&start);
            let time_r = Arc::clone(&time_t);
            let message = self.messages()[0].to_string();
            let builder = Builder::new().name(fit_thread_name(&message));

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
