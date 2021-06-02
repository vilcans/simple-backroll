extern crate minifb;

use minifb::{Key, Window, WindowOptions};

use crate::{Game, Player};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

pub struct View {
    buffer: Vec<u32>,
    window: Window,
}

impl View {
    pub fn new() -> Self {
        let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
        buffer.fill(0x808080);

        let window = Window::new(
            "Test - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        // Limit to max ~60 fps update rate
        //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Self { buffer, window }
    }

    pub(crate) fn input(&self) -> u8 {
        let mut input = 0u8;
        if self.window.is_key_down(Key::Up) {
            input |= 1;
        }
        if self.window.is_key_down(Key::Down) {
            input |= 2;
        }
        input
    }

    pub(crate) fn update(&mut self, game: &Game) -> bool {
        if !self.window.is_open() || self.window.is_key_down(Key::Escape) {
            return false;
        }

        self.buffer.fill(0x888888);

        for (index, p) in game.players.iter().enumerate() {
            if let Some(Player { state, .. }) = p {
                let x = index * 20 + 10;
                let y = state.y;
                for yoffs in 0..10 {
                    let i = x + (y as usize + yoffs) * WIDTH as usize;
                    self.buffer[i..i + 10].fill(0xffff00);
                }
            }
        }

        self.window
            .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
            .unwrap();

        true
    }
}
