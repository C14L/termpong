extern crate itertools;

use itertools::join;
use rand::{prelude::ThreadRng, Rng};
use std::io::{stdout, Write};
use std::time;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal,
    ExecutableCommand,
    Result as CrosstermResult,
};


fn main() -> CrosstermResult<()> {
    let _ = terminal::enable_raw_mode()?;
    let mut rng = rand::thread_rng();
    let mut stdout = stdout();

    let mut ball_speed: usize = 250;
    let mut tick_counter: usize = 0;
    let mut game_tick: time::Duration;
    let mut is_paused: bool = false;

    // Init terminal

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(SetBackgroundColor(Color::Black))?
        .execute(SetForegroundColor(Color::White))?
        .flush()?;

    // Game loop

    'gameloop: loop {
        game_tick = time::Duration::from_millis(ball_speed as u64);
        tick_counter += 1;

        if poll(game_tick)? {
            let event = read()?;

            if event == Event::Key(KeyCode::Esc.into()) {
                break 'gameloop;
            }

        }
    }

    let _ = terminal::disable_raw_mode()?;
    Ok(())
}
