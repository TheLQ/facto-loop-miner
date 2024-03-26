/// ```text
/// ▄▄▄▄
///    █
///    █
/// ```
pub fn rail_turn_template_up_left() -> [[bool; 12]; 12] {
    const READABLE_TEMPLATE: [[u8; 12]; 12] = [
        *b"0000........", // 0
        *b"00000.......",
        *b"...000......", // 2
        *b".....000....",
        *b"......000...", // 4 ////
        *b".......00...", //   //////
        *b"........00..", // 6 //////q
        *b".........00.", //   ////
        *b".........000", // 8
        *b"..........00",
        *b"..........00", // 10
        *b"..........00",
    ];
    template_to_bool(&READABLE_TEMPLATE)
}

/// ```text
/// ▄▄▄▄
/// █
/// █
/// ```
pub fn rail_turn_template_up_right() -> [[bool; 12]; 12] {
    let mut res = rail_turn_template_up_left();
    template_flip_horizontal(&mut res);
    res
}

/// ```text
///    █
///    █
/// ▀▀▀▀
/// ```
pub fn rail_turn_template_down_left() -> [[bool; 12]; 12] {
    let mut res = rail_turn_template_up_left();
    template_flip_vertical(&mut res);
    res
}

/// ```text
/// █
/// █
/// ▀▀▀▀
/// ```
pub fn rail_turn_template_down_right() -> [[bool; 12]; 12] {
    let mut res = rail_turn_template_up_left();
    template_flip_horizontal(&mut res);
    template_flip_vertical(&mut res);
    res
}

fn template_to_bool<const S: usize>(input: &[[u8; S]; S]) -> [[bool; S]; S] {
    let mut res: [[bool; S]; S] = [[false; S]; S];
    for row_pos in 0..S {
        for col_pos in 0..S {
            let value = input[row_pos][col_pos];
            if value == b'0' {
                res[row_pos][col_pos] = true;
            }
        }
    }
    res
}

fn template_flip_horizontal<T, const S: usize>(input: &mut [[T; S]; S]) {
    for row in input.iter_mut() {
        for col_start in 0..(row.len() / 2) {
            let col_end = row.len() - 1 - col_start;
            row.swap(col_start, col_end);
        }
    }
}

fn template_flip_vertical<T, const S: usize>(input: &mut [[T; S]; S]) {
    for row_start in 0..(input.len() / 2) {
        let row_end = input.len() - 1 - row_start;
        input.swap(row_start, row_end);
    }
}
