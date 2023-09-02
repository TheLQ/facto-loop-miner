use kiddo::KdTree;
use num_format::Locale;

mod gamedata;
mod opencv;
pub mod state;
pub mod surface;

pub type PixelKdTree = KdTree<f32, 2>;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;
