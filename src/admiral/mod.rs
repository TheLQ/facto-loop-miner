use opencv::core::Point2f;

pub mod client;
mod err;
mod generators;
mod lua_command;

pub fn must_whole_number(point: Point2f) {
    let rounded = Point2f {
        x: point.x.round(),
        y: point.y.round(),
    };
    if rounded != point {
        panic!("Point is not round {:?}", rounded);
    }
}

pub fn must_odd_number(point: Point2f) {
    if point.x as i32 % 2 == 0 || point.y as i32 % 2 == 0 {
        panic!("Point is even {:?}", point);
    }
}

pub fn must_even_number(point: Point2f) {
    if point.x as i32 % 2 == 1 || point.y as i32 % 2 == 1 {
        panic!("Point is odd {:?}", point);
    }
}
