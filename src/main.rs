use std::thread;
use structopt::StructOpt;

mod game;
mod view;

#[derive(StructOpt)]
#[structopt(name = "Backroll test")]
struct Opts {
    /// Player numbers to run as.
    #[structopt(name = "player_numbers")]
    player_numbers: Vec<usize>,
}

fn main() {
    let opts = Opts::from_args();

    let threads = opts
        .player_numbers
        .iter()
        .map(|&player_number| {
            let name = format!("Player {}", player_number);
            thread::Builder::new()
                .name(name)
                .spawn(move || {
                    game::play(player_number);
                })
                .unwrap()
        })
        .collect::<Vec<_>>();

    for t in threads {
        t.join().unwrap();
    }
}
