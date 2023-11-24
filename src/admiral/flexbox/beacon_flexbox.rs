use crate::admiral::flexbox::model::{FlexSize, Flexbox, FlexboxBuilder};

#[derive(Debug, Clone)]
struct BeaconFlexbox {
    size: FlexSize,
    cell_size: u32,
    // cell_content: Box<dyn FlexboxBuilder>,
}

impl FlexboxBuilder for BeaconFlexbox {
    fn make_flexbox(&self, siblings: &mut Vec<Flexbox>) {}
}
