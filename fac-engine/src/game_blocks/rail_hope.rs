pub trait RailHopeAppender {
    fn add_straight(&mut self, length: usize);

    fn add_turn90(&mut self, clockwise: bool);

    fn add_shift45(&mut self, clockwise: bool, length: usize);
}

pub trait RailHopeAppenderExt<R> {
    fn add_straight(&self, length: usize) -> R;

    fn add_turn90(&self, clockwise: bool) -> R;

    fn add_shift45(&self, clockwise: bool, length: usize) -> R;
}

// pub trait RailHopeAppenderExt<S, R> {
//     fn add_straight(&mut self, state: S, length: usize) -> R;
//
//     fn add_turn90(&mut self, state: S, clockwise: bool) -> R;
//
//     fn add_shift45(&mut self, state: S, clockwise: bool, length: usize) -> R;
// }
