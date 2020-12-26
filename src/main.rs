#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

// TODO: Change ball angle when ball hits corner of player
// TODO: Change ball angle when player moves while hitting the ball
// TODO: Allow players to move forwards and backwards

extern crate itertools;

use std::str;
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
        for y in thing.get_ymin()..thing.get_ymax() {
            self.curr[self.get_idx(&x, &y)] = thing.pixel;
        }
    }
    fn write(self: &mut Self, x: u16, y: u16, text: &str) {
       let i = self.get_idx(&x, &y);
       text.as_bytes().iter().enumerate().for_each(|(j, c)| { self.curr[i+j] = *c; });
    }
}

fn main() -> CrosstermResult<()> {
    let mut rng = rand::thread_rng();
    let mut stdout = stdout();

    let mut tick_counter: usize = 0;
    let mut game_tick: time::Duration;
    let tick_threshold: usize = 2;

    let mut is_paused: bool = false;
    let mut move_things: bool;
    let mut init_ball: bool = false;
    let mut round_winner: usize = 0;
    let mut score: [usize; 2] = [0, 0];
    let mut ball_step: f32 = 0.5;
    let mut rand_angle: f32;

    let mut field: Field = Field::new();
    let mut ball = Thing::new(XMAX / 2, YMAX / 2, 1, b'O');
    let mut player1 = Thing::new(XMIN + 5, (YMAX - YMIN) / 2 - 1, 4, b'X');
    let mut player2 = Thing::new(XMAX - 5, (YMAX - YMIN) / 2 - 1, 4, b'X');
    let mut debugstr: String = String::from("debug on");
    let mut show_debug: bool = false;
    let mut show_help: bool = false;

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

            if event == Event::Key(KeyCode::Char('h').into()) {
                show_help = !show_help;
            }
            if event == Event::Key(KeyCode::Char('d').into()) {
                show_debug = !show_debug;
            }

            if !init_ball {
                if event == Event::Key(KeyCode::Char('9').into()) {
                    ball_step = 0.9;
                } else if event == Event::Key(KeyCode::Char('8').into()) {
                    ball_step = 0.8;
                } else if event == Event::Key(KeyCode::Char('7').into()) {
                    ball_step = 0.7;
                } else if event == Event::Key(KeyCode::Char('6').into()) {
                    ball_step = 0.6;
                } else if event == Event::Key(KeyCode::Char('5').into()) {
                    ball_step = 0.5;
                } else if event == Event::Key(KeyCode::Char('4').into()) {
                    ball_step = 0.4;
                } else if event == Event::Key(KeyCode::Char('3').into()) {
                    ball_step = 0.3;
                } else if event == Event::Key(KeyCode::Char('2').into()) {
                    ball_step = 0.2;
                } else if event == Event::Key(KeyCode::Char('1').into()) {
                    ball_step = 0.1;
                }
            }

            if event == Event::Key(KeyCode::Char('p').into()) {
                is_paused = !is_paused;
            }

            if event == Event::Key(KeyCode::Char(' ').into()) {
                if init_ball {
                    init_ball = false;
                    ball = Thing::new(XMAX / 2, YMAX / 2, 1, b'O');
                } else {
                    init_ball = true;
                    rand_angle = rng.gen_range(-45, 45) as f32;
                    ball.xmov = rand_angle.cos() * ball_step;
                    ball.ymov = rand_angle.sin() * ball_step;
                    debugstr = format!("angle={:02} x={:.03} y={:.03}", rand_angle, ball.xmov, ball.ymov);
                }
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

        // Move ball and find bounces

        if init_ball {
            ball.xf32 += ball.xmov;
            ball.yf32 += ball.ymov;

            ball.xpos = ball.xf32 as u16;
            ball.ypos = ball.yf32 as u16;

            // with walls

            if ball.xpos >= XMAX {
                round_winner = 1;
            }
            if ball.xpos <= XMIN {
                round_winner = 2;
            }
            if ball.get_ymin() <= YMIN || ball.get_ymax() >= YMAX {
                ball.ymov *= -1.0
            }

            // with players

            if (
                ball.xpos == player1.xpos
                && ball.ypos >= player1.get_ymin()
                && ball.ypos <= player1.get_ymax()
            ) || (
                ball.xpos == player2.xpos
                && ball.ypos >= player2.get_ymin()
                && ball.ypos <= player2.get_ymax()
            ) {
                ball.xmov *= -1.0
            }

            // avoid overflows

            if ball.yf32 > YMAX as f32 {
                ball.yf32 = YMAX as f32 - 1.0;
                ball.ypos = YMAX - 1;
            }
        }

        // Render all the Things

        if move_things {
            field.clear();

            if show_debug {
                field.write(3, 2, &debugstr);
            }

            if show_help {
                field.write(3, YMIN, format!(" speed={:1} [1-9] ", (ball_step * 10.0) as u8).as_str());
                field.write(2, YMAX - 1, " a=up - z=down ");
                field.write(XMAX - 17, YMAX - 1, " k=up - m=down ");
                field.write(XMAX / 2 - 9, YMAX - 1, " space=start/reset ");
            } else {
                field.write(XMIN + 2, YMIN, " [h]elp ");
            }

            field.write(XMAX / 2 - 4, YMIN, format!(" {:02} ", score[0]).as_str());
            field.write(XMAX / 2 + 2, YMIN, format!(" {:02} ", score[1]).as_str());

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
            score[round_winner - 1] += 1;
            round_winner = 0;
            init_ball = false;
            ball = Thing::new(XMAX / 2, YMAX / 2, 1, b'O');
        }
    }

    stdout.execute(cursor::Show)?.flush()?;
    let _ = terminal::disable_raw_mode()?;
    Ok(())
}


