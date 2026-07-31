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

/// Owns a temporary directory and removes it on the way out, panic included.
///
/// Cleanup used to be the last statement of each run function, which any
/// earlier failure skipped - a compile error left the directory behind for
/// good, because the guard below only deletes directories kanban marked, and
/// a re-run with the same --tmpdir then failed to create it.
pub struct TempDir {
    path: String,
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Only ever delete a directory carrying our marker file. Without this,
        // `--tmpdir ~/important` would be a recursive delete of real work.
        if !std::path::Path::new(&self.path)
            .join("kanban.idfile")
            .exists()
        {
            log::warn!("{} has no kanban.idfile, leaving it alone", self.path);
            return;
        }
        if let Err(e) = std::fs::remove_dir_all(&self.path) {
            log::warn!("failed to remove {}: {}", self.path, e);
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

    /// Run one compiled message. Thread count and duration travel through the
    /// environment, so every generated binary is identical apart from its name.
    fn execute(&self, dir_name: &str, message: &str, threads: usize, time: usize) {
        let path = std::path::Path::new(dir_name).join(message);
        // A relative program name is resolved against the *parent's* working
        // directory, not Command::current_dir, so make it absolute. This is
        // what removed the need to chdir the whole process.
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        run_checked(
            std::process::Command::new(&path)
                .env("KANBAN_THREAD", threads.to_string())
                .env("KANBAN_TIME", time.to_string()),
            &path.to_string_lossy(),
        );
    }

    /// Create the temporary directory, mark it as ours, and hand back a guard
    /// that removes it again.
    fn temp_dir(&self) -> TempDir {
        self.mkdir(self.dir_name());
        self.create_idfile();
        TempDir {
            path: self.dir_name().to_string(),
        }
    }

    fn mkdir(&self, dir_name: &str) {
        if let Err(e) = std::fs::create_dir_all(dir_name) {
            log::error!("failed to create directory {}: {}", dir_name, e);
            std::process::exit(1);
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

        let _guard = self.temp_dir();
        let run_dir = Arc::new(format!("{}/{}", dir_name, "run"));

        let self_t = Arc::new(self);
        self_t.mkdir(&run_dir);

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

        match order {
            ExecutionOrder::Parallel => {
                let mut thrs = Vec::new();
                for i in 0..messages.len() {
                    let messages = Arc::clone(&messages);
                    let threads = Arc::clone(&threads);
                    let run_dir = Arc::clone(&run_dir);
                    let self_r = Arc::clone(&self_t);
                    thrs.push(std::thread::spawn(move || {
                        self_r.execute(&run_dir, &messages[i], threads[i], time);
                    }));
                }
                thrs.into_iter().for_each(|h| h.join().unwrap());
            }
            ExecutionOrder::Sequential => {
                for i in 0..messages.len() {
                    self_t.execute(&run_dir, &messages[i], threads[i], time);
                }
            }
        }
    }
}
