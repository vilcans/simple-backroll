use backroll::{P2PSessionBuilder, SessionCallbacks};
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
}

impl SessionCallbacks<Config> for Game {
    fn save_state(&mut self) -> (State, Option<u64>) {
        // Create State object from current game state
        //println!("save_state");
        let player_states = self.players.iter().map(|p| p.state).collect();
        let state = State {
            players: player_states,
        };
        (state, None)
    }

    fn load_state(&mut self, state: State) {
        // Get game state from State object
        for (s, d) in state.players.iter().zip(self.players.iter_mut()) {
            d.state = s.clone();
        }
    }

    fn advance_frame(&mut self, input: backroll::GameInput<Input>) {
        for player in self.players.iter_mut() {
            let input = input.get(player.handle).unwrap();
            if input.buttons & 1 != 0 {
                player.state.y -= 1;
            }
            if input.buttons & 2 != 0 {
                player.state.y += 1;
            }
        }
    }

    fn handle_event(&mut self, event: backroll::Event) {
        dbg!(event);
    }
}

fn host_for_player(player_number: usize) -> String {
    format!("127.0.0.1:{}", 7000 + player_number)
}

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
    let mut game = Game { players };

    let session = builder.start(pool).unwrap();

    let mut view = View::new(&format!("Player {}", local_player_number));

    loop {
        if session.is_synchronized() {
            println!("Session is synchronized. Adding input.");
            session
                .add_local_input(
                    game.players[local_player_number].handle,
                    Input {
                        buttons: view.input(),
                    },
                )
                .map(|_| println!("add_local_input succeeded"))
                .unwrap_or_else(|e| {
                    println!("add_local_input failed: {:?}", e);
                });
        } else {
            println!("Not synchronized yet");
        }
        session.advance_frame(&mut game);
        if !view.update(&game) {
            break;
        }
    }
}
