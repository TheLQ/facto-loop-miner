/// We don't need no `\s+` regex library
pub fn string_space_shrinker(input: impl AsRef<str>) -> String {
    match 2 {
        1 => string_space_shrinker_state_slice(input),
        2 => string_space_shrinker_state_vec(input),
        3 => string_space_shrinker_doubler(input),
        _ => unreachable!(),
    }
}

/// State machine edition
/// Tweak of _vec using theoretically better memcpy copy_from_slice
///
/// Benchmark verdict:
///  - wider execution time range, faster in some
///  - Vec is faster?
pub fn string_space_shrinker_state_slice(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    let in_bytes = input.as_bytes();
    let in_len = in_bytes.len();
    // must pad because pure newline seps output longer string than input
    let mut out_bytes = vec![0u8; (in_len as f32 * 1.1) as usize];

    enum Code {
        Space,
        Newline,
        Letter,
    }
    enum State {
        Stuff { start: usize },
        Space,
        SpaceAndNewline,
    }
    let mut state = State::Stuff { start: 0 };
    let mut out_pos = 0;
    for i in 0..in_len {
        let code = match in_bytes[i] {
            b' ' => Code::Space,
            b'\n' => Code::Newline,
            _ => Code::Letter,
        };

        match (code, &state) {
            (Code::Space, State::Space | State::SpaceAndNewline) => {
                // is space, keep counting spaces
            }
            (Code::Newline, State::Space) => {
                // keep counting spaces
                state = State::SpaceAndNewline
            }
            (Code::Newline, State::SpaceAndNewline) => {
                // is newline, keep counting spaces
            }
            (Code::Space, State::Stuff { start }) => {
                // stuff ended
                let len = i - start;
                out_bytes[out_pos..(out_pos + len)].copy_from_slice(&in_bytes[*start..i]);
                out_pos += len;
                state = State::Space;
            }
            (Code::Newline, State::Stuff { start }) => {
                // stuff ended
                let len = i - start;
                out_bytes[out_pos..(out_pos + len)].copy_from_slice(&in_bytes[*start..i]);
                out_pos += len;
                state = State::SpaceAndNewline;
            }
            (Code::Letter, State::Stuff { start: _ }) => {
                // continue counting
            }
            (Code::Letter, State::Space) => {
                // space ended, without including start space
                if out_pos != 0 {
                    out_bytes[out_pos] = b' ';
                    out_pos += 1;
                }
                state = State::Stuff { start: i };
            }
            (Code::Letter, State::SpaceAndNewline) => {
                // space ended, without including start space
                if out_pos != 0 {
                    out_bytes[out_pos..(out_pos + 2)].copy_from_slice(b"  ");
                    out_pos += 2;
                }
                state = State::Stuff { start: i };
            }
        }
    }
    // last word
    if let State::Stuff { start } = state {
        let len = in_len - start;
        out_bytes[out_pos..(out_pos + len)].copy_from_slice(&in_bytes[start..in_len]);
        out_pos += len;
    }
    out_bytes.truncate(out_pos);

    // str::from_utf8(&out_bytes[0..out_pos]).unwrap().to_string()
    String::from_utf8(out_bytes).unwrap()
}

