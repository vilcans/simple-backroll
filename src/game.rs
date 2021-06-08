use backroll::{command::Command, Event, P2PSessionBuilder};
use backroll_transport_udp::{UdpConnectionConfig, UdpManager};
use bevy_tasks::TaskPool;
use bytemuck::{Pod, Zeroable};

use crate::view::View;

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, PartialEq, Eq, Debug)]
struct Input {
    pub buttons: u8,
}

#[derive(Clone, Copy, Hash)]
pub struct PlayerState {
    pub y: u32,
}

#[derive(Clone, Hash)]
struct State {
    pub players: Vec<PlayerState>,
}

struct Config {}
impl backroll::Config for Config {
    type Input = Input;
    type State = State;
}

#[derive(Clone)]
pub struct Player {
    pub handle: backroll::PlayerHandle,
    pub state: PlayerState,
    last_input: Input,
}

pub struct Game {
    pub players: Vec<Player>,
    pub time_sync_frames: u8,
}

impl Game {
    fn run_command(&mut self, command: Command<Config>) {
        match command {
            Command::Save(save_state) => {
                // println!("Save State");
                // Create State object from current game state
                let player_states = self.players.iter().map(|p| p.state).collect();
                let state = State {
                    players: player_states,
                };
                save_state.save(state);
            }
            Command::Load(load_state) => {
                // println!("Load State");
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
                // println!("Advance frame {}", inputs.frame);
                for player in self.players.iter_mut() {
                    let input = inputs.get(player.handle).unwrap();
                    if *input != player.last_input {
                        println!("{:?} Input for player {:?}: {:?} (Frame: {})", 
                            std::thread::current().id(), 
                            player.handle, 
                            input,
                            inputs.frame,
                        );
                    }
                    if input.buttons & 1 != 0 {
                        if player.state.y > 0 {
                            player.state.y -= 1;
                        }
                    }
                    if input.buttons & 2 != 0 {
                        player.state.y += 1;
                    }
                    player.last_input = *input;
                }
            }
            Command::Event(event) => {
                match event {
                    // this is to prevent jittering reset of game state
                    Event::Synchronized(remote) => {
                        println!("Synchronized with {:?}", remote);
                    },
                    Event::Running => {
                        println!("Synchronized with all remote players.");
                    },
                    Event::TimeSync { frames_ahead } => {
                        self.time_sync_frames = frames_ahead;
                        println!(
                            "We are {} frames ahead of the remotes. Stalling.",
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
            Player { handle, state, last_input: Zeroable::zeroed() }
        })
        .collect();
    let mut game = Game {
        players,
        time_sync_frames: 0,
    };

    let session = builder.start(pool).unwrap();
    for player in game.players.iter() {
        session.set_frame_delay(player.handle, 3);
    }

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
            println!("Time Sync: Stalling frame {}", game.time_sync_frames);
            continue;
        }

        match session.add_local_input(
            game.players[local_player_number].handle,
            Input {
                buttons: view.input(),
            },
        ) {
            Ok(()) => {}
            Err(err) => {
                // println!("add_local_input failed: {:?}", err);
                continue;
            }
        }

        for command in session.advance_frame() {
            game.run_command(command);
        }

        if !view.update(&game) {
            break;
        }
    }
}
