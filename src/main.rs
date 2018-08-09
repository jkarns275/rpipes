extern crate ctrlc;
extern crate rand;
extern crate clap;
extern crate pancurses;

use pancurses::*;
use clap::{ Arg, App, SubCommand };

use rand::Rng;

use std::io::Write;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::char;

use Direction::*;
use Color::*;

/// Maps the last direction and the current direction to an index in the pipe character array.
const PRINT_MAP: [[usize; 4]; 4] = [[0, 2, 0, 3],
                                    [4, 1, 3, 1],
                                    [0, 5, 0, 4],
                                    [5, 1, 2, 1]];

const COLORSETS: [[Color; 6]; 3] = [[White, Red, White, Yellow, Cyan, Magenta],
                                    [Cyan, Magenta, Blue, Yellow, Green, Red],
                                    [White, Blue, Cyan, Magenta, Green, White]];

const CHARSETS: [[&'static str; 6]; 1] = [["┃", "━", "┏", "┓", "┛", "┗"]];
const DEFAULT_CHARSET: usize = 1;
const DEFAULT_N_PIPES: usize = 2;
const DEFAULT_COLOR_MODE: usize = 0;

#[derive(Copy, Clone)]
#[repr(u8)]
enum Color {
    White   = 7u8,
    Cyan    = 6u8,
    Magenta = 5u8,
    Blue    = 4u8,
    Yellow  = 3u8,
    Green   = 2u8,
    Red     = 1u8,
    Black   = 0u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
enum Direction {
    Up      = 0usize,
    Right   = 1usize,
    Down    = 2usize,
    Left    = 3usize,
}

impl Direction {
    pub fn turn(self, rng: &mut rand::StdRng) -> Self {
        let rb = rng.gen::<bool>();

        match self {
            Up | Down => if rb { Left } else { Right }
            Left | Right => if rb { Up } else { Down }
        }
    }
}

#[derive(Copy, Clone)]
struct Position {
    pub x: u32,
    pub y: u32
}

impl Position {
    pub fn new(x: u32, y: u32) -> Position {
        Position { x, y }
    }
}

#[derive(Clone, Copy)]
struct Pipe {
    pub pos: Position,
    pub dir: Direction,
    pub last_dir: Direction,
    pub rng: rand::StdRng,
    /// How much many squares this pipe should go before it changes direction
    pub track_len: u32,
    pub color: Color,
    /// Maximum allowed track length.
    pub max_track_len: u32,
    pub colorset: usize,
    pub charset: usize,
    pub rows: u32,
    pub cols: u32,
    pub min_track_len: u32,
}

impl Pipe {
    const DIRS: [Direction; 4] = [Up, Down, Left, Right];
    
    pub fn new(colorset: usize, charset: usize, min_track_len: u32, max_track_len: u32, cols: u32, rows: u32) -> Pipe {
        let pos = Position::new(0, 0);
        let mut rng = rand::StdRng::new().unwrap();
        
        let dir = Pipe::DIRS[rng.gen::<usize>() & 3];        

        let last_dir = dir.clone();

        let mut pipe = Pipe {   pos, rng, dir, last_dir, track_len: 4, color: White, max_track_len,
                                min_track_len, rows, cols, charset, colorset };
        pipe.reset();
        pipe
    }

    pub fn set_random_color(&mut self) {
        let rusize = self.rng.gen::<usize>() % 6;
        self.color = COLORSETS[self.colorset][rusize];
    }

    pub fn reset(&mut self) {
        self.set_random_color();
        let r = self.rng.gen::<usize>() & 3;
        
        self.dir = Pipe::DIRS[r];
        self.last_dir = self.dir;
        
        match r {
            2 => self.pos.x = self.cols - 1,
            0 => self.pos.y = self.rows - 1,
            3 => self.pos.x = 0,
            1 => self.pos.y = 0,
            _ => unreachable!()
        };

        if r < 2 {
            self.pos.x = self.rng.gen::<u32>() % self.cols;
        } else {
            self.pos.y = self.rng.gen::<u32>() % self.rows;
        }

        self.set_track_len();
        self.set_random_color();
    }

    pub fn set_track_len(&mut self) {
        self.track_len = self.min_track_len + (self.rng.gen::<u32>() % (self.max_track_len - self.min_track_len));
    }

    pub fn turn_pipe(&mut self) {
        let rb = self.rng.gen::<bool>();
        self.last_dir = self.dir;
        self.dir = match self.dir {
            Up | Down => if rb { Left } else { Right },
            Left | Right => if rb { Up } else { Down }
        };

        self.set_track_len();
    }

    pub fn update(&mut self) {
        self.track_len -= 1;
        self.last_dir = self.dir;
        if self.track_len == 0 {
            self.turn_pipe();
        } else {
            if self.pos.x == 0 && self.dir == Direction::Left {
                self.pos.x = self.cols;
            } else if self.pos.x == self.cols && self.dir == Direction::Right {
                self.pos.x = 0;
            }
            if self.pos.y == 0 && self.dir == Direction::Up {
                self.pos.y = self.rows;
            } else if self.pos.y == self.rows && self.dir == Direction::Down {
                self.pos.y = 0;
            }
            /*if  self.pos.x <= 0 || self.pos.x >= self.cols ||
                self.pos.y <= 0 || self.pos.y >= self.rows {
                self.reset()
            }*/

            match self.dir {
                Up => self.pos.y -= 1,
                Down => self.pos.y += 1,
                Left => self.pos.x -= 1,
                Right => self.pos.x += 1,
            };
        }
    }

    pub fn print(&mut self, window: &Window) {
        let ind = PRINT_MAP[self.last_dir as usize][self.dir as usize];
        let ch = CHARSETS[self.charset][ind];
        set_color(self.color, window);
        window.mvaddstr(self.pos.y as i32, self.pos.x as i32, ch);
    }
}

fn clear_screen(window: &Window) {
    window.erase();
    doupdate();
}

fn move_cursor(pos: Position, window: &Window) {
    window.mv(pos.x as i32, pos.y as i32);
}
fn set_color(color: Color, window: &Window) {
    window.color_set(color as u8 as i16);
}

fn safe_exit() -> ! {
    endwin();
    std::process::exit(0);
}

static mut READY_TO_EXIT: bool = false;

fn main() {
    ctrlc::set_handler(move || {
        unsafe { READY_TO_EXIT = true };
    });

    let matches = App::new("ripes")
        .version("1.0")
        .author("Joshua Karns")
        .about("Prints moving pipes in the terminal")
        .arg(
            Arg::with_name("colorset")
                .short("s")
                .long("colorset")
                .value_name("colorset")
                .help("Sets the color set to be used"))
        .arg(
            Arg::with_name("charset")
                .short("c")
                .long("charset")
                .value_name("charset")
                .help("Sets the character set to be used"))
        .arg(
            Arg::with_name("max_len")
                .short("M")
                .long("max_len")
                .value_name("max_len")
                .help("The maximum length a pipe will be before it turns.")
        )
        .arg(
            Arg::with_name("min_len")
                .short("m")
                .long("min_len")
                .value_name("min_len")
                .help("The minimum length of a pipe before it turns.")
        )
        .arg(
            Arg::with_name("numpipes")
                .short("n")
                .long("numpipes")
                .value_name("numpipes")
                .help("The number of pipes to be drawn at the same time.")
        )
        .arg(
            Arg::with_name("delay")
                .short("d")
                .long("delay")
                .value_name("delay")
                .help("the delay between updates (ms).")
        )
        .get_matches();
    
    let colorset_string = matches.value_of("colorset").unwrap_or("0");
    let colorset = match colorset_string.parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("{} is not a valid colorset, 0 - {} are valid. Defaulting to 0.", &colorset_string, COLORSETS.len());
            0
        }
    };

    let charset_string = matches.value_of("charset").unwrap_or("0");
    let charset = match charset_string.parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("{} is not a valid charset, 0 - {} are valid. Defaulting to 0.", &charset_string, CHARSETS.len());
            0
        }
    };

    let max_len_string = matches.value_of("max_len").unwrap_or("7");
    let max_len = match max_len_string.parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            println!("{} is not a valid positive integer. Defaulting max_len to 7", max_len_string);
            7
        }
    };

    let min_len_string = matches.value_of("min_len").unwrap_or("4");
    let min_len = match min_len_string.parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            println!("{} is not a valid positive integer. Defaulting min_len to 4", max_len_string);
            4
        }
    };
    
    let numpipes_string = matches.value_of("numpipes").unwrap_or("1");
    let numpipes = match numpipes_string.parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("{} is not a valid positive integer > 1. Defaulting numpipes to 1", numpipes_string);
            1
        }
    };

    let delay_string = matches.value_of("delay").unwrap_or("16");
    let delay = match delay_string.parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            println!("{} is not a valid positive integer > 1. Defaulting delay to 1", delay_string);
            1
        }
    };

    if charset != 0 {
        println!("There is only one charset right now.");
        return;
    }
   
    if min_len >= max_len {
        
        println!("\nMinimum length must be less than the maximum length ({} >= {})", min_len, max_len);
        return;
    }

    if colorset > 2 {
        println!("There are only 3 supported color sets right now! Please specify a color set between 0 and 2.");
        return;
    }

    if numpipes == 0 {
        println!("No pipes?");
        return
    }

    let window = initscr();

    let (height, width) = window.get_max_yx();
 
    let mut pipes = vec![];
    for _ in 0..numpipes {
        pipes.push(Pipe::new(colorset, charset, min_len, max_len, width as u32, height as u32));
    }

    clear_screen(&window);
    start_color();
    use_default_colors();

    for i in 0..8 {
        init_pair(i + 1, i, -1);
    }

    use std::time::{Duration, Instant};
    while !unsafe { READY_TO_EXIT } {
        let now = Instant::now();
        pipes.iter_mut().map(|pipe| { pipe.update(); pipe.print(&window) }).count();
        window.refresh();
        let elapsed = now.elapsed();
        let ms = elapsed.as_secs() as u32 * 1000 + elapsed.subsec_millis();

        if ms < delay { std::thread::sleep_ms(delay - ms); }
    }

    window.erase();
    doupdate();
    endwin();
    println!("Goodbye!");
}
