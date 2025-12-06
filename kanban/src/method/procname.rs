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

        let messages = self.messages();
        let thread_count = self.thread();

        // Determine the list of messages (thread names) to spawn.
        let final_messages: Vec<String> = if messages.len() == 1 {
            // Case 1: Single message.
            // If thread_count > 1, we want N threads with the same name.
            vec![messages[0].clone(); thread_count]
        } else if messages.len() == thread_count {
            // Case 2: Number of messages equals thread count.
            // This happens in MultipleArg (where messages() returns N copies),
            // Multiple2Arg, VerticalArg, LongArg (where thread() returns messages().len()).
            messages
        } else {
            // Case 3: Messages count != thread count.
            // WaveArg falls here?
            // WaveArg: messages() returns M strings (shifted versions).
            // thread() returns user input (default 1).
            // But wait, in WaveArg implementation of CommonTopMessage, I returned `self.thread`.
            // If I want to simulate "wave", I should probably spawn M threads,
            // OR spawn M * thread threads?
            // Original Wave implementation runs M processes sequentially (or concurrently? wait).
            // In `run` for WaveArg:
            // It spawns threads to compile M variants.
            // Then it runs `execute` for each variant.
            // `for message in self.messages() { self_t.execute(".", &message); }`
            // This executes them *sequentially* in the main thread (or rather, the spawned process executes and waits?).
            // `self.execute` uses `Command::output`, which waits for completion.
            // BUT, `execute` runs the compiled binary which runs for 2 seconds.
            // So WaveArg runs variant 0 for 2s, then variant 1 for 2s, etc.
            // This creates a slow wave effect.

            // If we want to replicate this in `procname` mode:
            // We can't easily spawn sequential threads that block the main loop in the same way with just `run_by_procname` structure.
            // Unless we change the structure.

            // However, maybe the user wants parallel execution for Wave in procname mode (like "all variants visible at once")?
            // "one message on one top like electric bulletin board"
            // If all variants are visible at once, it's just a lot of processes.
            // If it's a bulletin board, it should change over time.

            // If `messages` are shifted versions, displaying them all at once would look like a mess or a static block.
            // To make it "wave", we need to change the thread name dynamically?
            // `set_name` API for thread is not stable or available easily in cross-platform std.
            // Linux `prctl` can do it.

            // But `Procname` mode seems to be "Process Name" mode, i.e., spawning threads with names.
            // If we can't change name, maybe we just spawn all of them?

            // Let's look at `MultipleArg`. `messages` returns `thread` copies.
            // `Multiple2Arg`. `messages` returns user input list. `thread` returns list length.

            // For WaveArg:
            // If we simply spawn all M variants as threads, they will all appear in `top`.
            // It won't look like a wave, it will look like M threads.

            // Given the constraints and the goal "implement procname for other modes",
            // maybe "running concurrently" is the intended behavior for `procname` mode?
            // If so, we should spawn M threads (or M * thread threads).

            // If `thread_count` is 1 (default for WaveArg), and `messages` len is M.
            // We should probably spawn M threads.

            if thread_count > 0 && messages.len() % thread_count == 0 {
                 // Maybe this case?
                 messages
            } else {
                 // Just spawn all messages?
                 // Or spawn M * thread_count?
                 let mut v = Vec::new();
                 for m in messages {
                     for _ in 0..std::cmp::max(1, thread_count) {
                         v.push(m.clone());
                     }
                 }
                 v
            }
        };

        for message in final_messages {
            let start = Arc::clone(&start);
            let time_r = Arc::clone(&time_t);
            let builder = Builder::new().name(message);

            thrs.push(
                builder
                    .spawn(move || loop {
                        if start.elapsed().as_secs() >= *time_r as u64 {
                            break;
                        }
                        // To avoid 100% CPU usage, maybe sleep a bit?
                        // But the original `compile` method loops tight (or waits for `output` which waits for the process).
                        // The process itself:
                        // `ms.rs` template:
                        // `let start = std::time::Instant::now(); loop { if start.elapsed().as_secs() >= time { break; } }`
                        // It's a busy loop.
                        // So we keep busy loop here too.
                        std::thread::yield_now();
                    })
                    .unwrap(),
            );
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());
    }
}
