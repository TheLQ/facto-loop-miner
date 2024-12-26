const C_SHELL_ESCAPE: &str = "\x1b";
const FORMAT_START: &str = "[";
const FORMAT_RESET: &str = "0";
const FOROMAT_END: &str = "m";

pub fn ansi_color(string: impl AsRef<str>, color: Color) -> String {
    [
        // start format
        C_SHELL_ESCAPE,
        FORMAT_START,
        &color.escape_code(),
        FOROMAT_END,
        // actual string
        string.as_ref(),
        // format reset
        C_SHELL_ESCAPE,
        FORMAT_START,
        FORMAT_RESET,
        FOROMAT_END,
    ]
    .concat()
}

pub enum Color {
    Green,
    Purple,
}

impl Color {
    fn escape_code(&self) -> String {
        match &self {
            Self::Green => "32".into(),
            Self::Purple => "95".into(),
        }
    }
}

pub const EMOJI_BROWN: &str = "\u{1F3FB}"; // 🏽
pub const EMOJI_POINT: &str = "\u{1F449}"; // 👉

pub fn ascii_previous_line() -> String {
    [
        // previous
        C_SHELL_ESCAPE,
        FORMAT_START,
        "F",
    ]
    .concat()
}

pub fn ascii_erase_line() -> String {
    [
        // erase line
        C_SHELL_ESCAPE,
        FORMAT_START,
        "K",
    ]
    .concat()
}
