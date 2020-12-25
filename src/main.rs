#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

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

const XMIN: u16 = 0;
const YMIN: u16 = 0;
const XMAX: u16 = 79;
const YMAX: u16 = 24;

const FIELD_SIZE: u16 = XMAX * YMAX;
const PIXEL_EMPTY: u8 = b' ';
const PIXEL_SOLID: u8 = b'X';

struct Thing {
    xpos: u16,
    ypos: u16,
    size: u16,
    speed: f32,
    accel: f32,
}

impl Thing {
    fn get_pos_idx(self: &Self) -> usize {
        (self.xpos + self.ypos * XMAX) as usize
    }
    fn get_ymin(self: &Self) -> u16 {
        self.ypos
    }
    fn get_ymax(self: &Self) -> u16 {
        self.ypos + self.size
    }
}

struct Field {
    curr: [u8; FIELD_SIZE as usize],
    prev: [u8; FIELD_SIZE as usize],
}

impl Field {
    fn new() -> Self {
        Self {
            curr: [PIXEL_EMPTY; FIELD_SIZE as usize],
            prev: [PIXEL_EMPTY; FIELD_SIZE as usize],
        }
    }
    fn clear(self: &mut Self) {
        for y in YMIN..YMAX {
            for x in XMIN..XMAX {
                let i = self.get_idx(&x, &y);
                let c: u8;
                if y == YMIN || y == YMAX-1 {
                    c = b'-';
                } else if x == (XMAX - XMIN) / 2 {
                    c = b'\'';
                } else if x == XMIN || x == XMAX-1 {
                    c = b'|';
                } else {
                    c = PIXEL_EMPTY;
                }
                self.curr[i as usize] = c;
            }
        }
    }
    fn get_idx(self: &Self, x: &u16, y: &u16) -> usize {
        (x + y * XMAX) as usize
    }
    fn set_pixel(self: &mut Self, x: u16, y: u16) {
        self.curr[self.get_idx(&x, &y)] = PIXEL_SOLID;
    }
    fn draw_thing(self: &mut Self, thing: &Thing) {
        let x = thing.xpos;
        for y in thing.ypos..(thing.ypos+thing.size) {
            self.curr[self.get_idx(&x, &y)] = PIXEL_SOLID;
        }
    }
}

fn main() -> CrosstermResult<()> {
    let mut rng = rand::thread_rng();
    let mut stdout = stdout();
    let mut tick_counter: usize = 0;
    let mut game_tick: time::Duration;
    let tick_threshold: usize = 5;
    let mut is_paused: bool = false;
    let mut move_things: bool = false;
    let mut field: Field = Field::new();

    let mut ball = Thing {
        xpos: XMAX / 2,
        ypos: YMAX / 2,
        size: 0,
        speed: 0.0,
        accel: 0.0,
    };
    let mut player1 = Thing {
        xpos: XMIN + 5,
        ypos: (YMAX - YMIN) / 2,
        size: 4,
        speed: 0.0,
        accel: 0.0,
    };
    let mut player2 = Thing {
        xpos: XMAX - 5,
        ypos: (YMAX - YMIN) / 2,
        size: 4,
        speed: 0.0,
        accel: 0.0,
    };

    // Init terminal

    let _ = terminal::enable_raw_mode()?;
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::Hide)?
        .execute(SetBackgroundColor(Color::Black))?
        .execute(SetForegroundColor(Color::White))?
        .flush()?;

    // Game loop

    'gameloop: loop {
        game_tick = time::Duration::from_millis(ball.speed as u64);
        tick_counter += 1;
        move_things = tick_counter == tick_threshold;

        if poll(game_tick)? {
            let event = read()?;

            if event == Event::Key(KeyCode::Esc.into()) {
                break 'gameloop;
            }
            if event == Event::Key(KeyCode::Char('p').into()) {
                is_paused = !is_paused;
            }
            if event == Event::Key(KeyCode::Char('a').into()) {
                if player1.get_ymin() > YMIN + 1 { player1.ypos -= 1; }
            }
            if event == Event::Key(KeyCode::Char('z').into()) {
                if player1.get_ymax() < YMAX - 1 { player1.ypos += 1; }
            }
            if event == Event::Key(KeyCode::Up.into()) {
                if player2.get_ymin() > YMIN + 1 { player2.ypos -= 1; }
            }
            if event == Event::Key(KeyCode::Down.into()) {
                if player2.get_ymax() < YMAX - 1 { player2.ypos += 1; }
            }
        }

        // Render all the Things

        if move_things {
            field.clear();
            field.draw_thing(&ball);
            field.draw_thing(&player1);
            field.draw_thing(&player2);
            tick_counter = 0;
        }

        // Paint Field to Canvas

        for y in YMIN..YMAX {
            for x in XMIN..XMAX {
                let i = x + y * XMAX;
                stdout
                    .execute(cursor::MoveTo(x, y))?
                    .execute(Print(field.curr[i as usize] as char))?
                    .flush()?;
            }
        }

    }

    stdout.execute(cursor::Show)?.flush()?;
    let _ = terminal::disable_raw_mode()?;
    Ok(())
}


