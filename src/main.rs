use backroll::{
    transport::Peer, BackrollConfig, BackrollPlayer, BackrollPlayerHandle, P2PSessionBuilder,
    SessionCallbacks,
};
use bevy_tasks::TaskPool;
use bytemuck::{Pod, Zeroable};
use std::thread;

mod view;
use view::View;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Input {
    pub buttons: u8,
}
unsafe impl Zeroable for Input {}
unsafe impl Pod for Input {}

#[derive(Clone, Copy)]
struct PlayerState {
    pub y: u32,
}

#[derive(Clone)]
struct State {
    /// Player index, state
    pub players: [Option<PlayerState>; 2],
}

struct Config {}
impl BackrollConfig for Config {
    type Input = Input;
    type State = State;
    const MAX_PLAYERS_PER_MATCH: usize = 8;
    const RECOMMENDATION_INTERVAL: u32 = 333; // seems to be unused
}

#[derive(Clone)]
struct Player {
    pub handle: BackrollPlayerHandle,
    pub state: PlayerState,
}

struct Game {
    pub players: [Option<Player>; 2],
}

impl SessionCallbacks<Config> for Game {
    fn save_state(&mut self) -> (State, Option<u64>) {
        // Create State object from current game state
        //println!("save_state");
        let mut player_states = [None; 2];
        for (index, player_state) in self.players.iter().enumerate() {
            if let Some(p) = player_state {
                player_states[index] = Some(p.state);
            }
        }
        let state = State {
            players: player_states,
        };
        (state, None)
    }

    fn load_state(&mut self, state: State) {
        // Get game state from State object
        for (index, player_state) in state.players.iter().enumerate() {
            if let Some(player_state) = player_state {
                self.players[index].as_mut().unwrap().state = player_state.clone();
            }
        }
    }

    fn advance_frame(&mut self, input: backroll::GameInput<Input>) {
        for player in self.players.iter_mut().filter_map(|p| p.as_mut()) {
            let input = input.get(player.handle).unwrap();
            if input.buttons & 1 != 0 {
                player.state.y -= 1;
            }
            if input.buttons & 2 != 0 {
                player.state.y += 1;
            }
        }
    }

    fn handle_event(&mut self, event: backroll::BackrollEvent) {
        dbg!(event);
    }
}

fn play(local_player_number: usize) {
    let pool = TaskPool::new();

    let (local_host, remote_host) = if local_player_number == 0 {
        ("127.0.0.1:7000", "127.0.0.1:7001")
    } else {
        ("127.0.0.1:7001", "127.0.0.1:7000")
    };
    let connection_manager = UdpManager::bind(pool.clone(), local_host).unwrap();
    let connect_config = UdpConnectionConfig::unbounded(remote_host.parse().unwrap());
    let remote_peer = connection_manager.connect(connect_config);

    let mut builder = P2PSessionBuilder::<Config>::new();

    let mut game = Game {
        players: [None, None],
    };
    for player_number in 0usize..2 {
        let state = PlayerState { y: 10 };
        game.players[player_number] = if player_number == local_player_number {
            Some(Player {
                handle: builder.add_player(BackrollPlayer::Local),
                state,
            })
        } else {
            Some(Player {
                handle: builder.add_player(BackrollPlayer::Remote(remote_peer.clone())),
                state,
            })
        }
    }

    let session = builder.start(pool).unwrap();

    let mut view = View::new();

    loop {
        if session.is_synchronized() {
            session
                .add_local_input(
                    game.players[local_player_number].as_ref().unwrap().handle,
                    Input {
                        buttons: view.input(),
                    },
                )
                .unwrap_or_else(|e| {
                    println!("add_local_input failed: {}", e);
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

fn main() {
    let t0 = thread::spawn(|| {
        play(0);
    });
    let t1 = thread::spawn(|| {
        play(1);
    });
    t0.join().unwrap();
    t1.join().unwrap();
}
