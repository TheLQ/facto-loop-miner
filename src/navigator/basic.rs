use crate::surface::patch::Patch;
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use opencv::core::Point;

pub struct Navigator<'a> {
    pub surface: &'a Surface,
    pub end: PointU32,
    pub current: PointU32,
}

impl<'a> Navigator<'a> {
    pub fn start(&mut self) {}

    pub fn most_valuable_direction(&self) {
        let mut winning_direction = NavDirection::Up(0);
        for direction in NavDirection::all_directions(1) {
            if direction.score(self.current, self.end)
                > winning_direction.score(self.current, self.end)
            {
                winning_direction = direction;
            }
        }
        tracing::debug("most valuable direction {:?}", winning_direction)
    }
}

#[derive(Debug)]
enum NavDirection {
    Up(u32),
    Down(u32),
    Left(u32),
    Right(u32),
}

impl NavDirection {
    #[allow(dead_code)]
    fn step_size(&self) -> &u32 {
        match self {
            NavDirection::Up(step_size) => step_size,
            NavDirection::Down(step_size) => step_size,
            NavDirection::Left(step_size) => step_size,
            NavDirection::Right(step_size) => step_size,
        }
    }

    pub fn score(&self, start: PointU32, end: PointU32) -> i32 {
        let start_line = match self {
            NavDirection::Up(_) | NavDirection::Down(_) => start.y,
            NavDirection::Left(_) | NavDirection::Right(_) => start.x,
        };
        let end_line = match self {
            NavDirection::Up(_) | NavDirection::Down(_) => end.y,
            NavDirection::Left(_) | NavDirection::Right(_) => end.x,
        };
        let step: i32 = match *self {
            NavDirection::Up(size) | NavDirection::Right(size) => size as i32,
            NavDirection::Down(size) | NavDirection::Left(size) => -(size as i32),
        };
        let distance_before = end_line - start_line;
        let new_start: i32 = start_line as i32 + step;
        let distance_after: i32 = end_line as i32 - new_start;

        let good = distance_after < distance_before as i32;
        if good {
            distance_after as i32
        } else {
            -distance_after
        }
    }

    fn all_directions(step_size: u32) -> [NavDirection; 4] {
        [
            NavDirection::Up(step_size),
            NavDirection::Down(step_size),
            NavDirection::Left(step_size),
            NavDirection::Right(step_size),
        ]
    }
}

#[allow(dead_code)]
fn route_patch(surface: &mut Surface, patch: &Patch) {
    let mut offset = 0;
    loop {
        let pos = Point::new(patch.x + offset, patch.y);
        let existing = surface.get_pixel_point_i32(pos);
        match existing {
            &Pixel::Empty | &Pixel::Highlighter => {
                surface.set_pixel_point_i32(Pixel::Rail, pos);
            }
            existing => {
                tracing::debug("stopping at {} offset {}", existing.as_ref(), offset);
                break;
            }
        }
        offset = offset - 1;
    }
}
