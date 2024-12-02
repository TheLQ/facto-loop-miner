#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point2f {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
