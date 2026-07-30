use crate::method::*;
use std::io::prelude::*;
use std::sync::Arc;

/// How many threads each spawned process runs.
///
/// `top` sorts by CPU share, so giving earlier processes more threads is what
/// keeps a multi-row message readable top to bottom instead of shuffling every
/// refresh. The three variants are genuinely different and all three are in use.
#[derive(Debug, Clone, Copy)]
pub enum ThreadPlan {
    /// One thread per process: `multiple` and `multiple2`, where every row is
    /// the same message and order does not matter.
    One,
    /// Process `i` of `n` gets `n - i` threads: `long` and `vertical`, whose
    /// rows have to stay in order to be legible.
    Decreasing,
    /// Every process gets the same count: `wave`, which runs one frame at a
    /// time and has no ordering to preserve.
    Uniform(usize),
}

impl ThreadPlan {
    fn counts(&self, len: usize) -> Vec<usize> {
        match self {
            ThreadPlan::One => vec![1; len],
            ThreadPlan::Decreasing => (0..len).map(|i| len - i).collect(),
            ThreadPlan::Uniform(n) => vec![*n; len],
        }
    }
}

/// Whether the compiled binaries run all at once or one after another.
#[derive(Debug, Clone, Copy)]
pub enum ExecutionOrder {
    /// Every message on screen simultaneously.
    Parallel,
    /// One message at a time, which is what makes `wave` animate.
    Sequential,
}

pub trait CompileTopMessage: CommonTopMessage
where
    Self: 'static,
{
    fn run_by_compile(self); // due to parallel process

    fn compile(&self, dir_name: &str, message: &str) {
        run_checked(
            std::process::Command::new("rustc")
                .arg(format!("{}/{}", dir_name, "ms.rs"))
                .arg("-o")
                .arg(format!("{}/{}", dir_name, message)),
            "rustc",
        );
    }

    fn compile_with_subdir(&self, dir_name: &str, subdir: &str, message: &str) {
        run_checked(
            std::process::Command::new("rustc")
                .arg(format!("{}/{}/{}", dir_name, subdir, "ms.rs"))
                .arg("-o")
                .arg(format!("{}/{}/{}", dir_name, "run", message)),
            "rustc",
        );
    }

    fn record_current_dir(&self) -> String {
        let current_dir = std::path::PathBuf::from("./");
        let current_dir = std::fs::canonicalize(current_dir);
        match current_dir {
            Ok(s) => s.to_string_lossy().to_string(),
            Err(_) => {
                log::error!("failed to record current directory");
                String::from("err")
            }
        }
    }

    fn cd(&self, dir_name: &str) {
        let cd_result = std::env::set_current_dir(dir_name);
        match cd_result {
            Ok(_) => (),
            Err(_) => {
                log::error!("failed to cd");
            }
        }
    }

    /// Run one compiled message. Thread count and duration travel through the
    /// environment, so every generated binary is identical apart from its name.
    fn execute(&self, dir_name: &str, message: &str, threads: usize, time: usize) {
        let path = format!("{}/{}", dir_name, message);
        run_checked(
            std::process::Command::new(&path)
                .env("KANBAN_THREAD", threads.to_string())
                .env("KANBAN_TIME", time.to_string()),
            &path,
        );
    }

    fn rmdir(&self) {
        let dir_path = std::path::Path::new(self.dir_name());
        let idfile_path = dir_path.join("kanban.idfile");

        if idfile_path.exists() && std::fs::remove_dir_all(self.dir_name()).is_err() {
            log::warn!("failed to rmdir but continue");
        }
    }

    fn mkdir(&self, dir_name: &str) {
        let mkdir_result = std::fs::create_dir(dir_name);
        match mkdir_result {
            Ok(_) => (),
            Err(_) => {
                log::error!("failed to create directory");
                log::error!("check authority");
                std::process::exit(1);
            }
        }
    }

    fn create_idfile(&self) {
        let template = include_str!("../template/kanban.idfile");
        let output_path = format!("{}/kanban.idfile", self.dir_name());
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(template.as_bytes()).unwrap();
    }

    fn create_mainfile(&self, dir_name: &str) {
        let template = include_str!("../template/ms.rs");
        let output_path = format!("{}/ms.rs", dir_name);
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(template.as_bytes()).unwrap();
    }

    fn template_run(self, time: usize, plan: ThreadPlan, order: ExecutionOrder)
    where
        Self: Sync + Send + Sized,
    {
        // messages() rebuilds its Vec on every call, and vertical even re-sorts;
        // it used to sit in the loop conditions. Compute it once.
        let messages = Arc::new(self.messages());
        let threads = Arc::new(plan.counts(messages.len()));
        let dir_name = Arc::new(self.dir_name().to_string());
        let self_t = Arc::new(self);

        self_t.mkdir(&dir_name);
        self_t.mkdir(&format!("{}/{}", dir_name, "run"));

        self_t.create_idfile();

        let mut thrs = Vec::new();
        for i in 0..messages.len() {
            let dir_name = Arc::clone(&dir_name);
            let messages = Arc::clone(&messages);
            let self_r = Arc::clone(&self_t);
            thrs.push(std::thread::spawn(move || {
                let subdir = format!("{}/{}", dir_name, i);
                self_r.mkdir(&subdir);
                self_r.create_mainfile(&subdir);
                self_r.compile_with_subdir(&dir_name, &i.to_string(), &messages[i]);
            }));
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());

        let current_dir = self_t.record_current_dir();
        self_t.cd(&format!("{}/{}", dir_name, "run"));

        match order {
            ExecutionOrder::Parallel => {
                let mut thrs = Vec::new();
                for i in 0..messages.len() {
                    let messages = Arc::clone(&messages);
                    let threads = Arc::clone(&threads);
                    let self_r = Arc::clone(&self_t);
                    thrs.push(std::thread::spawn(move || {
                        self_r.execute(".", &messages[i], threads[i], time);
                    }));
                }
                thrs.into_iter().for_each(|h| h.join().unwrap());
            }
            ExecutionOrder::Sequential => {
                for i in 0..messages.len() {
                    self_t.execute(".", &messages[i], threads[i], time);
                }
            }
        }

        self_t.cd(&current_dir);

        self_t.rmdir();
    }
}
