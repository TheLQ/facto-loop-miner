use opencv::core::Point2f;

pub mod client;
mod err;
pub mod executor;
mod flexbox;
mod generators;
mod lua_command;

pub fn must_whole_number(point: Point2f) {
    let rounded = Point2f {
        x: point.x.round(),
        y: point.y.round(),
    };
    assert!(!(rounded != point), "Point is not round {:?}", rounded);
}

pub fn must_odd_number(point: Point2f) {
    assert!(
        !(point.x as i32 % 2 == 0 || point.y as i32 % 2 == 0),
        "Point is even {:?}",
        point
    );
}

pub fn must_even_number(point: Point2f) {
    assert!(
        !(point.x as i32 % 2 == 1 || point.y as i32 % 2 == 1),
        "Point is odd {:?}",
        point
    );
}

pub fn must_half_number(point: Point2f) {
    let dec_x = point.x.floor() - point.x;
    let dec_y = point.y.floor() - point.y;
    assert!(
        !(dec_x > 0.4 && dec_x < 0.6 && dec_y > 0.4 && dec_y < 0.6),
        "Point isn't half {:?}",
        point
    );
}
