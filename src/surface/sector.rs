use crate::surface::easier_box::EasierBox;
use crate::surface::surface::Surface;

enum SectorCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

enum SectorSide {
    Top,
    Bottom,
    Left,
    Right,
}

struct SectorLookup {
    center: EasierBox,
    surface_width: u32,
    surface_height: u32,
}

impl SectorLookup {
    pub fn new_from_surface(surface: &Surface, center_tiles: u32) -> Self {
        let center_corner_x: u32 = surface.area_box.game_centered_x_i32(-(center_tiles as i32));
        let center_corner_y: u32 = surface.area_box.game_centered_y_i32(-(center_tiles as i32));
        let center_corner_x_end: u32 = surface.area_box.game_centered_x_i32(center_tiles as i32);
        let center_corner_y_end: u32 = surface.area_box.game_centered_y_i32(center_tiles as i32);

        SectorLookup {
            surface_width: surface.width,
            surface_height: surface.height,
            center: EasierBox {
                x: center_corner_x,
                y: center_corner_y,
                width: center_corner_x_end - center_corner_x,
                height: center_corner_y_end - center_corner_y,
            },
        }
    }

    fn get_side_box(&self, side: SectorSide) -> EasierBox {
        match side {
            SectorSide::Left => EasierBox {
                x: 0,
                width: self.center.x,
                y: self.center.y,
                height: self.center.height,
            },
            SectorSide::Right => EasierBox {
                x: self.center.end_x(),
                width: self.surface_width - self.center.end_x(),
                y: self.center.y,
                height: self.center.height,
            },
            SectorSide::Top => EasierBox {
                x: self.center.x,
                width: self.center.width,
                y: 0,
                height: self.center.y,
            },
            SectorSide::Bottom => EasierBox {
                x: self.center.x,
                width: self.center.width,
                y: self.center.end_y(),
                height: self.surface_height - self.center.end_y(),
            },
        }
    }

    fn get_corner_box(&self, corner: SectorCorner) -> EasierBox {
        match corner {
            SectorCorner::TopLeft => EasierBox {
                x: 0,
                width: self.center.x,
                y: 0,
                height: self.center.y,
            },
            SectorCorner::TopRight => EasierBox {
                x: self.center.end_x(),
                width: self.surface_width - self.center.end_x(),
                y: 0,
                height: self.center.y,
            },
            SectorCorner::BottomLeft => EasierBox {
                x: 0,
                width: self.center.x,
                y: self.center.end_y(),
                height: self.surface_height - self.center.end_y(),
            },
            SectorCorner::BottomRight => EasierBox {
                x: self.center.end_x(),
                width: self.surface_width - self.center.end_x(),
                y: self.center.end_y(),
                height: self.surface_height - self.center.end_y(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::surface::game_locator::GameLocator;
    use crate::surface::surface::Surface;

    #[test]
    fn test() {
        // let surface = Surface::new(10, 10 - 1);
        // surface.area_box = GameLocator {
        //     min_x: -5,
        //     max_x: 5,
        //     min_y: -5,
        //     max_y: 5,
        //     width: 10,
        //     height: 10,
        // };
    }
}
