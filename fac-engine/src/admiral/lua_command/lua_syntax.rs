use crate::blueprint::bpfac::position::FacBpPosition;
use crate::common::vpoint::PSugar;
use std::borrow::Borrow;

pub struct LuaSyntax {
    pub method: String,
    pub args: Vec<SyntaxArg>,
}

pub struct SyntaxArg {
    pub key: String,
    pub value: String,
}

impl LuaSyntax {
    pub fn method(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.push(SyntaxArg {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn arg_pos(self, key: impl Into<String>, value: FacBpPosition) -> Self {
        let PSugar { x, y } = value.sugar();
        self.arg(key, format!("{{ {x}, {y} }}"))
    }

    pub fn arg_color(self, key: impl Into<String>, [r, g, b]: [u8; 3]) -> Self {
        self.arg(key, format!("{{ r={r},g={g},b={b} }}"))
    }

    pub fn arg_maybe<V>(
        mut self,
        key: impl Into<String>,
        value: Option<V>,
        mapper: impl Fn(V) -> String,
    ) -> Self {
        if let Some(value) = value {
            self.arg(key, mapper(value))
        } else {
            self
        }
    }

    pub fn build(self) -> String {
        let mut output = self.method;
        output.push('{');

        let mut args_iter = self.args.into_iter().peekable();
        while let Some(SyntaxArg { key, value }) = args_iter.next() {
            output.push_str(&key);
            output.push('=');
            output.push_str(&value);
            if args_iter.peek().is_some() {
                output.push_str(", ");
            }
        }

        output.push('}');
        output
    }
}
