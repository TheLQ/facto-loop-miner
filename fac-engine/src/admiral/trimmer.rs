/// We don't need no `\s+` regex library
pub fn string_space_shrinker(input: impl AsRef<str>) -> String {
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
    cur_char == b'\n' || cur_char == b' '
}

fn is_newline(cur_char: u8) -> bool {
    cur_char == b'\n'
}

#[cfg(test)]
mod test {
    use crate::admiral::trimmer::string_space_shrinker;
    use facto_loop_miner_common::log_init_trace;

    #[test]
    fn basic_test() {
        log_init_trace();
        let input = "  first second  \n  next  line  ".to_string();

        let expected = "first second  next line";
        let actual = string_space_shrinker(input.clone());
        assert_eq!(format!("|{expected}|"), format!("|{actual}|"))
    }

    #[test]
    fn good_test() {
        log_init_trace();
        let input = "i'm a teapot".to_string();
        let actual = string_space_shrinker(input.clone());
        assert_eq!(format!("|{input}|"), format!("|{actual}|"))
    }
}
