use crate::surface::easier_box::EasierBox;

/// Inspired by CSS Flexbox
///
/// Define base as nested containers.
/// Lua commands xy coordinates are unsigned, relative to 0,0
pub struct Flexbox {
    site: FlexSize,
    children: Vec<Flexbox>,
}

#[derive(Debug, Clone)]
pub struct FlexSize {
    width: u32,
    height: u32,
}

pub trait FlexboxBuilder: Clone {
    fn make_flexbox(&self, siblings: &mut Vec<Flexbox>);
}
