pub mod cached_renderer;
pub mod debug_renderer;

use std::{
    collections::HashMap,
    io::{self, Write},
};

use crossterm::style::Color;
use tetrs_engine::{FeedbackEvents, Game, Tetromino, TileTypeID};

use crate::terminal_user_interface::{Application, GraphicsColor, RunningGameStats};

pub trait Renderer {
    fn render<T>(
        &mut self,
        app: &mut Application<T>,
        running_game_stats: &mut RunningGameStats,
        game: &Game,
        new_feedback_events: FeedbackEvents,
        screen_resized: bool,
    ) -> io::Result<()>
    where
        T: Write;
}

pub fn tet_str_small(t: &Tetromino) -> &'static str {
    match t {
        Tetromino::O => "██",
        Tetromino::I => "▄▄▄▄",
        Tetromino::S => "▄█▀",
        Tetromino::Z => "▀█▄",
        Tetromino::T => "▄█▄",
        Tetromino::L => "▄▄█",
        Tetromino::J => "█▄▄",
    }
}

pub fn tet_str_minuscule(t: &Tetromino) -> &'static str {
    match t {
        Tetromino::O => "⠶", //"⠶",
        Tetromino::I => "⡇", //"⠤⠤",
        Tetromino::S => "⠳", //"⠴⠂",
        Tetromino::Z => "⠞", //"⠲⠄",
        Tetromino::T => "⠗", //"⠴⠄",
        Tetromino::L => "⠧", //"⠤⠆",
        Tetromino::J => "⠼", //"⠦⠄",
    }
}

pub fn tile_to_color(
    color_mode: GraphicsColor,
    custom_palette: &HashMap<u8, Color>,
) -> Box<dyn Fn(TileTypeID) -> Option<Color> + '_> {
    match color_mode {
        GraphicsColor::Monochrome => Box::new(|tile_type_id: TileTypeID| {
            HashMap::from(COLOR_PALETTE_MONOCHROME)
                .get(&tile_type_id.get())
                .copied()
        }),
        GraphicsColor::Color16 => Box::new(|tile_type_id: TileTypeID| {
            HashMap::from(COLOR_PALETTE_COLOR16)
                .get(&tile_type_id.get())
                .copied()
        }),
        GraphicsColor::Fullcolor => Box::new(|tile_type_id: TileTypeID| {
            HashMap::from(COLOR_PALETTE_DEFAULT)
                .get(&tile_type_id.get())
                .copied()
        }),
        GraphicsColor::Custom => {
            Box::new(|tile_type_id: TileTypeID| custom_palette.get(&tile_type_id.get()).copied())
        }
    }
}

pub const COLOR_PALETTE_MONOCHROME: [(u8, Color); 0] = [];

pub const COLOR_PALETTE_COLOR16: [(u8, Color); 7 + 3] = [
    (1, Color::Yellow),
    (2, Color::DarkCyan),
    (3, Color::Green),
    (4, Color::DarkRed),
    (5, Color::DarkMagenta),
    (6, Color::Red),
    (7, Color::Blue),
    (253, Color::Black),
    (254, Color::DarkGrey),
    (255, Color::White),
];

#[rustfmt::skip]
pub const COLOR_PALETTE_DEFAULT: [(u8, Color); 7+3] = [
    (  1, Color::Rgb{r:254,g:203,b:  1}),
    (  2, Color::Rgb{r:  0,g:159,b:219}),
    (  3, Color::Rgb{r:105,g:190,b: 41}),
    (  4, Color::Rgb{r:237,g: 41,b: 58}),
    (  5, Color::Rgb{r:149,g: 45,b:153}),
    (  6, Color::Rgb{r:255,g:121,b:  1}),
    (  7, Color::Rgb{r:  0,g:101,b:190}),
    (253, Color::Rgb{r:  0,g:  0,b:  1}),
    (254, Color::Rgb{r:127,g:127,b:127}),
    (255, Color::Rgb{r:255,g:255,b:255}),
];

#[rustfmt::skip]
pub const COLOR_PALETTE_EXPERIMENTAL: [(u8, Color); 7+3] = [
    (  1, Color::Rgb{r: 14,g:198,b:244}),
    (  2, Color::Rgb{r:242,g:192,b: 29}),
    (  3, Color::Rgb{r: 70,g:201,b: 50}),
    (  4, Color::Rgb{r:230,g: 53,b:197}),
    (  5, Color::Rgb{r:147,g: 41,b:229}),
    (  6, Color::Rgb{r: 36,g:118,b:242}),
    (  7, Color::Rgb{r:244,g: 50,b: 48}),
    (253, Color::Rgb{r:  0,g:  0,b:  0}),
    (254, Color::Rgb{r:127,g:127,b:127}),
    (255, Color::Rgb{r:255,g:255,b:255}),
];
