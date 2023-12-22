use crate::surface::surface::PointU32;

#[derive(Debug)]
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

    pub fn contains_point_u32(&self, point: PointU32) -> bool {
        self.contains_point(point.x, point.y)
    }

    fn contains_point(&self, needle_x: u32, needle_y: u32) -> bool {
        let x = needle_x >= self.x && needle_x < self.end_x();
        let y = needle_y >= self.y && needle_y < self.end_y();
        x && y
    }
}

#[cfg(test)]
mod test {
    use crate::surface::easier_box::EasierBox;

    #[test]
    fn test() {
        let my_box = EasierBox {
            x: 2,
            y: 2,
            height: 3,
            width: 3,
        };

        assert!(!my_box.contains_point(1, 2));
        assert!(my_box.contains_point(2, 2));
        assert!(my_box.contains_point(3, 2));
        assert!(my_box.contains_point(4, 2));
        assert!(!my_box.contains_point(5, 2));

        assert!(!my_box.contains_point(2, 1));
        assert!(my_box.contains_point(2, 2));
        assert!(my_box.contains_point(2, 3));
        assert!(my_box.contains_point(2, 4));
        assert!(!my_box.contains_point(2, 5));

        // assert!(my_box.contains_point(3,2))
        // assert!(my_box.contains_point(4,2))
        // assert!(my_box.contains_point(5,2))
        // assert!(my_box.contains_point(,2))
    }
}
