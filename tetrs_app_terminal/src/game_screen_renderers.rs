use std::{
    collections::VecDeque, fmt::Debug, io::{self, Write}, time::{Duration, Instant}
};

use crossterm::{cursor, event::KeyCode, style::{self, Color, Stylize}, terminal, QueueableCommand};
use tetrs_lib::{Button, FeedbackEvent, Game, GameStateView, MeasureStat, Tetromino, TileTypeID};

use crate::terminal_tetrs::TerminalTetrs;

pub trait GameScreenRenderer {
    fn render(
        &mut self,
        ctx: &mut TerminalTetrs<impl Write>,
        game: &mut Game,
        new_feedback_events: Vec<(Instant, FeedbackEvent)>,
    ) -> io::Result<()>;
}

#[derive(Eq, PartialEq, Clone, Hash, Default, Debug)]
pub struct DebugRenderer {
    feedback_event_buffer: VecDeque<(Instant, FeedbackEvent)>,
}

#[derive(Eq, PartialEq, Clone, Hash, Default, Debug)]
pub struct UnicodeRenderer {
    event_buffer: VecDeque<(Instant, FeedbackEvent)>,
}

impl GameScreenRenderer for DebugRenderer {
    fn render(
        &mut self,
        ctx: &mut TerminalTetrs<impl Write>,
        game: &mut Game,
        new_feedback_events: Vec<(Instant, FeedbackEvent)>,
    ) -> io::Result<()> {
        // Draw game stuf
        let GameStateView {
            time_updated,
            board,
            active_piece,
            ..
        } = game.state();
        let mut temp_board = board.clone();
        if let Some(active_piece) = active_piece {
            for ((x, y), tile_type_id) in active_piece.tiles() {
                temp_board[y][x] = Some(tile_type_id);
            }
        }
        ctx.term
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        ctx.term
            .queue(style::Print("   +--------------------+"))?
            .queue(cursor::MoveToNextLine(1))?;
        for (idx, line) in temp_board.iter().take(20).enumerate().rev() {
            let txt_line = format!(
                "{idx:02} |{}|",
                line.iter()
                    .map(|cell| {
                        cell.map_or(" .", |tile| match tile.get() {
                            1 => "OO",
                            2 => "II",
                            3 => "SS",
                            4 => "ZZ",
                            5 => "TT",
                            6 => "LL",
                            7 => "JJ",
                            t => unimplemented!("formatting unknown tile id {t}"),
                        })
                    })
                    .collect::<Vec<_>>()
                    .join("")
            );
            ctx.term
                .queue(style::Print(txt_line))?
                .queue(cursor::MoveToNextLine(1))?;
        }
        ctx.term
            .queue(style::Print("   +--------------------+"))?
            .queue(cursor::MoveToNextLine(1))?;
        ctx.term
            .queue(style::Print(format!(
                "   {:?}",
                time_updated.saturating_duration_since(game.state().time_started)
            )))?
            .queue(cursor::MoveToNextLine(1))?;
        // Draw feedback stuf
        for event in new_feedback_events {
            self.feedback_event_buffer.push_front(event);
        }
        let mut feed_evt_msgs = Vec::new();
        for (_, feedback_event) in self.feedback_event_buffer.iter() {
            feed_evt_msgs.push(match feedback_event {
                FeedbackEvent::Accolade {
                    score_bonus,
                    shape,
                    spin,
                    lineclears,
                    perfect_clear,
                    combo,
                    opportunity,
                } => {
                    let mut strs = Vec::new();
                    if *spin {
                        strs.push(format!("{shape:?}-Spin"))
                    }
                    let accolade = match lineclears {
                        1 => "Single",
                        2 => "Double",
                        3 => "Triple",
                        4 => "Quadruple",
                        x => unreachable!("unexpected line clear count {x}"),
                    };
                    let excl = match opportunity {
                        1 => "'",
                        2 => "!",
                        3 => "!'",
                        4 => "!!",
                        x => unreachable!("unexpected opportunity count {x}"),
                    };
                    strs.push(format!("{accolade}{excl}"));
                    if *combo > 1 {
                        strs.push(format!("[{combo        }.combo]"));
                    }
                    if *perfect_clear {
                        strs.push("PERFECT!".to_string());
                    }
                    strs.push(format!("+{score_bonus}"));
                    strs.join(" ")
                }
                FeedbackEvent::PieceLocked(_) => continue,
                FeedbackEvent::LineClears(..) => continue,
                FeedbackEvent::HardDrop(_, _) => continue,
                FeedbackEvent::Debug(s) => s.clone(),
            });
        }
        for str in feed_evt_msgs.iter().take(16) {
            ctx.term
                .queue(style::Print(str))?
                .queue(cursor::MoveToNextLine(1))?;
        }
        // Execute draw.
        ctx.term.flush()?;
        Ok(())
    }
}

