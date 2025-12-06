use crate::arg::*;
use crate::method::{compile::*, procname::*, *};
use std::sync::Arc;
use std::thread::Builder;
use std::time::Instant;

impl CommonTopMessage for SingleArg {
    fn messages(&self) -> Vec<String> {
        vec![self.message.clone()]
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn method(&self) -> Method {
        self.method
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
        self.mkdir(self.dir_name());

        self.create_mainfile(self.dir_name(), self.thread(), self.time());

        self.create_idfile();

        self.compile(self.dir_name(), &self.messages()[0]);

        let current_dir = self.record_current_dir();

        self.cd(&self.dir_name);

        self.execute(".", &self.messages()[0]);

        self.cd(&current_dir);

        self.rmdir();
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
