use std::{
    collections::VecDeque,
    io::{self, Write},
};

use crossterm::{
    cursor::{self, MoveToNextLine},
    style::{self, Print},
    terminal, QueueableCommand,
};
use tetrs_engine::{FeedbackEvent, Game, GameState, GameTime};

use crate::{
    game_renderers::GameScreenRenderer,
    terminal_tetrs::{App, GameRunningStats},
};

#[derive(Clone, Default, Debug)]
pub struct Renderer {
    feedback_event_buffer: VecDeque<(GameTime, FeedbackEvent)>,
}

impl GameScreenRenderer for Renderer {
    fn render<T>(
        &mut self,
        app: &mut App<T>,
        game: &mut Game,
        _action_stats: &mut GameRunningStats,
        new_feedback_events: Vec<(GameTime, FeedbackEvent)>,
        _screen_resized: bool,
    ) -> io::Result<()>
    where
        T: Write,
    {
        // Draw game stuf
        let GameState {
            game_time,
            board,
            active_piece_data,
            ..
        } = game.state();
        let mut temp_board = board.clone();
        if let Some((active_piece, _)) = active_piece_data {
            for ((x, y), tile_type_id) in active_piece.tiles() {
                temp_board[y][x] = Some(tile_type_id);
            }
        }
        app.term
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        app.term
            .queue(Print("   +--------------------+"))?
            .queue(MoveToNextLine(1))?;
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
            app.term.queue(Print(txt_line))?.queue(MoveToNextLine(1))?;
        }
        app.term
            .queue(Print("   +--------------------+"))?
            .queue(MoveToNextLine(1))?;
        app.term
            .queue(style::Print(format!("   {:?}", game_time)))?
            .queue(MoveToNextLine(1))?;
        // Draw feedback stuf
        for evt in new_feedback_events {
            self.feedback_event_buffer.push_front(evt);
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
                        strs.push(format!("{shape:?}-Spin"));
                    }
                    let clear_action = match lineclears {
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
                    strs.push(format!("{clear_action}{excl}"));
                    if *combo > 1 {
                        strs.push(format!("[{combo}.combo]"));
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
            app.term.queue(Print(str))?.queue(MoveToNextLine(1))?;
        }
        // Execute draw.
        app.term.flush()?;
        Ok(())
    }
}