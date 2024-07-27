use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::Debug,
    io::{self, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::KeyCode,
    style::{self, Color, Print, Stylize},
    terminal, QueueableCommand,
};
use tetrs_engine::{
    Button, Coord, Feedback, FeedbackEvents, Game, GameState, GameTime, Orientation, Tetromino,
    TileTypeID,
};

use crate::{
    game_renderers::GameScreenRenderer,
    terminal_tetrs::{format_duration, format_key, format_keybinds, App, RunningGameStats},
};

#[derive(Clone, Default, Debug)]
struct ScreenBuf {
    prev: Vec<Vec<(char, Option<Color>)>>,
    next: Vec<Vec<(char, Option<Color>)>>,
    x_draw: usize,
    y_draw: usize,
}

#[derive(Clone, Default, Debug)]
pub struct Renderer {
    screen: ScreenBuf,
    visual_events: Vec<(GameTime, Feedback, bool)>,
    messages: VecDeque<(GameTime, String)>,
    hard_drop_tiles: Vec<(GameTime, Coord, usize, TileTypeID, bool)>,
}

impl ScreenBuf {
    fn buf_from_strs(&mut self, base_screen: Vec<String>) {
        self.next = base_screen
            .iter()
            .map(|str| str.chars().zip(std::iter::repeat(None)).collect())
            .collect();
    }

    fn buf_str(&mut self, str: &str, fg_color: Option<Color>, (x, y): (usize, usize)) {
        for (x_c, c) in str.chars().enumerate() {
            // Lazy: just fill up until desired thing row exists.
            while y >= self.next.len() {
                self.next.push(Vec::new());
            }
            let row = &mut self.next[y];
            while x + x_c >= row.len() {
                row.push((' ', None));
            }
            row[x + x_c] = (c, fg_color);
        }
    }

    fn buf_reset(&mut self, (x, y): (usize, usize)) {
        self.prev.clear();
        (self.x_draw, self.y_draw) = (x, y);
    }

