use crate::method::*;
use std::sync::Arc;
use std::thread::Builder;
use std::time::Instant;

/// Linux stores a thread name in TASK_COMM_LEN bytes, one of which is the
/// terminator.
const MAX_THREAD_NAME_BYTES: usize = 15;

/// Shorten a thread name to what the kernel will keep, cutting on a character
/// boundary.
///
/// `Builder::name` ends up in prctl(PR_SET_NAME), which truncates at a fixed
/// byte count with no regard for encoding: `abあけましてお` came back from
/// /proc as `abあけまし\xe3`, invalid UTF-8, and rendered as a replacement
/// character in top. ASCII names are unaffected - the cut lands in the same
/// place either way.
pub fn fit_thread_name(name: &str) -> String {
    if name.len() <= MAX_THREAD_NAME_BYTES {
        return name.to_string();
    }
    let mut end = MAX_THREAD_NAME_BYTES;
    while end > 0 && !name.is_char_boundary(end) {
        end -= 1;
    }
    name[..end].to_string()
}

pub trait ProcnameTopMessage: CommonTopMessage
where
    Self: Sized,
{
    fn run_by_procname(self) {
        let names = thread_names(self.messages(), self.thread());

        let start = Arc::new(Instant::now());
        let time = Arc::new(self.time());
        let mut thrs = Vec::new();

        for name in names {
            let start = Arc::clone(&start);
            let time = Arc::clone(&time);

            thrs.push(
                Builder::new()
                    .name(fit_thread_name(&name))
                    // Busy-wait rather than sleep, matching the compiled
                    // template: the thread has to burn CPU to rank high enough
                    // for top to show it.
                    .spawn(move || {
                        while start.elapsed().as_secs() < *time as u64 {
                            std::thread::yield_now();
                        }
                    })
                    .unwrap(),
            );
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());
    }
}

/// One name per thread to spawn.
///
/// Most modes line up already, because `thread()` reports the length of
/// `messages()`. Two do not: a lone message wants `thread_count` copies of
/// itself, and `wave`'s frame count is unrelated to `-@`, so each frame gets
/// `thread_count` threads unless the counts happen to divide.
fn thread_names(messages: Vec<String>, thread_count: usize) -> Vec<String> {
    if messages.len() == 1 {
        return vec![messages[0].clone(); thread_count];
    }
    if messages.len().is_multiple_of(thread_count.max(1)) {
        return messages;
    }
    messages
        .iter()
        .flat_map(|m| std::iter::repeat_n(m.clone(), thread_count.max(1)))
        .collect()
}
