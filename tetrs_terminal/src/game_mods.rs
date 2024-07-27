use tetrs_engine::{
    Feedback, FeedbackEvents, GameConfig, GameMode, GameState, InternalEvent, Tetromino,
    TetrominoGenerator,
};

#[allow(dead_code)]
pub fn display_tetromino_likelihood_mod(
    config: &mut GameConfig,
    _mode: &mut GameMode,
    state: &mut GameState,
    feedback_events: &mut FeedbackEvents,
    event: Result<InternalEvent, InternalEvent>,
) {
    if event == Err(InternalEvent::Spawn) {
        let TetrominoGenerator::Recency { last_generated: lg } = config.tetromino_generator else {
            panic!()
        };
        let mut pieces_played_strs = [
            Tetromino::O,
            Tetromino::I,
            Tetromino::S,
            Tetromino::Z,
            Tetromino::T,
            Tetromino::L,
            Tetromino::J,
        ];
        pieces_played_strs.sort_by_key(|&t| lg[t]);
        feedback_events.push((
            state.time,
            Feedback::Message(format!(
                "{} {}",
                state.event_ticker,
                pieces_played_strs
                    .map(|t| format!(
                        "{t:?}{}{}{}",
                        lg[t],
                        // "█".repeat(lg[t] as usize),
                        "█".repeat((lg[t] * lg[t]) as usize / 8),
                        [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉"][(lg[t] * lg[t]) as usize % 8]
                    )
                    .to_ascii_lowercase())
                    .join("")
            )),
        ));
        // config.line_clear_delay = Duration::ZERO;
        // config.appearance_delay = Duration::ZERO;
        // state.board.remove(0);
        // state.board.push(Default::default());
        // state.board.remove(0);
        // state.board.push(Default::default());
    }
}
