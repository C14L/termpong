#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

extern crate itertools;

//use itertools::join;
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

const GAME_TICK_MILLIS: u64 = 10;
const FIELD_SIZE: u16 = XMAX * YMAX;
const PIXEL_EMPTY: u8 = b' ';
const PIXEL_SOLID: u8 = b'X';

struct Thing {
    xpos: u16,
    ypos: u16,
    size: u16,
    pixel: u8,
    xmov: f32,
    ymov: f32,
    xf32: f32,
    yf32: f32,
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

impl Thing {
    fn new(x: u16, y: u16, size: u16, pixel: u8) -> Self {
        Self {
            xpos: x,
            ypos: y,
            size: size,
            pixel: pixel,
            xmov: 0.0,
            ymov: 0.0,
            xf32: x as f32,
            yf32: y as f32,
        }
    }
}

struct Field {
    curr: [u8; FIELD_SIZE as usize],
}

impl Field {
    fn new() -> Self {
        Self {
            curr: [PIXEL_EMPTY; FIELD_SIZE as usize],
        }
    }
    fn clear(self: &mut Self) {
        for i in 0..self.curr.len() {
            let x: u16 = i as u16 % XMAX;
            let y: u16 = i as u16 / XMAX;
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
    fn get_idx(self: &Self, x: &u16, y: &u16) -> usize {
        (x + y * XMAX) as usize
    }
    fn set_pixel(self: &mut Self, x: u16, y: u16) {
        self.curr[self.get_idx(&x, &y)] = PIXEL_SOLID;
    }
    fn draw_thing(self: &mut Self, thing: &Thing) {
        let x = thing.xpos;
        for y in thing.ypos..(thing.ypos+thing.size) {
            self.curr[self.get_idx(&x, &y)] = thing.pixel;
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
    let mut move_things: bool;
    let mut init_ball: bool = false;
    let mut round_winner: u8 = 0;

    let mut field: Field = Field::new();
    let mut ball = Thing::new(XMAX / 2, YMAX / 2, 1, b'O');
    let mut player1 = Thing::new(XMIN + 5, (YMAX - YMIN) / 2, 4, b'X');
    let mut player2 = Thing::new(XMAX - 5, (YMAX - YMIN) / 2, 4, b'X');

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
        game_tick = time::Duration::from_millis(GAME_TICK_MILLIS);
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
            if event == Event::Key(KeyCode::Char(' ').into()) && !init_ball {
                // init ball
                init_ball = true;
                ball.xmov = 0.7;
                ball.ymov = -0.3;
            }
            if event == Event::Key(KeyCode::Char('a').into()) {
                if player1.get_ymin() > YMIN + 1 { player1.ypos -= 1; }
            }
            if event == Event::Key(KeyCode::Char('z').into()) {
                if player1.get_ymax() < YMAX - 1 { player1.ypos += 1; }
            }
            if event == Event::Key(KeyCode::Char('k').into()) {
                if player2.get_ymin() > YMIN + 1 { player2.ypos -= 1; }
            }
            if event == Event::Key(KeyCode::Char('m').into()) {
                if player2.get_ymax() < YMAX - 1 { player2.ypos += 1; }
            }
        }

        // Move ball

        if init_ball {
            ball.xf32 += ball.xmov;
            ball.yf32 += ball.ymov;

            ball.xpos = ball.xf32 as u16;
            ball.ypos = ball.yf32 as u16;

            if ball.xpos >= XMAX {
                round_winner = 1;
            }
            if ball.xpos <= XMIN {
                round_winner = 2;
            }
            if ball.get_ymin() <= XMIN || ball.get_ymax() >= XMAX {
                ball.ymov *= (-1.0)
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
        for i in 0..field.curr.len() {
            let x: u16 = i as u16 % XMAX;
            let y: u16 = i as u16 / XMAX;
            let c: char = field.curr[i] as char;

            stdout
                .execute(cursor::MoveTo(x, y))?
                .execute(Print(c))?
                .flush()?;
        }

        // Check if round is over and reset ball and count points

        if round_winner > 0 {
            round_winner = 0;
            init_ball = false;
            ball = Thing::new(XMAX / 2, YMAX / 2, 1, b'O');
        }
    }

    stdout.execute(cursor::Show)?.flush()?;
    let _ = terminal::disable_raw_mode()?;
    Ok(())
}


