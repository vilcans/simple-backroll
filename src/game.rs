use backroll::{Command, Event, P2PSessionBuilder};
use backroll_transport_udp::{UdpConnectionConfig, UdpManager};
use bevy_tasks::TaskPool;
use bytemuck::{Pod, Zeroable};

use crate::view::View;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Input {
    pub buttons: u8,
}
unsafe impl Zeroable for Input {}
unsafe impl Pod for Input {}

#[derive(Clone, Copy)]
pub struct PlayerState {
    pub y: u32,
}

#[derive(Clone)]
struct State {
    pub players: Vec<PlayerState>,
}

struct Config {}
impl backroll::Config for Config {
    type Input = Input;
    type State = State;
    const MAX_PLAYERS_PER_MATCH: usize = 8;
    const RECOMMENDATION_INTERVAL: u32 = 333; // seems to be unused
}

#[derive(Clone)]
pub struct Player {
    pub handle: backroll::PlayerHandle,
    pub state: PlayerState,
}

pub struct Game {
    pub players: Vec<Player>,
    pub time_sync_frames: u8,
}

impl Game {
    fn run_command(&mut self, command: backroll::Command<Config>) {
        match command {
            Command::Save(save_state) => {
                // Create State object from current game state
                //println!("save_state");
                let player_states = self.players.iter().map(|p| p.state).collect();
                let state = State {
                    players: player_states,
                };
                save_state.save(state, None);
            }
            Command::Load(load_state) => {
                // Get game state from State object
                for (s, d) in load_state
                    .load()
                    .players
                    .iter()
                    .zip(self.players.iter_mut())
                {
                    d.state = s.clone();
                }
            }
            Command::AdvanceFrame(inputs) => {
                for player in self.players.iter_mut() {
                    let input = inputs.get(player.handle).unwrap();
                    if input.buttons & 1 != 0 {
                        if player.state.y > 0 {
                            player.state.y -= 1;
                        }
                    }
                    if input.buttons & 2 != 0 {
                        player.state.y += 1;
                    }
                }
            }
            Command::Event(event) => {
                match event {
                    // this is to prevent jittering reset of game state
                    Event::TimeSync { frames_ahead } => {
                        self.time_sync_frames = frames_ahead;
                        println!(
                            "Time syncs frames are {} frames ahead",
                            self.time_sync_frames
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

fn host_for_player(player_number: usize) -> String {
    format!("127.0.0.1:{}", 7000 + player_number)
}

/*
    TODO: Whenever there is a ReachedPredictionBarrier error, it leads to a desync. Current guess is that inputs from remote player are skipped.
*/

pub fn play(num_players: usize, local_player_number: usize) {
    let pool = TaskPool::new();

    let local_host = host_for_player(local_player_number);
    let connection_manager = UdpManager::bind(pool.clone(), local_host).unwrap();

    let mut builder = P2PSessionBuilder::<Config>::new();

    let players = (0usize..num_players)
        .map(|player_number| {
            let state = PlayerState { y: 10 };
            let handle = if player_number == local_player_number {
                builder.add_player(backroll::Player::Local)
            } else {
                let connect_config =
                    UdpConnectionConfig::unbounded(host_for_player(player_number).parse().unwrap());
                let remote_peer = connection_manager.connect(connect_config);
                builder.add_player(backroll::Player::Remote(remote_peer))
            };
            Player { handle, state }
        })
        .collect();
    let mut game = Game {
        players,
        time_sync_frames: 0,
    };

    let session = builder.start(pool).unwrap();

    let mut view = View::new(&format!("Player {}", local_player_number));

    loop {
        for command in session.poll() {
            game.run_command(command);
        }
        if game.time_sync_frames > 0 {
            if !view.update(&game) {
                break;
            }
            game.time_sync_frames -= 1;
            continue;
        }
        if session.is_synchronized() {
            println!("Session is synchronized. Adding input.");
            /*.map(|_| println!("add_local_input succeeded"))
            .unwrap_or_else(|e| {
                println!("add_local_input failed: {:?}", e);
            });
            */
            match session.add_local_input(
                game.players[local_player_number].handle,
                Input {
                    buttons: view.input(),
                },
            ) {
                Ok(()) => {}
                Err(err) => {
                    println!("add_local_input failed: {:?}", err);
                    continue;
                }
            }
        } else {
            println!("Not synchronized yet");
        }
        for command in session.advance_frame() {
            game.run_command(command);
        }
        if !view.update(&game) {
            break;
        }
    }
}
