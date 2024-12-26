const C_SHELL_ESCAPE: &str = "\x1b";
const FORMAT_START: &str = "[";
const FORMAT_RESET: &str = "0";
const FOROMAT_END: &str = "m";

pub fn ascii_color(string: impl AsRef<str>, color: Color) -> String {
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
}

impl Color {
    fn escape_code(&self) -> String {
        match &self {
            Self::Green => "32".into(),
        }
    }
}
