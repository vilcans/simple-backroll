use std::thread;

mod game;
mod view;

fn main() {
    let t0 = thread::spawn(|| {
        game::play(0);
    });
    let t1 = thread::spawn(|| {
        game::play(1);
    });
    t0.join().unwrap();
    t1.join().unwrap();
}