    fn flush(&mut self, term: &mut impl Write) -> io::Result<()> {
        // Begin frame update.
        term.queue(terminal::BeginSynchronizedUpdate)?;
        if self.prev.is_empty() {
            // Redraw entire screen.
            term.queue(terminal::Clear(terminal::ClearType::All))?;
            for (y, line) in self.next.iter().enumerate() {
                for (x, cell) in line.iter().enumerate() {
                    self.put(term, cell, x, y)?;
                }
            }
        } else {
            // Compare two frames and only write differences.
            for (y, (prev_line, next_line)) in self.prev.iter().zip(self.next.iter()).enumerate() {
                for (x, (prev_cell, next_cell)) in
                    prev_line.iter().zip(next_line.iter()).enumerate()
                {
                    if next_cell != prev_cell {
                        self.put(term, next_cell, x, y)?;
                    }
                }
                match prev_line.len().cmp(&next_line.len()) {
                    Ordering::Less => {
                        for (x, next_cell) in next_line.iter().enumerate().skip(prev_line.len()) {
                            self.put(term, next_cell, x, y)?;
                        }
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {
                        for x in next_line.len()..prev_line.len() {
                            self.put(term, &(' ', None), x, y)?;
                        }
                    }
                }
            }
            match self.prev.len().cmp(&self.next.len()) {
                Ordering::Less => {
                    for (y, next_line) in self.next.iter().enumerate().skip(self.prev.len()) {
                        for (x, next_cell) in next_line.iter().enumerate() {
                            self.put(term, next_cell, x, y)?;
                        }
                    }
                }
                Ordering::Equal => {}
                Ordering::Greater => {
                    for (y, prev_line) in self.prev.iter().enumerate().skip(self.next.len()) {
                        for (x, _) in prev_line.iter().enumerate() {
                            self.put(term, &(' ', None), x, y)?;
                        }
                    }
                }
            }
        }
        // End frame update and flush.
        term.queue(cursor::MoveTo(0, 0))?;
        term.queue(terminal::EndSynchronizedUpdate)?;
        term.flush()?;
        // Clear old.
        self.prev = Vec::new();
        // Swap buffers.
        std::mem::swap(&mut self.prev, &mut self.next);
        Ok(())
    }

    fn put(
        &self,
        term: &mut impl Write,
        (c, col): &(char, Option<Color>),
        x: usize,
        y: usize,
    ) -> io::Result<()> {
        term.queue(cursor::MoveTo(
            u16::try_from(self.x_draw + x).unwrap(),
            u16::try_from(self.y_draw + y).unwrap(),
        ))?;
        if let Some(color) = col {
            term.queue(style::PrintStyledContent(c.with(*color)))?;
        } else {
            term.queue(Print(c))?;
        }
        Ok(())
    }
}

impl GameScreenRenderer for Renderer {
    // NOTE self: what is the concept of having an ADT but some functions are only defined on some variants (that may contain record data)?
    fn render<T>(
        &mut self,
        app: &mut App<T>,
        game: &mut Game,
        action_stats: &mut RunningGameStats,
        new_feedback_events: FeedbackEvents,
        screen_resized: bool,
    ) -> io::Result<()>
    where
        T: Write,
    {
        if screen_resized {
            let (x_main, y_main) = App::<T>::fetch_main_xy();
            self.screen
                .buf_reset((usize::from(x_main), usize::from(y_main)));
        }
        let GameState {
            game_time,
            update_counter: _,
            end: _,
            events: _,
            buttons_pressed: _,
            board,
            active_piece_data,
            next_pieces,
            pieces_played,
            lines_cleared,
            level,
            score,
            consecutive_line_clears: _,
            back_to_back_special_clears: _,
        } = game.state();
        // Screen: some titles.
        let mode_name = game.mode().name.to_ascii_uppercase();
        let mode_name_space = mode_name.len().max(14);
        let (goal_name, goal_value) = [
            game.mode().limits.time.map(|(_, max_dur)| {
                (
                    "Time left:",
                    format_duration(max_dur.saturating_sub(*game_time)),
                )
            }),
            game.mode().limits.pieces.map(|(_, max_pcs)| {
                (
                    "Pieces remaining:",
                    max_pcs
                        .saturating_sub(pieces_played.iter().sum::<u32>())
                        .to_string(),
                )
            }),
            game.mode().limits.lines.map(|(_, max_lns)| {
                (
                    "Lines left to clear:",
                    max_lns.saturating_sub(*lines_cleared).to_string(),
                )
            }),
            game.mode().limits.level.map(|(_, max_lvl)| {
                (
                    "Levels remaining:",
                    max_lvl.get().saturating_sub(level.get()).to_string(),
                )
            }),
            game.mode().limits.score.map(|(_, max_pts)| {
                (
                    "Points to score:",
                    max_pts.saturating_sub(*score).to_string(),
                )
            }),
        ]
        .into_iter()
        .find_map(|limit_text| limit_text)
        .unwrap_or_default();
        let (focus_name, focus_value) = match game.mode().name.as_str() {
            "Marathon" => ("Score:", score.to_string()),
            "40-Lines" => ("Time taken:", format_duration(*game_time)),
            "Time Trial" => ("Lines cleared:", lines_cleared.to_string()),
            "Master" => ("Lines cleared:", lines_cleared.to_string()),
            "Puzzle" => ("", "".to_string()),
            _ => ("Lines cleared:", lines_cleared.to_string()),
        };
        let key_icon_pause = format_key(KeyCode::Esc);
        let key_icons_moveleft = format_keybinds(Button::MoveLeft, &app.settings().keybinds);
        let key_icons_moveright = format_keybinds(Button::MoveRight, &app.settings().keybinds);
        let mut key_icons_move = format!("{key_icons_moveleft} {key_icons_moveright}");
        let key_icons_rotateleft = format_keybinds(Button::RotateLeft, &app.settings().keybinds);
        let key_icons_rotateright = format_keybinds(Button::RotateRight, &app.settings().keybinds);
        let mut key_icons_rotate = format!("{key_icons_rotateleft} {key_icons_rotateright}");
        let key_icons_dropsoft = format_keybinds(Button::DropSoft, &app.settings().keybinds);
        let key_icons_drophard = format_keybinds(Button::DropHard, &app.settings().keybinds);
        let mut key_icons_drop = format!("{key_icons_dropsoft} {key_icons_drophard}");
        // JESUS Christ https://users.rust-lang.org/t/truncating-a-string/77903/9 :
        let eleven = key_icons_move
            .char_indices()
            .map(|(i, _)| i)
            .nth(11)
            .unwrap_or(key_icons_move.len());
        key_icons_move.truncate(eleven);
        let eleven = key_icons_rotate
            .char_indices()
            .map(|(i, _)| i)
            .nth(11)
            .unwrap_or(key_icons_rotate.len());
        key_icons_rotate.truncate(eleven);
        let eleven = key_icons_drop
            .char_indices()
            .map(|(i, _)| i)
            .nth(11)
            .unwrap_or(key_icons_drop.len());
        key_icons_drop.truncate(eleven);
        let piececnts_o = format!("{}o", pieces_played[Tetromino::O]);
        let piececnts_i_s_z = [
            format!("{}i", pieces_played[Tetromino::I]),
            format!("{}s", pieces_played[Tetromino::S]),
            format!("{}z", pieces_played[Tetromino::Z]),
        ]
        .join("  ");
        let piececnts_t_l_j = [
            format!("{}t", pieces_played[Tetromino::T]),
            format!("{}l", pieces_played[Tetromino::L]),
            format!("{}j", pieces_played[Tetromino::J]),
        ]
        .join("  ");
        // Screen: draw.
        #[allow(clippy::useless_format)]
        #[rustfmt::skip]
        let base_screen = if app.settings().ascii_graphics {
            vec![
                format!("                                                            ", ),
                format!("                       +- - - - - - - - - - +{:-^w$       }+", "mode", w=mode_name_space),
                format!("   ALL STATS           |                    |{: ^w$       }|", mode_name, w=mode_name_space),
                format!("   ----------          |                    +{:-^w$       }+", "", w=mode_name_space),
                format!("   Level: {:<13       }|                    |  {          }:", level, goal_name),
                format!("   Score: {:<13       }|                    |{:^15         }", score, goal_value),
                format!("   Lines: {:<13       }|                    |               ", lines_cleared),
                format!("                       |                    |  {          }:", focus_name),
                format!("   Time elapsed        |                    |{:^15         }", focus_value),
                format!("    {:<19             }|                    |               ", format_duration(*game_time)),
                format!("                       |                    |-----next-----+", ),
                format!("   PIECES              |                    |              |", ),
                format!("   -------             |                    |              |", ),
                format!("   {:<20              }|                    |--------------+", piececnts_o),
                format!("   {:<20              }|                    |               ", piececnts_i_s_z),
                format!("   {:<20              }|                    |               ", piececnts_t_l_j),
                format!("                       |                    |               ", ),
                format!("   CONTROLS            |                    |               ", ),
                format!("   ---------           |                    |               ", ),
                format!("   Move    {:<12      }|                    |               ", key_icons_move),
                format!("   Rotate  {:<12      }|                    |               ", key_icons_rotate),
                format!("   Drop    {:<12      }|                    |               ", key_icons_drop),
                format!("   Pause   {:<9    }  ~#====================#~              ", key_icon_pause),
                format!("                                                            ", ),
            ]
        } else {
            vec![
                format!("                                                            ", ),
                format!("                       ╓╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╥{:─^w$       }┐", "mode", w=mode_name_space),
                format!("   ALL STATS           ║                    ║{: ^w$       }│", mode_name, w=mode_name_space),
                format!("   ─────────╴          ║                    ╟{:─^w$       }┘", "", w=mode_name_space),
                format!("   Level: {:<13       }║                    ║  {           }", level, goal_name),
                format!("   Score: {:<13       }║                    ║{:^15         }", score, goal_value),
                format!("   Lines: {:<13       }║                    ║               ", lines_cleared),
                format!("                       ║                    ║  {           }", focus_name),
                format!("   Time elapsed        ║                    ║{:^15         }", focus_value),
                format!("    {:<19             }║                    ║               ", format_duration(*game_time)),
                format!("                       ║                    ║─────next─────┐", ),
                format!("   PIECES              ║                    ║              │", ),
                format!("   ──────╴             ║                    ║              │", ),
                format!("   {:<20              }║                    ║──────────────┘", piececnts_o),
                format!("   {:<20              }║                    ║               ", piececnts_i_s_z),
                format!("   {:<20              }║                    ║               ", piececnts_t_l_j),
                format!("                       ║                    ║               ", ),
                format!("   CONTROLS            ║                    ║               ", ),
                format!("   ────────╴           ║                    ║               ", ),
                format!("   Move    {:<12      }║                    ║               ", key_icons_move),
                format!("   Rotate  {:<12      }║                    ║               ", key_icons_rotate),
                format!("   Drop    {:<12      }║                    ║               ", key_icons_drop),
                format!("   Pause   {:<9    }░▒▓█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█▓▒░            ", key_icon_pause),
                format!("                                                            ", ),
            ]
        };
        self.screen.buf_from_strs(base_screen);
        let (x_board, y_board) = (24, 1);
        let (x_preview, y_preview) = (48, 12);
        let (x_messages, y_messages) = (47, 15);
        let pos_board = |(x, y)| (x_board + 2 * x, y_board + Game::SKYLINE - y);
        // Board: helpers.
        #[rustfmt::skip]
        let tile_color = if app.settings().ascii_graphics {
            |_tile: TileTypeID| None
        } else {
            |tile: TileTypeID| {
                Some(match tile.get() {
                      1 => Color::Rgb { r:254, g:203, b:  0 },
                      2 => Color::Rgb { r:  0, g:159, b:218 },
                      3 => Color::Rgb { r:105, g:190, b: 40 },
                      4 => Color::Rgb { r:237, g: 41, b: 57 },
                      5 => Color::Rgb { r:149, g: 45, b:152 },
                      6 => Color::Rgb { r:255, g:121, b:  0 },
                      7 => Color::Rgb { r:  0, g:101, b:189 },
                    255 => Color::Rgb { r:127, g:127, b:127 },
                    t => unimplemented!("formatting unknown tile id {t}"),
                })
            }
        };
        // Board: draw hard drop trail.
        for (event_time, pos, h, tile_type_id, relevant) in self.hard_drop_tiles.iter_mut() {
            let elapsed = game_time.saturating_sub(*event_time);
            let luminance_map = "@$#%*+~.".as_bytes();
            // let Some(&char) = [50, 60, 70, 80, 90, 110, 140, 180]
            let Some(&char) = [50, 70, 90, 110, 130, 150, 180, 240]
                .iter()
                .enumerate()
                .find_map(|(idx, ms)| (elapsed < Duration::from_millis(*ms)).then_some(idx))
                .and_then(|dt| luminance_map.get(*h / 2 + dt))
            else {
                *relevant = false;
                continue;
            };
            // SAFETY: Valid ASCII bytes.
            let tile = String::from_utf8(vec![char, char]).unwrap();
            self.screen
                .buf_str(&tile, tile_color(*tile_type_id), pos_board(*pos));
        }
        self.hard_drop_tiles.retain(|elt| elt.4);
        // Board: draw fixed tiles.
        let (tile_locked, tile_ghost, tile_active, tile_preview) = if app.settings().ascii_graphics
        {
            ("##", "::", "[]", "[]")
        } else {
            ("██", "░░", "▓▓", "▒▒")
        };
        for (y, line) in board.iter().enumerate().take(21).rev() {
            for (x, cell) in line.iter().enumerate() {
                if let Some(tile_type_id) = cell {
                    self.screen
                        .buf_str(tile_locked, tile_color(*tile_type_id), pos_board((x, y)));
                }
            }
        }
        // If a piece is in play.
        if let Some((active_piece, _)) = active_piece_data {
            // Draw ghost piece.
            for (tile_pos, tile_type_id) in active_piece.well_piece(board).tiles() {
                if tile_pos.1 <= Game::SKYLINE {
                    self.screen
                        .buf_str(tile_ghost, tile_color(tile_type_id), pos_board(tile_pos));
                }
            }
            // Draw active piece.
            for (tile_pos, tile_type_id) in active_piece.tiles() {
                if tile_pos.1 <= Game::SKYLINE {
                    self.screen
                        .buf_str(tile_active, tile_color(tile_type_id), pos_board(tile_pos));
                }
            }
        }
        // Draw preview.
        // TODO: Possibly implement more preview.
        if game.config().preview_count > 0 {
            // SAFETY: `preview_count > 0`.
            let next_piece = next_pieces.front().unwrap();
            let color = tile_color(next_piece.tiletypeid());
            for (x, y) in next_piece.minos(Orientation::N) {
                let pos = (x_preview + 2 * x, y_preview - y);
                self.screen.buf_str(tile_preview, color, pos);
            }
        }
        // Update stored events.
        self.visual_events.extend(
            new_feedback_events
                .into_iter()
                .map(|(time, event)| (time, event, true)),
        );
        // Draw events.
        for (event_time, event, relevant) in self.visual_events.iter_mut().rev() {
            let elapsed = game_time.saturating_sub(*event_time);
            match event {
                Feedback::PieceLocked(piece) => {
                    #[rustfmt::skip]
                    let lock_anim = if app.settings().ascii_graphics {
                        [
                            ( 50, "()"),
                            ( 75, "()"),
                            (100, "{}"),
                            (125, "{}"),
                            (150, "<>"),
                            (175, "<>"),
                        ]
                    } else {
                        [
                            ( 50, "██"),
                            ( 75, "▓▓"),
                            (100, "▒▒"),
                            (125, "░░"),
                            (150, "▒▒"),
                            (175, "▓▓"),
                        ]
                    };
                    let Some(tile) = lock_anim.iter().find_map(|(ms, tile)| {
                        (elapsed < Duration::from_millis(*ms)).then_some(tile)
                    }) else {
                        *relevant = false;
                        continue;
                    };
                    for (tile_pos, _tile_type_id) in piece.tiles() {
                        if tile_pos.1 <= Game::SKYLINE {
                            self.screen
                                .buf_str(tile, Some(Color::White), pos_board(tile_pos));
                        }
                    }
                }
                Feedback::LineClears(lines_cleared, line_clear_delay) => {
                    if line_clear_delay.is_zero() {
                        *relevant = false;
                        continue;
                    }
                    let line_clear_frames = if app.settings().ascii_graphics {
                        [
                            "$$$$$$$$$$$$$$$$$$$$",
                            "$$$$$$$$$$$$$$$$$$$$",
                            "                    ",
                            "                    ",
                            "$$$$$$$$$$$$$$$$$$$$",
                            "$$$$$$$$$$$$$$$$$$$$",
                            "                    ",
                            "                    ",
                            "$$$$$$$$$$$$$$$$$$$$",
                            "$$$$$$$$$$$$$$$$$$$$",
                        ]
                    } else {
                        [
                            "████████████████████",
                            " ██████████████████ ",
                            "  ████████████████  ",
                            "   ██████████████   ",
                            "    ████████████    ",
                            "     ██████████     ",
                            "      ████████      ",
                            "       ██████       ",
                            "        ████        ",
                            "         ██         ",
                        ]
                    };
                    let percent = elapsed.as_secs_f64() / line_clear_delay.as_secs_f64();
                    // SAFETY: `0.0 <= percent && percent <= 1.0`.
                    let idx = if percent < 1.0 {
                        unsafe { (10.0 * percent).to_int_unchecked::<usize>() }
                    } else {
                        *relevant = false;
                        continue;
                    };
                    for y_line in lines_cleared {
                        let pos = (x_board, y_board + Game::SKYLINE - *y_line);
                        self.screen
                            .buf_str(line_clear_frames[idx], Some(Color::White), pos);
                    }
                }
                Feedback::HardDrop(_top_piece, bottom_piece) => {
                    for ((x_tile, y_tile), tile_type_id) in bottom_piece.tiles() {
                        for y in y_tile..Game::SKYLINE {
                            self.hard_drop_tiles.push((
                                *event_time,
                                (x_tile, y),
                                y - y_tile,
                                tile_type_id,
                                true,
                            ));
                        }
                    }
                    *relevant = false;
                }
                Feedback::Accolade {
                    score_bonus,
                    shape,
                    spin,
                    lineclears,
                    perfect_clear,
                    combo,
                    back_to_back,
                } => {
                    action_stats.1.push(*score_bonus);
                    let mut strs = Vec::new();
                    strs.push(format!("+{score_bonus}"));
                    if *perfect_clear {
                        strs.push("PERFECT".to_string());
                    }
                    if *spin {
                        strs.push(format!("{shape:?}-Spin"));
                        action_stats.0[0] += 1;
                    }
                    let clear_action = match lineclears {
                        1 => "Single",
                        2 => "Double",
                        3 => "Triple",
                        4 => "Quadruple",
                        5 => "Quintuple",
                        6 => "Sextuple",
                        7 => "Septuple",
                        8 => "Octuple",
                        9 => "Nonuple",
                        10 => "Decuple",
                        11 => "Undecuple",
                        12 => "Duodecuple",
                        13 => "Tredecuple",
                        14 => "Quattuordecuple",
                        15 => "Quindecuple",
                        16 => "Sexdecuple",
                        17 => "Septendecuple",
                        18 => "Octodecuple",
                        19 => "Novemdecuple",
                        20 => "Vigintuple",
                        21 => "Kirbtris",
                        _ => "unreachable",
                    }
                    .to_string();
                    if *lineclears <= 4 {
                        action_stats.0[usize::try_from(*lineclears).unwrap()] += 1;
                    } else {
                        // TODO: Record higher lineclears, if even possible.
                    }
                    strs.push(clear_action);
                    if *combo > 1 {
                        strs.push(format!("({combo}.combo)"));
                    }
                    if *back_to_back > 1 {
                        strs.push(format!("({back_to_back}.B2B)"));
                    }
                    self.messages.push_front((*event_time, strs.join(" ")));
                    *relevant = false;
                }
                Feedback::Message(msg) => {
                    self.messages.push_front((*event_time, msg.clone()));
                    *relevant = false;
                }
            }
        }
        self.visual_events.retain(|elt| elt.2);
        // Draw messages.
        for (y, (_event_time, message)) in self.messages.iter().enumerate() {
            let pos = (x_messages, y_messages + y);
            self.screen.buf_str(message, None, pos);
        }
        self.messages.retain(|(timestamp, _message)| {
            game_time.saturating_sub(*timestamp) < Duration::from_millis(10000)
        });
        self.screen.flush(&mut app.term)
    }
}
