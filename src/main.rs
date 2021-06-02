use std::thread;
use structopt::StructOpt;

mod game;
mod view;

#[derive(StructOpt)]
#[structopt(name = "Backroll test")]
struct Opts {
    #[structopt(short = "n", name = "num_players", default_value = "2")]
    num_players: usize,

    /// Player numbers to run as.
    #[structopt(name = "player_numbers", default_value = "0")]
    player_numbers: Vec<usize>,
}

fn main() {
    let opts = Opts::from_args();

    let num_players = opts.num_players;

    let threads = opts
        .player_numbers
        .iter()
        .map(|&player_number| {
            let name = format!("Player {}", player_number);
            thread::Builder::new()
                .name(name)
                .spawn(move || {
                    game::play(num_players, player_number);
                })
                .unwrap()
        })
        .collect::<Vec<_>>();

    for t in threads {
        t.join().unwrap();
    }
}
