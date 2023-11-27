use itertools::Itertools;
use opencv::core::Point;
use thiserror::Error;

pub type VResult<R> = Result<R, VError>;

#[derive(Error, Debug)]
pub enum VError {
    #[error("XYOutOfBounds positions {}", positions_to_strings(positions))]
    XYOutOfBounds { positions: Vec<Point> },
}

fn positions_to_strings(positions: &[Point]) -> String {
    positions.iter().map(|e| format!("{:?}", e)).join(",")
}
