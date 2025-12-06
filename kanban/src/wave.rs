use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for WaveArg {
    fn messages(&self) -> Vec<String> {
        let msg_len = self.message.len();
        let mut message_list = Vec::new();

        if msg_len < self.length {
            for i in 0..msg_len {
                let tmp = format!(
                    "{}{}{}",
                    &self.message[i..],
                    " ".repeat(self.length - msg_len),
                    &self.message[..i]
                );
                message_list.push(tmp);
            }

            for i in 0..(self.length - msg_len) {
                let tmp = format!(
                    "{}{}{}",
                    " ".repeat(self.length - msg_len - i),
                    self.message,
                    " ".repeat(i)
                );
                message_list.push(tmp);
            }
        } else {
            for i in 0..=msg_len {
                let tmp = format!("{} {}", &self.message[i..], &self.message[..i]);
                message_list.push(tmp[..self.length.min(tmp.len())].to_string());
            }
        }
        message_list
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn method(&self) -> Method {
        self.method
    }

    fn thread(&self) -> usize {
        // In wave mode, multiple processes are launched, each with self.thread threads.
        // But for ProcnameTopMessage generic implementation we might want to return
        // something else if we want to mimic the behavior.
        // However, based on the plan, we will handle messages iteration in run_by_procname.
        // The `thread` method here returns user specified thread count per "process" (or message).
        self.thread
    }

    fn time(&self) -> usize {
        // Wave mode calculates execution time automatically based on length or message length in compile mode.
        // But here we need to return a usize.
        // If we want to simulate the wave, we might need a longer time.
        // But `WaveArg` doesn't have a `time` field.
        // Let's check `WaveArg` definition. It doesn't have `time`.
        // The help says "execute time is automatically determined".
        // In `run_by_compile` it doesn't use `time` either?
        // Wait, `WaveArg` implementation in `kanban/src/wave.rs`:
        // It calls `execute` and waits.
        // But `ms.rs` template uses `time`.
        // In `run` implementation of `WaveArg` (old code):
        // `self_r.create_mainfile(..., self.thread, 2);`
        // It hardcodes time to 2 seconds per step?
        // Ah, `WaveArg` logic is: compile N variants, then run them sequentially or in parallel?
        // "one message on one top like electric bulletin board"
        // It runs variants one by one or shifted?
        // `run` implementation:
        // `for message in self.messages() { self_t.execute(".", &message); }`
        // It executes them sequentially!
        // So each execution lasts for the time specified in `create_mainfile`.
        // The code says `2`.
        2
    }
}

impl CompileTopMessage for WaveArg {
    fn run_by_compile(self)
    where
        Self: Sync + Send,
    {
        let dir_name_t = std::sync::Arc::new(self.dir_name().to_string().clone());
        let message_list_t = std::sync::Arc::new(self.messages().clone());
        let self_t = std::sync::Arc::new(self.clone());

        self_t.mkdir(self_t.dir_name());
        self_t.mkdir(&format!("{}/{}", self_t.dir_name(), "run"));

        self_t.create_idfile();

        let mut thrs = Vec::new();

        for i in 0..self_t.messages().len() {
            let dir_name_r = std::sync::Arc::clone(&dir_name_t);
            let message_list_r = std::sync::Arc::clone(&message_list_t);
            let self_r = std::sync::Arc::clone(&self_t);

            thrs.push(std::thread::spawn(move || {
                self_r.mkdir(&format!("{}/{}", dir_name_r, i));
                // Hardcoded time 2 seconds as per original implementation
                self_r.create_mainfile(&format!("{}/{}", dir_name_r, i), self_r.thread(), 2);
                self_r.compile_with_subdir(&dir_name_r, &i.to_string(), &message_list_r[i]);
            }));
        }

        thrs.into_iter().for_each(|h| h.join().unwrap());

        let current_dir = self_t.record_current_dir();
        self_t.cd(&format!("{}/{}", self_t.dir_name(), "run"));

        for message in self.messages() {
            self_t.execute(".", &message);
        }

        self_t.cd(&current_dir);

        self_t.rmdir();
    }
}

impl ProcnameTopMessage for WaveArg {}
