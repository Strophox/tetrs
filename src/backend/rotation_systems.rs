use crate::backend::game::{Coord, Board, ActivePiece, Tetromino, Orientation};

pub type RotateFn = fn(board: Board, piece: ActivePiece, right_turns: i32) -> Option<ActivePiece>;
type Offset = (isize,isize);

fn add((x0,y0): Coord, (x1,y1): Offset) -> Option<Coord> {
    Some((x0.checked_add_signed(x1)?, y0.checked_add_signed(y1)?))
}

fn first_valid_kick(board: Board, old_piece: ActivePiece, right_turns: i32, kicks: impl IntoIterator<Item=Offset>) -> Option<ActivePiece> {
    let ActivePiece(shape, o, pos) = old_piece;
    kicks.into_iter().find_map(|offset| {
        let new_piece = ActivePiece(shape, o.rotate_r(right_turns), add(pos, offset)?);
        if new_piece.fits(board) {
            Some(new_piece)
        } else {
            None
        }
    })
}

pub fn rotate_dummy(board: Board, piece: ActivePiece, right_turns: i32) -> Option<ActivePiece> {
    first_valid_kick(board, piece, right_turns, std::iter::once((0,0)))
}

pub fn rotate_classic(board: Board, piece: ActivePiece, right_turns: i32) -> Option<ActivePiece> {
    let ActivePiece(shape, o, pos) = piece;
    let r = match right_turns.rem_euclid(4) {
        // No rotation
        0 => return Some(piece),
        // Right rotation
        1 => true, 
        // 180 rotation doesn't exist, so we just try default 180 rotation
        2 => return first_valid_kick(board, piece, 2, std::iter::once((0,0))),
        // Left rotation
        3 => false,
        _ => unreachable!()
    };
    use Orientation::*;
    let kick = match shape {
        Tetromino::O => (0,0), // ⠶
        Tetromino::I => match o {
            N | S => (2,-1), // ⠤⠤ -> ⡇
            E | W => (-2,1), // ⡇  -> ⠤⠤
        },
        Tetromino::S | Tetromino::Z => match o {
            N | S => (1,0), // ⠴⠂ -> ⠳ // ⠲⠄ -> ⠞
            E | W => (-1,0), // ⠳  -> ⠴⠂ // ⠞  -> ⠲⠄
        },
        Tetromino::T | Tetromino::L | Tetromino::J => match o {
            N => if r {(1,-1)} else {(-1,1)}, // ⠴⠄ <-> ⠗ // ⠤⠆ <-> ⠧ // ⠦⠄ <-> ⠏
            E => if r {(-1,0)} else {(1,0)}, // ⠗  <-> ⠲⠂ // ⠧  <-> ⠖⠂ // ⠏  <-> ⠒⠆
            S => (0,0), // ⠲⠂ <-> ⠺ // ⠖⠂ <-> ⠹ // ⠒⠆ <-> ⠼
            W => if r {(0,1)} else {(0,-1)}, // ⠺  <-> ⠴⠄ // ⠹  <-> ⠤⠆ // ⠼  <-> ⠦⠄
        },
    };
    first_valid_kick(board, piece, right_turns, std::iter::once(kick))
}

// TODO The 'Leaning' Rotation System.
/*pub fn rotate_leaning(board: Board, piece: ActivePiece, right_turns: i32) -> Option<ActivePiece> {
    let ActivePiece(shape, o, pos) = piece;
    let r = match right_turns.rem_euclid(4) {
        0 => return Some(piece),
        1 => true,
        2 => return first_valid_kick(board, ActivePiece(shape, o.rotate_r(2), pos), 2, std::iter::once((0,0))),
        3 => false,
        _ => unreachable!()
    };
    use Orientation::*;
    let kick = match shape {
        Tetromino::O => (0,0), // ⠶ -> ⠶
        Tetromino::I => match o {
            N | S => (2,2), // ⠤⠤ -> ⡇
            E | W => (-2,-2), // ⡇  -> ⠤⠤
        },
        Tetromino::S => match o {
            N | S => (1,1), // ⠴⠂ -> ⠳ 
            E | W => (-1,-1), // ⠳  -> ⠴⠂
        },
        Tetromino::Z => match o {
            N | S => (1,1), // ⠲⠄ -> ⠞
            E | W => (-1,-1), // ⠞  -> ⠲⠄
        },
        Tetromino::T => match o {
            N => if r {(1,0)} else {(-1,0)}, // ⠴⠄ -> ⠗
            E => if r {(-1,-1)} else {(1,1)}, // ⠗  -> ⠲⠂
            S => if r {(0,1)} else {(0,-1)}, // ⠲⠂ -> ⠺
            W => (0,0), // ⠺  -> ⠴⠄
        },
        Tetromino::L => match o {
            N => if r {(1,0)} else {(-1,0)}, // ⠤⠆ -> ⠧
            E => if r {(-1,-1)} else {(1,1)}, // ⠧  -> ⠖⠂
            S => if r {(0,1)} else {(0,-1)}, // ⠖⠂ -> ⠹
            W => (0,0), // ⠹  -> ⠤⠆
        },
        Tetromino::J => match o {
            N => if r {(1,0)} else {(-1,0)}, // ⠦⠄ -> ⠏
            E => if r {(-1,-1)} else {(1,1)}, // ⠏  -> ⠒⠆
            S => if r {(0,1)} else {(0,-1)}, // ⠒⠆ -> ⠼
            W => (0,0), // ⠼  -> ⠦⠄
        },
    };
    first_valid_kick(board, piece, 1, std::iter::once(kick))
}*/

// TODO The Super Rotation System.
// pub fn rotate_super(board: Board, piece: ActivePiece, right_turns: i32) -> Option<ActivePiece> {}

// TODO The Arika Rotation System.
// pub fn rotate_arika(board: Board, piece: ActivePiece, right_turns: i32) -> Option<ActivePiece> {}