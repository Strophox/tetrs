mod rotation_systems;
mod tetromino_generators;

use std::{
    collections::{HashMap, VecDeque},
    fmt,
    num::NonZeroU32,
    ops,
    time::Duration,
};

use rand::rngs::ThreadRng;
pub use rotation_systems::RotationSystem;
pub use tetromino_generators::TetrominoGenerator;

pub type ButtonsPressed = [bool; 7];
// NOTE: Would've liked to use `impl Game { type Board = ...` (https://github.com/rust-lang/rust/issues/8995)
pub type TileTypeID = NonZeroU32;
pub type Line = [Option<TileTypeID>; Game::WIDTH];
pub type Board = Vec<Line>;
pub type Coord = (usize, usize);
pub type Offset = (isize, isize);
pub type GameTime = Duration;
pub type FeedbackEvents = Vec<(GameTime, Feedback)>;
pub type FnGameMod = Box<dyn FnMut(&mut GameConfig, &mut GameMode, &mut GameState, &mut FeedbackEvents, Option<InternalEvent>)>;
type EventMap = HashMap<InternalEvent, GameTime>;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Button {
    MoveLeft,
    MoveRight,
    RotateLeft,
    RotateRight,
    RotateAround,
    DropSoft,
    DropHard,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Orientation {
    N,
    E,
    S,
    W,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tetromino {
    O,
    I,
    S,
    Z,
    T,
    L,
    J,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivePiece {
    pub shape: Tetromino,
    pub orientation: Orientation,
    pub pos: Coord,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LockingData {
    touches_ground: bool,
    last_touchdown: Option<GameTime>,
    last_liftoff: Option<GameTime>,
    ground_time_left: Duration,
    lowest_y: usize,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Limits {
    pub time: Option<(bool, Duration)>,
    pub pieces: Option<(bool, u32)>,
    pub lines: Option<(bool, usize)>,
    pub level: Option<(bool, NonZeroU32)>,
    pub score: Option<(bool, u32)>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameMode {
    pub name: String,
    pub start_level: NonZeroU32,
    pub increment_level: bool,
    pub limits: Limits,
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameConfig {
    pub rotation_system: RotationSystem,
    pub tetromino_generator: TetrominoGenerator,
    pub preview_count: usize,
    pub delayed_auto_shift: Duration,
    pub auto_repeat_rate: Duration,
    pub soft_drop_factor: f64,
    pub hard_drop_delay: Duration,
    pub ground_time_max: Duration,
    pub line_clear_delay: Duration,
    pub appearance_delay: Duration,
    pub no_soft_drop_lock: bool,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InternalEvent {
    LineClear,
    Spawn,
    Lock,
    LockTimer,
    HardDrop,
    SoftDrop,
    MoveSlow,
    MoveFast,
    Rotate,
    Fall,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameOver {
    LockOut,
    BlockOut,
    ModeLimit,
    Forfeit,
}

#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameState {
    pub game_time: GameTime,
    pub update_counter: u64,
    pub end: Option<Result<(), GameOver>>,
    /// Invariants:
    /// * Until the game has finished there will always be more events: `finished.is_some() || !next_events.is_empty()`.
    /// * Unhandled events lie in the future: `for (event,event_time) in self.events { assert(self.time_updated < event_time); }`.
    pub events: EventMap,
    pub buttons_pressed: ButtonsPressed,
    pub board: Board,
    pub active_piece_data: Option<(ActivePiece, LockingData)>,
    pub next_pieces: VecDeque<Tetromino>,
    pub pieces_played: [u32; 7],
    pub lines_cleared: usize,
    pub level: NonZeroU32,
    pub score: u32,
    pub consecutive_line_clears: u32,
    pub back_to_back_special_clears: u32,
}

pub struct Game {
    config: GameConfig,
    mode: GameMode,
    state: GameState,
    rng: ThreadRng,
    modifiers: Vec<FnGameMod>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Feedback {
    PieceLocked(ActivePiece),
    LineClears(Vec<usize>, Duration),
    HardDrop(ActivePiece, ActivePiece),
    Accolade {
        score_bonus: u32,
        shape: Tetromino,
        spin: bool,
        lineclears: u32,
        perfect_clear: bool,
        combo: u32,
        back_to_back: u32,
    },
    Message(String),
}

impl Orientation {
    pub fn rotate_r(&self, right_turns: i32) -> Self {
        use Orientation::*;
        let base = match self {
            N => 0,
            E => 1,
            S => 2,
            W => 3,
        };
        match (base + right_turns).rem_euclid(4) {
            0 => N,
            1 => E,
            2 => S,
            3 => W,
            _ => unreachable!(),
        }
    }
}

impl Tetromino {
    pub fn minos(&self, oriented: Orientation) -> [Coord; 4] {
        use Orientation::*;
        match self {
            Tetromino::O => [(0, 0), (1, 0), (0, 1), (1, 1)], // ⠶
            Tetromino::I => match oriented {
                N | S => [(0, 0), (1, 0), (2, 0), (3, 0)], // ⠤⠤
                E | W => [(0, 0), (0, 1), (0, 2), (0, 3)], // ⡇
            },
            Tetromino::S => match oriented {
                N | S => [(0, 0), (1, 0), (1, 1), (2, 1)], // ⠴⠂
                E | W => [(1, 0), (0, 1), (1, 1), (0, 2)], // ⠳
            },
            Tetromino::Z => match oriented {
                N | S => [(1, 0), (2, 0), (0, 1), (1, 1)], // ⠲⠄
                E | W => [(0, 0), (0, 1), (1, 1), (1, 2)], // ⠞
            },
            Tetromino::T => match oriented {
                N => [(0, 0), (1, 0), (2, 0), (1, 1)], // ⠴⠄
                E => [(0, 0), (0, 1), (1, 1), (0, 2)], // ⠗
                S => [(1, 0), (0, 1), (1, 1), (2, 1)], // ⠲⠂
                W => [(1, 0), (0, 1), (1, 1), (1, 2)], // ⠺
            },
            Tetromino::L => match oriented {
                N => [(0, 0), (1, 0), (2, 0), (2, 1)], // ⠤⠆
                E => [(0, 0), (1, 0), (0, 1), (0, 2)], // ⠧
                S => [(0, 0), (0, 1), (1, 1), (2, 1)], // ⠖⠂
                W => [(1, 0), (1, 1), (0, 2), (1, 2)], // ⠹
            },
            Tetromino::J => match oriented {
                N => [(0, 0), (1, 0), (2, 0), (0, 1)], // ⠦⠄
                E => [(0, 0), (0, 1), (0, 2), (1, 2)], // ⠏
                S => [(2, 0), (0, 1), (1, 1), (2, 1)], // ⠒⠆
                W => [(0, 0), (1, 0), (1, 1), (1, 2)], // ⠼
            },
        }
    }

    pub const fn tiletypeid(&self) -> TileTypeID {
        use Tetromino::*;
        let u8 = match self {
            O => 1,
            I => 2,
            S => 3,
            Z => 4,
            T => 5,
            L => 6,
            J => 7,
        };
        // SAFETY: Ye, `u8 > 0`;
        unsafe { NonZeroU32::new_unchecked(u8) }
    }
}

impl TryFrom<usize> for Tetromino {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use Tetromino::*;
        Ok(match value {
            0 => O,
            1 => I,
            2 => S,
            3 => Z,
            4 => T,
            5 => L,
            6 => J,
            _ => Err(())?,
        })
    }
}

impl<T> ops::Index<Tetromino> for [T; 7] {
    type Output = T;

    fn index(&self, idx: Tetromino) -> &Self::Output {
        match idx {
            Tetromino::O => &self[0],
            Tetromino::I => &self[1],
            Tetromino::S => &self[2],
            Tetromino::Z => &self[3],
            Tetromino::T => &self[4],
            Tetromino::L => &self[5],
            Tetromino::J => &self[6],
        }
    }
}

impl<T> ops::IndexMut<Tetromino> for [T; 7] {
    fn index_mut(&mut self, idx: Tetromino) -> &mut Self::Output {
        match idx {
            Tetromino::O => &mut self[0],
            Tetromino::I => &mut self[1],
            Tetromino::S => &mut self[2],
            Tetromino::Z => &mut self[3],
            Tetromino::T => &mut self[4],
            Tetromino::L => &mut self[5],
            Tetromino::J => &mut self[6],
        }
    }
}

impl ActivePiece {
    pub fn tiles(&self) -> [(Coord, TileTypeID); 4] {
        let Self {
            shape,
            orientation,
            pos: (x, y),
        } = self;
        let tile_type_id = shape.tiletypeid();
        shape
            .minos(*orientation)
            .map(|(dx, dy)| ((x + dx, y + dy), tile_type_id))
    }

    pub fn fits(&self, board: &Board) -> bool {
        self.tiles()
            .iter()
            .all(|&((x, y), _)| x < Game::WIDTH && y < Game::HEIGHT && board[y][x].is_none())
    }

    pub fn fits_at(&self, board: &Board, offset: Offset) -> Option<ActivePiece> {
        let mut new_piece = *self;
        new_piece.pos = add(self.pos, offset)?;
        new_piece.fits(board).then_some(new_piece)
    }

    pub fn fits_at_rotated(
        &self,
        board: &Board,
        offset: Offset,
        right_turns: i32,
    ) -> Option<ActivePiece> {
        let mut new_piece = *self;
        new_piece.orientation = new_piece.orientation.rotate_r(right_turns);
        new_piece.pos = add(self.pos, offset)?;
        new_piece.fits(board).then_some(new_piece)
    }

    pub fn first_fit(
        &self,
        board: &Board,
        offsets: impl IntoIterator<Item = Offset>,
        right_turns: i32,
    ) -> Option<ActivePiece> {
        let mut new_piece = *self;
        new_piece.orientation = new_piece.orientation.rotate_r(right_turns);
        let old_pos = self.pos;
        offsets.into_iter().find_map(|offset| {
            new_piece.pos = add(old_pos, offset)?;
            new_piece.fits(board).then_some(new_piece)
        })
    }

    pub fn well_piece(&self, board: &Board) -> ActivePiece {
        let mut well_piece = *self;
        // Move piece all the way down.
        while let Some(piece_below) = well_piece.fits_at(board, (0, -1)) {
            well_piece = piece_below;
        }
        well_piece
    }
}

impl GameMode {
    #[allow(dead_code)]
    pub fn marathon() -> Self {
        Self {
            name: String::from("Marathon"),
            start_level: NonZeroU32::MIN,
            increment_level: true,
            limits: Limits { level: Some((true, Game::LEVEL_20G.saturating_add(1))), ..Default::default() },
        }
    }

    #[allow(dead_code)]
    pub fn sprint(start_level: NonZeroU32) -> Self {
        Self {
            name: String::from("40-Lines"),
            start_level,
            increment_level: false,
            limits: Limits { lines: Some((true, 40)), ..Default::default() },
        }
    }

    #[allow(dead_code)]
    pub fn ultra(start_level: NonZeroU32) -> Self {
        Self {
            name: String::from("Time Trial"),
            start_level,
            increment_level: false,
            limits: Limits { time: Some((true, Duration::from_secs(3 * 60))), ..Default::default() },
        }
    }

    #[allow(dead_code)]
    pub fn master() -> Self {
        Self {
            name: String::from("Master"),
            start_level: Game::LEVEL_20G,
            increment_level: true,
            limits: Limits { lines: Some((true, 300)), ..Default::default() },
        }
    }

    #[allow(dead_code)]
    pub fn endless() -> Self {
        Self {
            name: String::from("Zen"),
            start_level: NonZeroU32::MIN,
            increment_level: true,
            limits: Default::default(),
        }
    }
}

impl<T> ops::Index<Button> for [T; 7] {
    type Output = T;

    fn index(&self, idx: Button) -> &Self::Output {
        match idx {
            Button::MoveLeft => &self[0],
            Button::MoveRight => &self[1],
            Button::RotateLeft => &self[2],
            Button::RotateRight => &self[3],
            Button::RotateAround => &self[4],
            Button::DropSoft => &self[5],
            Button::DropHard => &self[6],
        }
    }
}

impl<T> ops::IndexMut<Button> for [T; 7] {
    fn index_mut(&mut self, idx: Button) -> &mut Self::Output {
        match idx {
            Button::MoveLeft => &mut self[0],
            Button::MoveRight => &mut self[1],
            Button::RotateLeft => &mut self[2],
            Button::RotateRight => &mut self[3],
            Button::RotateAround => &mut self[4],
            Button::DropSoft => &mut self[5],
            Button::DropHard => &mut self[6],
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            rotation_system: RotationSystem::Ocular,
            tetromino_generator: TetrominoGenerator::recency(),
            preview_count: 1,
            delayed_auto_shift: Duration::from_millis(200),
            auto_repeat_rate: Duration::from_millis(50),
            soft_drop_factor: 15.0,
            hard_drop_delay: Duration::from_micros(100),
            ground_time_max: Duration::from_millis(2250),
            line_clear_delay: Duration::from_millis(200),
            appearance_delay: Duration::from_millis(100),
            no_soft_drop_lock: false,
        }
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Game")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("rng", &std::any::type_name_of_val(&self.rng))
            .field("modifiers", &std::any::type_name_of_val(&self.modifiers))
            .finish()
    }
}

impl Game {
    pub const HEIGHT: usize = Self::SKYLINE + 7; // Max height *any* mino can reach before Lock out occurs.
    pub const WIDTH: usize = 10;
    pub const SKYLINE: usize = 20; // Typical maximal height of relevant (visible) playing grid.
                                   // SAFETY: 19 > 0, and this is the level at which blocks start falling with 20G.
    const LEVEL_20G: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(19) };

    pub fn new(gamemode: GameMode) -> Self {
        Self::with_config(gamemode, GameConfig::default())
    }

    pub fn with_config(gamemode: GameMode, mut config: GameConfig) -> Self {
        let mut rng = rand::thread_rng();
        let state = GameState {
            game_time: Duration::ZERO,
            update_counter: 0,
            end: None,
            events: HashMap::from([(InternalEvent::Spawn, Duration::ZERO)]),
            buttons_pressed: Default::default(),
            board: std::iter::repeat(Line::default())
                .take(Self::HEIGHT)
                .collect(),
            active_piece_data: None,
            next_pieces: config
                .tetromino_generator
                .with_rng(&mut rng)
                .take(config.preview_count)
                .collect(),
            pieces_played: [0; 7],
            lines_cleared: 0,
            level: gamemode.start_level,
            score: 0,
            consecutive_line_clears: 0,
            back_to_back_special_clears: 0,
        };
        Game {
            config,
            mode: gamemode,
            state,
            rng,
            modifiers: Vec::new(),
        }
    }

    pub fn forfeit(&mut self) {
        self.state.end = Some(Err(GameOver::Forfeit))
    }

    pub fn ended(&self) -> bool {
        self.state.end.is_some()
    }

    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut GameConfig {
        &mut self.config
    }

    pub fn mode(&self) -> &GameMode {
        &self.mode
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub unsafe fn add_modifier(&mut self, game_mod: FnGameMod) {
        self.modifiers.push(game_mod)
    }

    fn update_game_end(&mut self) {
        self.state.end = self.state.end.or_else(||
            [
                self.mode.limits.time.and_then(|(win, dur)| (dur <= self.state.game_time).then_some(win)),
                self.mode.limits.pieces.and_then(|(win, pcs)| (pcs <= self.state.pieces_played.iter().sum()).then_some(win)),
                self.mode.limits.lines.and_then(|(win, lns)| (lns <= self.state.lines_cleared).then_some(win)),
                self.mode.limits.level.and_then(|(win, lvl)| (lvl <= self.state.level).then_some(win)),
                self.mode.limits.score.and_then(|(win, pts)| (pts <= self.state.score).then_some(win)),
            ].into_iter().find_map(|limit_reached|
                limit_reached.map(|win|
                    if win {
                        Ok(())
                    } else {
                        Err(GameOver::ModeLimit)
                    }
                )
            )
        );
    }

    fn apply_modifiers(&mut self, feedback_events: &mut Vec<(GameTime, Feedback)>, before_event: Option<InternalEvent>) {
        for modify in &mut self.modifiers {
            modify(&mut self.config, &mut self.mode, &mut self.state, feedback_events, before_event);
        }
    }

    pub fn update(
        &mut self,
        mut new_button_state: Option<ButtonsPressed>,
        update_time: GameTime,
    ) -> Result<FeedbackEvents, ()> {
        /*
        Order:
        - if game already ended, return immediately
        * find next event
        - event less-or-equal update point:
            - allow modifiers
            - handle event
            - allow modifiers
            - update game end state, possibly return immediately
            - goto *
        - update point reached:
            - try adding input events, goto *
            - else return immediately
         */
        // Invalid call: return immediately.
        if update_time < self.state.game_time {
            return Err(());
        }
        // NOTE: Returning an empty Vec is efficient because it won't even allocate (as by Rust API).
        let mut feedback_events = Vec::new();
        if self.ended() {
            return Ok(feedback_events);
        };
        // We linearly process all events until we reach the update time.
        'event_simulation: loop {
            self.state.update_counter += 1;
            // Peek the next closest event.
            // SAFETY: `Game` invariants guarantee there's some event.
            let (&event, &event_time) = self
                .state
                .events
                .iter()
                .min_by_key(|(&event, &event_time)| (event_time, event))
                .unwrap();
            // Next event within requested update time, handle event first.
            if event_time <= update_time {
                self.apply_modifiers(&mut feedback_events, Some(event));
                // Remove next event and handle it.
                self.state.events.remove_entry(&event);
                let new_feedback_events = self.handle_event(event, event_time);
                self.state.game_time = event_time;
                feedback_events.extend(new_feedback_events);
                self.apply_modifiers(&mut feedback_events, None);
                // Stop simulation early if event or modifier ended game.
                self.update_game_end();
                if self.ended() {
                    break 'event_simulation
                }
            // Possibly process user input events now or break out.
            } else {
                // NOTE: We should be able to update the time here because `self.process_input(...)` does not access it.
                self.state.game_time = update_time;
                // Update button inputs.
                if let Some(buttons_pressed) = new_button_state.take() {
                    if self.state.active_piece_data.is_some() {
                        Self::add_input_events(
                            &mut self.state.events,
                            self.state.buttons_pressed,
                            buttons_pressed,
                            update_time,
                        );
                    }
                    self.state.buttons_pressed = buttons_pressed;
                } else {
                    self.update_game_end();
                    break 'event_simulation
                }
            }
        }
        Ok(feedback_events)
    }

    fn add_input_events(
        events: &mut EventMap,
        prev_buttons_pressed: ButtonsPressed,
        next_buttons_pressed: ButtonsPressed,
        update_time: GameTime,
    ) {
        #[allow(non_snake_case)]
        let [mL0, mR0, rL0, rR0, rA0, dS0, dH0] = prev_buttons_pressed;
        #[allow(non_snake_case)]
        let [mL1, mR1, rL1, rR1, rA1, dS1, dH1] = next_buttons_pressed;
        /*
        Table:                                 Karnaugh map:
        | mL0 mR0 mL1 mR1                      |           !mL1 !mL1  mL1  mL1
        |  0   0   0   0  :  -                 |           !mR1  mR1  mR1 !mR1
        |  0   0   0   1  :  move, move (DAS)  | !mL0 !mR0   -   DAS   -   DAS
        |  0   0   1   0  :  move, move (DAS)  | !mL0  mR0  rem   -   rem  ARR
        |  0   0   1   1  :  -                 |  mL0  mR0   -   ARR   -   ARR
        |  0   1   0   0  :  remove            |  mL0 !mR0  rem  ARR  rem   -
        |  0   1   0   1  :  -
        |  0   1   1   0  :  move, move (ARR)
        |  0   1   1   1  :  remove
        |  1   0   0   0  :  remove
        |  1   0   0   1  :  move, move (ARR)
        |  1   0   1   0  :  -
        |  1   0   1   1  :  remove
        |  1   1   0   0  :  -
        |  1   1   0   1  :  move, move (ARR)
        |  1   1   1   0  :  move, move (ARR)
        |  1   1   1   1  :  -
        */
        // No buttons pressed -> one button pressed, add initial move.
        if (!mL0 && !mR0) && (mL1 != mR1) {
            events.insert(InternalEvent::MoveSlow, update_time);
        // One/Two buttons pressed -> different/one button pressed, (re-)add fast repeat move.
        } else if (mL0 && (!mL1 && mR1)) || (mR0 && (mL1 && !mR1)) {
            events.remove(&InternalEvent::MoveFast);
            events.insert(InternalEvent::MoveFast, update_time);
        // Single button pressed -> both (un)pressed, remove future moves.
        } else if (mL0 != mR0) && (mL1 == mR1) {
            events.remove(&InternalEvent::MoveFast);
        }
        /*
        Table:                       Karnaugh map:
        | rL0 rR0 rL1 rR1            |           !rR1  rR1  rR1 !rR1
        |  0   0   0   0  :  -       |           !rL1 !rL1  rL1  rL1
        |  0   0   0   1  :  rotate  | !rL0 !rR0   -   rot   -   rot
        |  0   0   1   0  :  rotate  | !rL0  rR0   -    -   rot  rot
        |  0   0   1   1  :  -       |  rL0  rR0   -    -    -    -
        |  0   1   0   0  :  -       |  rL0 !rR0   -   rot  rot   -
        |  0   1   0   1  :  -
        |  0   1   1   0  :  rotate
        |  0   1   1   1  :  rotate
        |  1   0   0   0  :  -
        |  1   0   0   1  :  rotate
        |  1   0   1   0  :  -
        |  1   0   1   1  :  rotate
        |  1   1   0   0  :  -
        |  1   1   0   1  :  -
        |  1   1   1   0  :  -
        |  1   1   1   1  :  -
        We rotate around (rA) if (!rA0 && rA1).
        This always causes a rotation event (with no cancellation possible with rL,rR).
        */
        // Either a 180 rotation, or a single L/R rotation button was pressed.
        if (!rA0 && rA1) || (((!rR0 && rR1) || (!rL0 && rL1)) && (rL0 || rR0 || !rR1 || !rL1)) {
            events.insert(InternalEvent::Rotate, update_time);
        }
        // Soft drop button pressed.
        if !dS0 && dS1 {
            events.insert(InternalEvent::SoftDrop, update_time);
        }
        // Hard drop button pressed.
        if !dH0 && dH1 {
            events.insert(InternalEvent::HardDrop, update_time);
        }
    }

    fn handle_event(
        &mut self,
        event: InternalEvent,
        event_time: GameTime,
    ) -> FeedbackEvents {
        // Active piece touches the ground before update (or doesn't exist, counts as not touching).
        let mut feedback_events = Vec::new();
        let prev_piece_data = self.state.active_piece_data;
        let prev_piece = prev_piece_data.unzip().0;
        let next_piece = match event {
            // We generate a new piece above the skyline, and immediately queue a fall event for it.
            InternalEvent::Spawn => {
                debug_assert!(
                    prev_piece.is_none(),
                    "spawning new piece while an active piece is still in play"
                );
                let n_required_pieces =
                    (1 + self.config.preview_count).saturating_sub(self.state.next_pieces.len());
                self.state.next_pieces.extend(
                    self.config
                        .tetromino_generator
                        .with_rng(&mut self.rng)
                        .take(n_required_pieces),
                );
                let tetromino = self
                    .state
                    .next_pieces
                    .pop_front()
                    .expect("piece generator ran out before game finished");
                let next_piece = self.config.rotation_system.place_initial(tetromino);
                // Newly spawned piece conflicts with board - Game over.
                if !next_piece.fits(&self.state.board) {
                    self.state.end = Some(Err(GameOver::BlockOut));
                    return feedback_events;
                }
                self.state.events.insert(InternalEvent::Fall, event_time);
                if self.state.buttons_pressed[Button::MoveLeft]
                    || self.state.buttons_pressed[Button::MoveRight]
                {
                    self.state
                        .events
                        .insert(InternalEvent::MoveFast, event_time);
                }
                Some(next_piece)
            }
            InternalEvent::Rotate => {
                let prev_piece = prev_piece.expect("rotating none active piece");
                // Special 20G fall immediately after.
                if self.state.level >= Self::LEVEL_20G {
                    self.state.events.insert(InternalEvent::Fall, event_time);
                }
                let mut rotation = 0;
                if self.state.buttons_pressed[Button::RotateLeft] {
                    rotation -= 1;
                }
                if self.state.buttons_pressed[Button::RotateRight] {
                    rotation += 1;
                }
                if self.state.buttons_pressed[Button::RotateAround] {
                    rotation += 2;
                }
                self.config
                    .rotation_system
                    .rotate(&prev_piece, &self.state.board, rotation)
                    .or(Some(prev_piece))
            }
            InternalEvent::MoveSlow | InternalEvent::MoveFast => {
                // Handle move attempt and auto repeat move.
                let prev_piece = prev_piece.expect("moving none active piece");
                // Special 20G fall immediately after.
                if self.state.level >= Self::LEVEL_20G {
                    self.state.events.insert(InternalEvent::Fall, event_time);
                }
                let move_delay = if event == InternalEvent::MoveSlow {
                    self.config.delayed_auto_shift
                } else {
                    self.config.auto_repeat_rate
                };
                self.state
                    .events
                    .insert(InternalEvent::MoveFast, event_time + move_delay);
                #[rustfmt::skip]
                let dx = if self.state.buttons_pressed[Button::MoveLeft] { -1 } else { 1 };
                prev_piece
                    .fits_at(&self.state.board, (dx, 0))
                    .or(Some(prev_piece))
            }
            InternalEvent::Fall | InternalEvent::SoftDrop => {
                let prev_piece = prev_piece.expect("falling/softdropping none active piece");
                if self.state.level >= Self::LEVEL_20G {
                    Some(prev_piece.well_piece(&self.state.board))
                } else {
                    let drop_delay = if self.state.buttons_pressed[Button::DropSoft] {
                        Duration::from_secs_f64(
                            self.drop_delay().as_secs_f64() / self.config.soft_drop_factor,
                        )
                    } else {
                        self.drop_delay()
                    };
                    // Try to move active piece down.
                    if let Some(dropped_piece) = prev_piece.fits_at(&self.state.board, (0, -1)) {
                        self.state
                            .events
                            .insert(InternalEvent::Fall, event_time + drop_delay);
                        Some(dropped_piece)
                    // Piece hit ground but SoftDrop was pressed.
                    } else if event == InternalEvent::SoftDrop && !self.config.no_soft_drop_lock {
                        self.state.events.insert(InternalEvent::Lock, event_time);
                        Some(prev_piece)
                    // Piece hit ground and tried to drop naturally: don't do anything but try falling again later.
                    } else {
                        // NOTE: This could be changed if a reason for it appears.
                        self.state
                            .events
                            .insert(InternalEvent::Fall, event_time + drop_delay);
                        Some(prev_piece)
                    }
                }
            }
            InternalEvent::HardDrop => {
                let prev_piece = prev_piece.expect("harddropping none active piece");
                // Move piece all the way down.
                let dropped_piece = prev_piece.well_piece(&self.state.board);
                feedback_events.push((event_time, Feedback::HardDrop(prev_piece, dropped_piece)));
                self.state.events.insert(
                    InternalEvent::LockTimer,
                    event_time + self.config.hard_drop_delay,
                );
                Some(dropped_piece)
            }
            InternalEvent::LockTimer => {
                self.state.events.insert(InternalEvent::Lock, event_time);
                prev_piece
            }
            InternalEvent::Lock => {
                let prev_piece = prev_piece.expect("locking none active piece");
                // Attempt to lock active piece fully above skyline - Game over.
                if prev_piece
                    .tiles()
                    .iter()
                    .all(|((_, y), _)| *y >= Self::SKYLINE)
                {
                    self.state.end = Some(Err(GameOver::LockOut));
                    return feedback_events;
                }
                self.state.pieces_played[prev_piece.shape] += 1;
                // Pre-save whether piece was spun into lock position.
                let spin = prev_piece.fits_at(&self.state.board, (0, 1)).is_none();
                // Locking.
                for ((x, y), tile_type_id) in prev_piece.tiles() {
                    self.state.board[y][x] = Some(tile_type_id);
                }
                // Handle line clear counting for score (only do actual clearing in LineClear).
                let mut lines_cleared = Vec::<usize>::with_capacity(4);
                for y in (0..Self::HEIGHT).rev() {
                    if self.state.board[y].iter().all(|mino| mino.is_some()) {
                        lines_cleared.push(y);
                    }
                }
                let n_lines_cleared = u32::try_from(lines_cleared.len()).unwrap();
                if n_lines_cleared > 0 {
                    // Add score bonus.
                    let perfect_clear = self
                        .state
                        .board
                        .iter()
                        .all(|line| line.iter().all(|tile| tile.is_none()));
                    self.state.consecutive_line_clears += 1;
                    let special_clear = n_lines_cleared >= 4 || spin || perfect_clear;
                    if special_clear {
                        self.state.back_to_back_special_clears += 1;
                    } else {
                        self.state.back_to_back_special_clears = 0;
                    }
                    let score_bonus = 10
                        * (n_lines_cleared * n_lines_cleared)
                        * if spin { 4 } else { 1 }
                        * if perfect_clear { 16 } else { 1 }
                        * self.state.consecutive_line_clears
                        * (self.state.back_to_back_special_clears
                            * self.state.back_to_back_special_clears)
                            .max(1);
                    self.state.score += score_bonus;
                    let yippie = Feedback::Accolade {
                        score_bonus,
                        shape: prev_piece.shape,
                        spin,
                        lineclears: n_lines_cleared,
                        perfect_clear,
                        combo: self.state.consecutive_line_clears,
                        back_to_back: self.state.back_to_back_special_clears,
                    };
                    feedback_events.push((event_time, yippie));
                    feedback_events.push((
                        event_time,
                        Feedback::LineClears(lines_cleared, self.config.line_clear_delay),
                    ));
                } else {
                    self.state.consecutive_line_clears = 0;
                }
                // Clear all events and only put in line clear / appearance delay.
                self.state.events.clear();
                if n_lines_cleared > 0 {
                    self.state.events.insert(
                        InternalEvent::LineClear,
                        event_time + self.config.line_clear_delay,
                    );
                } else {
                    self.state.events.insert(
                        InternalEvent::Spawn,
                        event_time + self.config.appearance_delay,
                    );
                }
                feedback_events.push((event_time, Feedback::PieceLocked(prev_piece)));
                None
            }
            InternalEvent::LineClear => {
                for y in (0..Self::HEIGHT).rev() {
                    // Full line: move it to the cleared lines storage and push an empty line to the board.
                    if self.state.board[y].iter().all(|mino| mino.is_some()) {
                        self.state.board.remove(y);
                        self.state.board.push(Default::default());
                        self.state.lines_cleared += 1;
                    }
                }
                // Increment level if 10 lines cleared.
                if self.mode.increment_level && self.state.lines_cleared % 10 == 0 {
                    self.state.level = self.state.level.saturating_add(1);
                }
                self.state.events.insert(
                    InternalEvent::Spawn,
                    event_time + self.config.appearance_delay,
                );
                None
            }
        };
        self.state.active_piece_data = next_piece.map(|next_piece| {
            (
                next_piece,
                self.calculate_locking_data(
                    event,
                    event_time,
                    prev_piece_data,
                    next_piece,
                    next_piece.fits_at(&self.state.board, (0, -1)).is_none(),
                ),
            )
        });
        feedback_events
    }

    // TODO: THIS is, by far, the ugliest part of this entire program. For the love of what's good, I hope this code can someday be surgically excised and drop-in replaced with elegant code.
    fn calculate_locking_data(
        &mut self,
        event: InternalEvent,
        event_time: GameTime,
        prev_piece_data: Option<(ActivePiece, LockingData)>,
        next_piece: ActivePiece,
        touches_ground: bool,
    ) -> LockingData {
        /*
        Table (touches_ground):
        | ∅t0 !t1  :  [1] init locking data
        | ∅t0  t1  :  [3.1] init locking data, track touchdown etc., add LockTimer
        | !t0 !t1  :  [4]  -
        | !t0  t1  :  [3.2] track touchdown etc., add LockTimer
        |  t0 !t1  :  [2] track liftoff etc., RMV LockTimer
        |  t0  t1  :  [3.3] upon move/rot. add LockTimer
        */
        match (prev_piece_data, touches_ground) {
            // [1] Newly spawned piece does not touch ground.
            (None, false) => LockingData {
                touches_ground: false,
                last_touchdown: None,
                last_liftoff: Some(event_time),
                ground_time_left: self.config.ground_time_max,
                lowest_y: next_piece.pos.1,
            },
            // [2] Active piece lifted off the ground.
            (Some((_prev_piece, prev_locking_data)), false) if prev_locking_data.touches_ground => {
                self.state.events.remove(&InternalEvent::LockTimer);
                LockingData {
                    touches_ground: false,
                    last_liftoff: Some(event_time),
                    ..prev_locking_data
                }
            }
            // [3] A piece is on the ground. Complex update to locking values.
            (prev_piece_data, true) => {
                let next_locking_data = match prev_piece_data {
                    // If previous piece exists and next piece hasn't reached newest low (i.e. not a reset situation).
                    Some((_prev_piece, prev_locking_data))
                        if next_piece.pos.1 >= prev_locking_data.lowest_y =>
                    {
                        // Previously touched ground already, just continue previous data.
                        if prev_locking_data.touches_ground {
                            prev_locking_data
                        } else {
                            // SAFETY: We know we have an active piece that didn't touch ground before, so it MUST have its last_liftoff set.
                            let last_liftoff = prev_locking_data.last_liftoff.unwrap();
                            match prev_locking_data.last_touchdown {
                                /*
                                * `(prev_piece_data, Some((next_piece, true))) = (prev_piece_data, next_piece_dat)` [[NEXT ON GROUND]]
                                * `Some((_prev_piece, prev_locking_data)) if !(next_piece.pos.1 < prev_locking_data.lowest_y) = prev_piece_data` [[ACTIVE EXISTED, NO HEIGHT RESET]]
                                * `!prev_locking_data.touches_ground` [[PREV NOT ON GROUND]]

                                last_TD    notouch    CLOSE touchnow  :  TD = prev_locking_data.last_touchdown
                                -------    notouch    CLOSE touchnow  :  TD = Some(event_time)
                                last_TD    notouch      far touchnow  :  ground_time_left -= prev_stuff...,  TD = Some(event_time)
                                -------    notouch      far touchnow  :  TD = Some(event_time)
                                */
                                // Piece was a afloat before with valid last touchdown as well.
                                Some(last_touchdown) => {
                                    let (last_touchdown, ground_time_left) = if event_time
                                        .saturating_sub(last_liftoff)
                                        <= 2 * self.drop_delay()
                                    {
                                        (
                                            prev_locking_data.last_touchdown,
                                            prev_locking_data.ground_time_left,
                                        )
                                    } else {
                                        let elapsed_ground_time =
                                            last_liftoff.saturating_sub(last_touchdown);
                                        (
                                            Some(event_time),
                                            prev_locking_data
                                                .ground_time_left
                                                .saturating_sub(elapsed_ground_time),
                                        )
                                    };
                                    LockingData {
                                        touches_ground: true,
                                        last_touchdown,
                                        last_liftoff: None,
                                        ground_time_left,
                                        lowest_y: prev_locking_data.lowest_y,
                                    }
                                }
                                // Piece existed, was not touching ground, is touching ground now, but does not have a last touchdown. Just set touchdown.
                                None => LockingData {
                                    touches_ground: true,
                                    last_touchdown: Some(event_time),
                                    ..prev_locking_data
                                },
                            }
                        }
                    }
                    // It's a newly generated piece directly spawned on the stack, or a piece that reached new lowest and needs completely reset locking data.
                    _ => LockingData {
                        touches_ground: true,
                        last_touchdown: Some(event_time),
                        last_liftoff: None,
                        ground_time_left: self.config.ground_time_max,
                        lowest_y: next_piece.pos.1,
                    },
                };
                // Set lock timer if there isn't one, or refresh it if piece was moved.
                let repositioned = prev_piece_data
                    .map(|(prev_piece, _)| prev_piece != next_piece)
                    .unwrap_or(false);
                #[rustfmt::skip]
                let move_rotate = matches!(event, InternalEvent::Rotate | InternalEvent::MoveSlow | InternalEvent::MoveFast);
                if !self.state.events.contains_key(&InternalEvent::LockTimer)
                    || (repositioned && move_rotate)
                {
                    // SAFETY: We know this must be `Some` in this case.
                    let current_ground_time =
                        event_time.saturating_sub(next_locking_data.last_touchdown.unwrap());
                    let remaining_ground_time = next_locking_data
                        .ground_time_left
                        .saturating_sub(current_ground_time);
                    let lock_timer = std::cmp::min(self.lock_delay(), remaining_ground_time);
                    self.state
                        .events
                        .insert(InternalEvent::LockTimer, event_time + lock_timer);
                }
                next_locking_data
            }
            // [4] No change to state (afloat before and after).
            (Some((_prev_piece, prev_locking_data)), _next_piece_dat) => prev_locking_data,
        }
    }

    #[rustfmt::skip]
    const fn drop_delay(&self) -> Duration {
        Duration::from_nanos(match self.state.level.get() {
             1 => 1_000_000_000,
             2 =>   793_000_000,
             3 =>   617_796_000,
             4 =>   472_729_139,
             5 =>   355_196_928,
             6 =>   262_003_550,
             7 =>   189_677_245,
             8 =>   134_734_731,
             9 =>    93_882_249,
            10 =>    64_151_585,
            11 =>    42_976_258,
            12 =>    28_217_678,
            13 =>    18_153_329,
            14 =>    11_439_342,
            15 =>     7_058_616,
            16 =>     4_263_557,
            17 =>     2_520_084,
            18 =>     1_457_139,
             _ =>       823_907, // NOTE: 20G is at `833_333`, but falling speeds at that level are handled especially by the engine.
        })
    }

    #[rustfmt::skip]
    const fn lock_delay(&self) -> Duration {
        Duration::from_millis(match self.state.level.get() {
            1..=19 => 500,
                20 => 450,
                21 => 400,
                22 => 350,
                23 => 300,
                24 => 250,
                25 => 200,
                26 => 195,
                27 => 184,
                28 => 167,
                29 => 151,
                 _ => 150,
        })
    }
}

/*impl PartialOrd for FeedbackEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FeedbackEvent {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}*/

pub fn add((x0, y0): Coord, (x1, y1): Offset) -> Option<Coord> {
    Some((x0.checked_add_signed(x1)?, y0.checked_add_signed(y1)?))
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let res = add((1,2),(3,4));
        assert_eq!(res, (4,6));
    }
}*/
