use std::time::Duration;

fn main() {
    println!("spinspinner spinning");
    for i in 0..42 {
        std::thread::spawn(move || {
            println!("spawn {i}");
            loop {
                let mut test: i128 = 44;
                for i in 0..99 {
                    test = test.pow(i)
                }
                std::hint::black_box(test);
            }
        });
    }
    std::thread::sleep(Duration::from_millis(9999 * 1000));
}