pub fn string_space_shrinker_state_vec(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    let in_bytes = input.as_bytes();
    let in_len = in_bytes.len();
    let mut out_bytes = Vec::with_capacity(in_len);

    enum Code {
        Space,
        Newline,
        Letter,
    }
    enum State {
        Stuff { start: usize },
        Space,
        SpaceAndNewline,
    }
    let mut state = State::Stuff { start: 0 };
    for i in 0..in_len {
        let code = match in_bytes[i] {
            b' ' => Code::Space,
            b'\n' => Code::Newline,
            _ => Code::Letter,
        };

        match (code, &state) {
            (Code::Space, State::Space | State::SpaceAndNewline) => {
                // is space, keep counting spaces
            }
            (Code::Newline, State::Space) => {
                // keep counting spaces
                state = State::SpaceAndNewline
            }
            (Code::Newline, State::SpaceAndNewline) => {
                // is newline, keep counting spaces
            }
            (Code::Space, State::Stuff { start }) => {
                // stuff ended
                out_bytes.extend_from_slice(&in_bytes[*start..i]);
                state = State::Space;
            }
            (Code::Newline, State::Stuff { start }) => {
                // stuff ended
                out_bytes.extend_from_slice(&in_bytes[*start..i]);
                state = State::SpaceAndNewline;
            }
            (Code::Letter, State::Stuff { start: _ }) => {
                // continue counting
            }
            (Code::Letter, State::Space) => {
                // space ended, without including start space
                if !out_bytes.is_empty() {
                    out_bytes.push(b' ');
                }
                state = State::Stuff { start: i };
            }
            (Code::Letter, State::SpaceAndNewline) => {
                // space ended, without including start space
                if !out_bytes.is_empty() {
                    out_bytes.extend_from_slice(b"  ");
                }
                state = State::Stuff { start: i };
            }
        }
    }
    // last word
    if let State::Stuff { start } = state {
        out_bytes.extend_from_slice(&in_bytes[start..in_len]);
    }

    String::from_utf8(out_bytes).unwrap()
}

/// Double-pass
/// Benchmark
pub fn string_space_shrinker_doubler(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    let mut in_bytes = input.as_bytes().to_vec();
    let input_len = in_bytes.len();

    // detect
    #[derive(Debug)]
    struct RemoveEntry {
        start: usize,
        end: usize,
        is_newline: bool,
    }
    let mut active_remove: Option<RemoveEntry> = None;
    let mut removes: Vec<RemoveEntry> = Vec::new();
    for i in 0..input_len {
        let cur_char = in_bytes[i];
        if is_space_or_newline(cur_char) {
            if let Some(active_remove) = &mut active_remove {
                active_remove.end += 1;
                active_remove.is_newline = active_remove.is_newline || is_newline(cur_char);
            } else {
                active_remove = Some(RemoveEntry {
                    start: i,
                    end: i,
                    is_newline: is_newline(cur_char),
                })
            }
        } else {
            if let Some(active_remove) = active_remove {
                removes.push(active_remove);
            }
            active_remove = None;
        }
    }
    if let Some(active_remove) = active_remove {
        removes.push(active_remove);
    }

    // execute
    for remove in removes.iter().rev() {
        in_bytes.drain(remove.start..=remove.end);
        if remove.start == 0 || remove.end == input_len - 1 {
            // trim
            continue;
        }
        in_bytes.insert(remove.start, b' ');
        if remove.is_newline {
            // double
            in_bytes.insert(remove.start, b' ');
        }
    }
    String::from_utf8(in_bytes).unwrap()
}

fn is_space_or_newline(cur_char: u8) -> bool {
    is_space(cur_char) || is_newline(cur_char)
}

fn is_newline(cur_char: u8) -> bool {
    cur_char == b'\n'
}

fn is_space(cur_char: u8) -> bool {
    cur_char == b' '
}

#[cfg(test)]
mod test {
    use crate::admiral::trimmer::string_space_shrinker;

    #[test]
    fn basic_test() {
        let expected = "first second  next line";

        assert_eq!(
            string_space_shrinker("  first second  \n  next  line  "),
            expected
        );
        assert_eq!(
            string_space_shrinker("  first second \nnext  line"),
            expected
        );
        assert_eq!(
            string_space_shrinker("  first second\n next  line"),
            expected
        );
        assert_eq!(
            string_space_shrinker("first second \nnext  line\n"),
            expected
        );
        assert_eq!(
            string_space_shrinker("first second \nnext  line\n  "),
            expected
        );
        assert_eq!(
            string_space_shrinker("first second \nnext  line  \n"),
            expected
        );
        assert_eq!(string_space_shrinker("first second\nnext line  "), expected);
    }

    #[test]
    fn good_test() {
        let input = "i'm a teapot".to_string();
        let actual = string_space_shrinker(input.clone());
        assert_eq!(format!("|{input}|"), format!("|{actual}|"))
    }

    #[test]
    fn none_test() {
        let input = "teapot".to_string();
        let actual = string_space_shrinker(input.clone());
        assert_eq!(format!("|{input}|"), format!("|{actual}|"))
    }
}
