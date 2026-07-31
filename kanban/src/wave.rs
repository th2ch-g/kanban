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
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
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
    fn run_by_compile(self) {
        // This used to be a near-verbatim copy of template_run. The only real
        // difference is that the frames run one at a time, which is what makes
        // the message appear to scroll.
        let threads = self.thread();
        self.clone().template_run(
            self.time(),
            ThreadPlan::Uniform(threads),
            ExecutionOrder::Sequential,
        );
    }
}

impl ProcnameTopMessage for WaveArg {}
