use crate::surface::surface::PointU32;

pub struct EasierBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl EasierBox {
    pub fn end_x(&self) -> u32 {
        self.x + self.width
    }

    pub fn end_y(&self) -> u32 {
        self.y + self.height
    }

    fn contains_point_u32(&self, point: PointU32) -> bool {
        self.contains_point(point.x, point.y)
    }

    fn contains_point(&self, needle_x: u32, needle_y: u32) -> bool {
        let x = needle_x > self.x && needle_x < self.end_x();
        let y = needle_y > self.x && needle_y < self.end_y();
        x && y
    }
}
