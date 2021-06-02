use backroll::{
    BackrollConfig, BackrollPlayer, BackrollPlayerHandle, BackrollResult, P2PSessionBuilder,
    SessionCallbacks,
};
use bevy_tasks::TaskPool;
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, PartialEq, Eq)]
struct Input {}
unsafe impl Zeroable for Input {}
unsafe impl Pod for Input {}

#[derive(Clone)]
struct State {}

struct Config {}
impl BackrollConfig for Config {
    type Input = Input;
    type State = State;
    const MAX_PLAYERS_PER_MATCH: usize = 8;
    const RECOMMENDATION_INTERVAL: u32 = 333; // seems to be unused
}

struct Game {
    pub local_player_handle: BackrollPlayerHandle,
}

impl SessionCallbacks<Config> for Game {
    fn save_state(&mut self) -> (State, Option<u64>) {
        // Create State object from current game state
        let state = State {};
        let hash = 0u64;
        (state, Some(hash))
    }

    fn load_state(&mut self, state: State) {
        // Get game state from State object
    }

    fn advance_frame(&mut self, input: backroll::GameInput<Input>) {
        let _input = input.get(self.local_player_handle).unwrap();
    }

    fn handle_event(&mut self, event: backroll::BackrollEvent) {
        dbg!(event);
    }
}

fn main() -> BackrollResult<()> {
    let mut builder = P2PSessionBuilder::<Config>::new();
    let local_player_handle = builder.add_player(BackrollPlayer::Local);
    dbg!(local_player_handle);
    let pool = TaskPool::new();
    let session = builder.start(pool).unwrap();
    let mut game = Game {
        local_player_handle,
    };
    loop {
        session.add_local_input(local_player_handle, Input {})?;
        session.advance_frame(&mut game);
    }
}
