//! Expected output of every mode's `messages()`.
//!
//! `messages()` is the only place per-mode logic lives; everything downstream
//! just turns those strings into process or thread names. Pinning the exact
//! output is what makes the surrounding machinery safe to rearrange.
//!
//! These cases are all ASCII. Multi-byte input is slicing-unsafe today and gets
//! its own coverage once that is fixed.

use crate::arg::*;
use crate::method::*;

const TMPDIR: &str = "/tmp/kanban_test";

fn single(message: &str, thread: usize) -> SingleArg {
    SingleArg {
        message: message.to_string(),
        thread,
        time: 1,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

fn multiple(message: &str, thread: usize) -> MultipleArg {
    MultipleArg {
        message: message.to_string(),
        thread,
        time: 1,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

fn multiple2(message: &[&str]) -> Multiple2Arg {
    Multiple2Arg {
        message: message.iter().map(|s| s.to_string()).collect(),
        time: 1,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

fn long(message: &str, length: usize) -> LongArg {
    LongArg {
        message: message.to_string(),
        length,
        time: 1,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

fn vertical(message: &[&str]) -> VerticalArg {
    VerticalArg {
        message: message.iter().map(|s| s.to_string()).collect(),
        time: 1,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

fn wave(message: &str, length: usize, thread: usize) -> WaveArg {
    WaveArg {
        message: message.to_string(),
        length,
        thread,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

fn gpu(message: &str) -> GpuArg {
    GpuArg {
        message: message.to_string(),
        thread: 1,
        time: 1,
        common: CommonArgs {
            tmpdir: TMPDIR.to_string(),
            method: Method::Procname,
            dir_name: TMPDIR.to_string(),
        },
    }
}

#[test]
fn single_is_the_message_once_regardless_of_thread_count() {
    assert_eq!(single("aaa", 1).messages(), ["aaa"]);
    assert_eq!(single("aaa", 4).messages(), ["aaa"]);
    assert_eq!(single("aaa", 4).thread(), 4);
}

#[test]
fn multiple_repeats_the_message_once_per_thread() {
    assert_eq!(multiple("aaa", 3).messages(), ["aaa", "aaa", "aaa"]);
    assert_eq!(multiple("aaa", 1).messages(), ["aaa"]);
}

#[test]
fn multiple2_passes_its_input_through() {
    let arg = multiple2(&["aaa", "bbb", "ccc"]);
    assert_eq!(arg.messages(), ["aaa", "bbb", "ccc"]);
    // Thread count follows the message count rather than a flag.
    assert_eq!(arg.thread(), 3);
}

#[test]
fn long_splits_into_rows_of_length() {
    assert_eq!(long("aaabbb", 3).messages(), ["aaa", "bbb"]);
    // A trailing partial row is kept as-is.
    assert_eq!(long("aaabbbc", 3).messages(), ["aaa", "bbb", "c"]);
    // Nothing to split.
    assert_eq!(long("ab", 3).messages(), ["ab"]);
    assert_eq!(long("abc", 3).messages(), ["abc"]);
    assert_eq!(long("aaabbb", 3).thread(), 2);
}

#[test]
fn vertical_transposes_longest_first() {
    // Equal lengths: column i is the i-th character of each word.
    assert_eq!(vertical(&["aa", "bb"]).messages(), ["ab", "ab"]);

    // Shorter words are padded with spaces so the columns stay aligned, and
    // the longest word is placed first no matter the input order.
    assert_eq!(vertical(&["de", "abc"]).messages(), ["ad", "be", "c "]);
    assert_eq!(vertical(&["abc", "de"]).messages(), ["ad", "be", "c "]);

    assert_eq!(vertical(&["abc", "de"]).thread(), 3);
}

#[test]
fn wave_shifts_the_message_one_frame_at_a_time() {
    // Message at least as long as the window: one frame per shift, plus the
    // wrapped-around frame, each cropped to the window.
    assert_eq!(
        wave("aaabbb", 3, 1).messages(),
        ["aaa", "aab", "abb", "bbb", "bb ", "b a", " aa"]
    );

    // Message shorter than the window: it slides across the padding instead.
    assert_eq!(
        wave("ab", 5, 1).messages(),
        ["ab   ", "b   a", "   ab", "  ab ", " ab  "]
    );
}

#[test]
fn wave_ignores_time_and_uses_two_seconds_per_frame() {
    // WaveArg has no -t at all; the frame duration is fixed. Seven frames of
    // two seconds is what makes `wave -m aaabbb -l 3` take fourteen seconds.
    assert_eq!(wave("aaabbb", 3, 1).time(), 2);
    assert_eq!(wave("aaabbb", 3, 1).messages().len(), 7);
}

#[test]
fn gpu_is_the_message_once() {
    assert_eq!(gpu("gpu").messages(), ["gpu"]);
}
