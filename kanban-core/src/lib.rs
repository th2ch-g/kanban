//! Turning a message into the list of strings a mode wants to display.
//!
//! This is the whole of kanban's per-mode logic. Everything else - compiling a
//! binary per string, or naming a thread after each one - is shared machinery
//! that does not care which mode produced the list.
//!
//! Keeping it here, free of `clap` and of any I/O, means the same code answers
//! three callers: the CLI, the local server's preview endpoint, and a browser
//! build. A second implementation of "what would this look like" would drift.
//!
//! Every function works in `char`s. The original sliced byte offsets, which
//! mangled multi-byte text in `long`, misaligned the columns in `vertical`, and
//! panicked outright in `wave`.

/// One process showing the message.
pub fn single(message: &str) -> Vec<String> {
    vec![message.to_string()]
}

/// The same message repeated across `thread` processes.
pub fn multiple(message: &str, thread: usize) -> Vec<String> {
    vec![message.to_string(); thread]
}

/// One process per message, as given.
pub fn multiple2(messages: &[String]) -> Vec<String> {
    messages.to_vec()
}

/// The message wrapped into rows of `length` characters.
pub fn long(message: &str, length: usize) -> Vec<String> {
    // `chunks(0)` panics, and -l 0 is accepted by the CLI.
    let length = length.max(1);
    let chars: Vec<char> = message.chars().collect();

    if chars.len() <= length {
        return vec![message.to_string()];
    }

    chars
        .chunks(length)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

/// The messages transposed, so each word reads downwards in `top`.
///
/// Row `i` holds the `i`-th character of every word. Longer words are laid down
/// first and shorter ones padded with spaces, which keeps the columns aligned
/// no matter what order the words arrived in.
pub fn vertical(messages: &[String]) -> Vec<String> {
    let words: Vec<Vec<char>> = messages.iter().map(|s| s.chars().collect()).collect();
    let maxlen = words.iter().map(|w| w.len()).max().unwrap_or(0);

    let mut longest_first: Vec<&Vec<char>> = words.iter().collect();
    // Stable, so words of equal length keep their input order.
    longest_first.sort_by_key(|w| std::cmp::Reverse(w.len()));

    let mut rows = vec![String::new(); maxlen];
    for word in longest_first {
        for (i, c) in word.iter().enumerate() {
            rows[i].push(*c);
        }
        for row in rows.iter_mut().take(maxlen).skip(word.len()) {
            row.push(' ');
        }
    }
    rows
}

/// Successive frames of the message scrolling through a `length`-wide window.
///
/// The frames are meant to be shown one after another, not side by side.
pub fn wave(message: &str, length: usize) -> Vec<String> {
    let chars: Vec<char> = message.chars().collect();
    let msg_len = chars.len();
    let slice = |from: usize, to: usize| -> String { chars[from..to].iter().collect() };

    let mut frames = Vec::new();

    if msg_len < length {
        // The message is narrower than the window, so it slides across padding.
        let pad = length - msg_len;
        for i in 0..msg_len {
            frames.push(format!(
                "{}{}{}",
                slice(i, msg_len),
                " ".repeat(pad),
                slice(0, i)
            ));
        }
        for i in 0..pad {
            frames.push(format!(
                "{}{}{}",
                " ".repeat(pad - i),
                message,
                " ".repeat(i)
            ));
        }
    } else {
        // The message is at least as wide as the window, so it wraps around
        // through a single separating space and is cropped to the window.
        for i in 0..=msg_len {
            let frame: Vec<char> = slice(i, msg_len)
                .chars()
                .chain(std::iter::once(' '))
                .chain(slice(0, i).chars())
                .collect();
            let end = length.min(frame.len());
            frames.push(frame[..end].iter().collect());
        }
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    // Multi-byte input. `long` used to chunk raw bytes and rebuild them with
    // from_utf8_lossy, so every row boundary produced U+FFFD; `wave` sliced at
    // byte offsets and panicked outright; `vertical` measured width in bytes
    // but wrote characters, so the padding never lined up.

    #[test]
    fn long_splits_japanese_by_character() {
        assert_eq!(
            long("あけましておめでとう", 3),
            ["あけま", "してお", "めでと", "う"]
        );
        // No replacement characters anywhere.
        assert!(!long("あけまして", 2).concat().contains('\u{FFFD}'));
    }

    #[test]
    fn wave_shifts_japanese_without_panicking() {
        // Five characters through a three-wide window: one frame per shift plus
        // the fully wrapped one, six in all.
        assert_eq!(
            wave("あけまして", 3),
            ["あけま", "けまし", "まして", "して ", "て あ", " あけ"]
        );
    }

    #[test]
    fn vertical_aligns_japanese_columns() {
        // Two characters wide, two rows - byte lengths would have said six.
        assert_eq!(vertical(&owned(&["あい", "うえ"])), ["あう", "いえ"]);
        // The shorter word is padded to the full height.
        assert_eq!(
            vertical(&owned(&["あい", "うえお"])),
            ["うあ", "えい", "お "]
        );
    }

    #[test]
    fn long_survives_a_zero_length() {
        // The CLI accepts -l 0, and chunks(0) panics.
        assert_eq!(long("abc", 0), ["a", "b", "c"]);
    }

    #[test]
    fn empty_input_is_not_a_panic() {
        assert_eq!(long("", 3), [""]);
        assert_eq!(vertical(&[]), Vec::<String>::new());
        assert_eq!(multiple("a", 0), Vec::<String>::new());
        // Nothing to scroll, so every frame is blank window.
        assert_eq!(wave("", 3), ["   ", "   ", "   "]);
    }
}