impl GameScreenRenderer for UnicodeRenderer {
    // NOTE: (note) what is the concept of having an ADT but some functions are only defined on some variants (that may contain record data)?
    fn render(
        &mut self,
        ctx: &mut TerminalTetrs<impl Write>,
        game: &mut Game,
        new_feedback_events: Vec<(Instant, FeedbackEvent)>,
    ) -> io::Result<()> {
        let (WIDTH, HEIGHT) = terminal::size()?;
        let GameStateView {
            lines_cleared,
            level,
            score,
            time_updated,
            board,
            active_piece,
            next_pieces,
            pieces_played,
            time_started,
            gamemode,
        } = game.state();
        // Clear screen.
        ctx.term
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        // Screen: some values.
        let lines = lines_cleared.len();
        let time_elapsed = time_updated.saturating_duration_since(time_started);
        // Screen: some helpers.
        let stat_name = |stat| match stat {
            MeasureStat::Lines(_) => "Lines",
            MeasureStat::Level(_) => "Levels",
            MeasureStat::Score(_) => "Score",
            MeasureStat::Pieces(_) => "Pieces",
            MeasureStat::Time(_) => "Time",
        };
        let fmt_time = |dur: Duration| format!("{}:{:02}.{:02}", dur.as_secs()/60, dur.as_secs()%60, dur.as_millis() % 1000 / 10);
        let fmt_key = |key: KeyCode| format!("[{}]", match key {
            KeyCode::Backspace => "BACK".to_string(),
            KeyCode::Enter => "ENTR".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Home => "HOME".to_string(),
            KeyCode::End => "END".to_string(),
            KeyCode::PageUp => "PgUp".to_string(),
            KeyCode::PageDown => "PgDn".to_string(),
            KeyCode::Tab => "TAB".to_string(),
            KeyCode::Delete => "DEL".to_string(),
            KeyCode::F(n) => format!("F{n}"),
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::Esc => "ESC".to_string(),
            _ => "??".to_string(),
        });
        // Screen: some titles.
        let opti_name = stat_name(gamemode.optimize);
        let opti_value = match gamemode.optimize {
            MeasureStat::Lines(_) => format!("{}", lines),
            MeasureStat::Level(_) => format!("{}", level),
            MeasureStat::Score(_) => format!("{}", score),
            MeasureStat::Pieces(_) => format!("{}", pieces_played.iter().sum::<u32>()),
            MeasureStat::Time(_) => fmt_time(time_elapsed),
        };
        let (goal_name, goal_value) = if let Some(stat) = gamemode.limit {
            (
                format!("{} left:", stat_name(stat)),
                match stat {
                    MeasureStat::Lines(lns) => format!("{}", lns - lines),
                    MeasureStat::Level(lvl) => format!("{}", lvl.get() - level.get()),
                    MeasureStat::Score(pts) => format!("{}", pts - score),
                    MeasureStat::Pieces(pcs) => format!("{}", pcs - pieces_played.iter().sum::<u32>()),
                    MeasureStat::Time(dur) => fmt_time(dur - time_elapsed),
                },
            )
        } else {
            ("".to_string(), "".to_string())
        };
        let key_icon_pause = fmt_key(KeyCode::Esc);
        let key_icons_moveleft = ctx.settings.keybinds.iter().filter_map(|(&k, &b)| (b==Button::MoveLeft).then_some(fmt_key(k))).collect::<Vec<String>>().join(" ");
        let key_icons_moveright = ctx.settings.keybinds.iter().filter_map(|(&k, &b)| (b==Button::MoveRight).then_some(fmt_key(k))).collect::<Vec<String>>().join(" ");
        let key_icons_move = format!("{key_icons_moveleft} {key_icons_moveright}");
        let key_icons_rotateleft = ctx.settings.keybinds.iter().filter_map(|(&k, &b)| (b==Button::RotateLeft).then_some(fmt_key(k))).collect::<Vec<String>>().join(" ");
        let key_icons_rotateright = ctx.settings.keybinds.iter().filter_map(|(&k, &b)| (b==Button::RotateRight).then_some(fmt_key(k))).collect::<Vec<String>>().join(" ");
        let key_icons_rotate = format!("{key_icons_rotateleft} {key_icons_rotateright}");
        let key_icons_dropsoft = ctx.settings.keybinds.iter().filter_map(|(&k, &b)| (b==Button::DropSoft).then_some(fmt_key(k))).collect::<Vec<String>>().join(" ");
        let key_icons_drophard = ctx.settings.keybinds.iter().filter_map(|(&k, &b)| (b==Button::DropHard).then_some(fmt_key(k))).collect::<Vec<String>>().join(" ");
        let key_icons_drop = format!("{key_icons_dropsoft} {key_icons_drophard}");
        let piececnts_o = format!("{}o", pieces_played[usize::from(Tetromino::O)]);
        let piececnts_i_s_z = vec![
            format!("{}i", pieces_played[usize::from(Tetromino::I)]),
            format!("{}s", pieces_played[usize::from(Tetromino::S)]),
            format!("{}z", pieces_played[usize::from(Tetromino::Z)]),
        ].join("  ");
        let piececnts_t_l_j = vec![
            format!("{}t", pieces_played[usize::from(Tetromino::T)]),
            format!("{}l", pieces_played[usize::from(Tetromino::L)]),
            format!("{}j", pieces_played[usize::from(Tetromino::J)]),
        ].join("  ");
        // Draw main screen assets (stats, frame, bg).
        let mut screen = Vec::new();
        screen.push(format!("                        ╓╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶╥─────mode─────┐", ));
        screen.push(format!("     ALL STATS          ║                    ║{:^14        }│", gamemode.name));
        screen.push(format!("     ─────────╴         ║                    ╟──────────────┘", ));
        screen.push(format!("     Level:{:>7  }      ║                    ║  {          }:", level, opti_name));
        screen.push(format!("     Score:{:>7  }      ║                    ║{:^15         }", score, opti_value));
        screen.push(format!("     Lines:{:>7  }      ║                    ║               ", lines));
        screen.push(format!("                        ║                    ║  {           }", goal_name));
        screen.push(format!("     Time elapsed       ║                    ║{:^15         }", goal_value));
        screen.push(format!("     {:>13       }      ║                    ║               ", fmt_time(time_elapsed)));
        screen.push(format!("                        ║                    ║─────next─────┐", ));
        screen.push(format!("     PIECES             ║                    ║              │", ));
        screen.push(format!("     ──────╴            ║                    ║              │", ));
        screen.push(format!("     {:<19             }║                    ║──────────────┘", piececnts_o));
        screen.push(format!("     {:<19             }║                    ║               ", piececnts_i_s_z));
        screen.push(format!("     {:<19             }║                    ║               ", piececnts_t_l_j));
        screen.push(format!("                        ║                    ║               ", ));
        screen.push(format!("     CONTROLS           ║                    ║               ", ));
        screen.push(format!("     ────────╴          ║                    ║               ", ));
        screen.push(format!("     Pause   {:<11     }║                    ║               ", key_icon_pause));
        screen.push(format!("     Move    {:<11     }║                    ║               ", key_icons_move));
        screen.push(format!("     Rotate  {:<11     }║                    ║               ", key_icons_rotate));
        screen.push(format!("     Drop    {:<11     }╚════════════════════╝               ", key_icons_drop));
        for str in screen {
            ctx.term
                .queue(style::Print(str))?
                .queue(cursor::MoveToNextLine(1))?;
        }
        let (DX, DY) = (25, 0);
        let (DXp, DYp) = (49, 11);
        // Render board tiles.
        let tile_color = |tile: TileTypeID| match tile.get() {
            1 => Color::Yellow,
            2 => Color::Cyan,
            3 => Color::Green,
            4 => Color::Red,
            5 => Color::DarkMagenta,
            6 => Color::DarkYellow,
            7 => Color::Blue,
            t => unimplemented!("formatting unknown tile id {t}"),
        };
        for (y, line) in board.iter().enumerate().take(21).rev() {
            for (x, cell) in line.iter().enumerate() {
                if let Some(tile_type_id) = cell {
                    let color = tile_color(*tile_type_id);
                    ctx.term
                    .queue(cursor::MoveTo(u16::try_from(DX + 2*x).unwrap(), u16::try_from(DY + (Game::SKYLINE - y)).unwrap()))?
                    .queue(style::PrintStyledContent("██".with(color)))?;
                }
            }
        }
        // If a piece is in play.
        if let Some(active_piece) = active_piece {
            // Draw ghost piece.
            for ((x, y), tile_type_id) in active_piece.well_piece(board).tiles() {
                if y > Game::SKYLINE {
                    continue;
                }
                let color = tile_color(tile_type_id);
                ctx.term
                .queue(cursor::MoveTo(u16::try_from(DX + 2*x).unwrap(), u16::try_from(DY + (Game::SKYLINE - y)).unwrap()))?
                .queue(style::PrintStyledContent("░░".with(color)))?;
            }
            // Draw active piece.
            for ((x, y), tile_type_id) in active_piece.tiles() {
                if y > Game::SKYLINE {
                    continue;
                }
                let color = tile_color(tile_type_id);
                ctx.term
                .queue(cursor::MoveTo(u16::try_from(DX + 2*x).unwrap(), u16::try_from(DY + (Game::SKYLINE - y)).unwrap()))?
                .queue(style::PrintStyledContent("▓▓".with(color)))?;
            }
        }
        // Draw preview.
        // TODO: SAFETY.
        let next_piece = next_pieces.front().unwrap();
        let color = tile_color(next_piece.tiletypeid());
        for (x, y) in next_piece.minos(tetrs_lib::Orientation::N) {
            ctx.term
            .queue(cursor::MoveTo(u16::try_from(DXp + 2*x).unwrap(), u16::try_from(DYp - y).unwrap()))?
            .queue(style::PrintStyledContent("▒▒".with(color)))?;
        }
        // Execute draw.
        ctx.term.flush()?;
        Ok(())
    }
}
