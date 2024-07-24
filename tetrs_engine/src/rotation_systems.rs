// TODO: Implement Super Rotation System.
use crate::{ActivePiece, Board, Orientation, Tetromino};

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RotationSystem {
    Ok,
    Super,
    Classic,
}

impl RotationSystem {
    pub fn rotate(
        &self,
        piece: &ActivePiece,
        board: &Board,
        right_turns: i32,
    ) -> Option<ActivePiece> {
        match self {
            RotationSystem::Classic => rotate_classic(piece, board, right_turns),
            RotationSystem::Super => rotate_super(piece, board, right_turns),
            RotationSystem::Ok => rotate_ok(piece, board, right_turns),
        }
    }

    pub fn place_initial(&mut self, shape: Tetromino) -> ActivePiece {
        let pos = match shape {
            Tetromino::O => (4, 20),
            _ => (3, 20),
        };
        let orientation = Orientation::N;
        /* TODO: Custom, but weird, spawn positions/orientation.
        let (orientation, pos) = match shape {
            Tetromino::O => (Orientation::N, (4, 20)),
            Tetromino::I => (Orientation::N, (3, 20)),
            Tetromino::S => (Orientation::E, (4, 20)),
            Tetromino::Z => (Orientation::W, (4, 20)),
            Tetromino::T => (Orientation::N, (3, 20)),
            Tetromino::L => (Orientation::E, (4, 20)),
            Tetromino::J => (Orientation::W, (4, 20)),
        };*/
        ActivePiece {
            shape,
            orientation,
            pos,
        }
    }
}

fn rotate_ok(piece: &ActivePiece, board: &Board, right_turns: i32) -> Option<ActivePiece> {
    /*
    Symmetries : "OISZTLJ NESW ↺↻" and "-" mirror.
    O N      :
        (No kicks.)
    I NE   ↺ :
        I NE ↻  = -I NE ↺
    S NE   ↺↻:
        Z NE ↺↻ = -S NE ↻↺
    T NESW ↺ :
        T NS ↻  = -T NS ↺
        T EW ↻  = -T WE ↺
    L NESW ↺↻:
        J NS ↺↻ = -L NS ↻↺
        J EW ↺↻ = -L WE ↻↺
    */
    let mut left = match right_turns.rem_euclid(4) {
        // No rotation occurred.
        0 => return Some(*piece),
        // One right rotation.
        1 => false,
        // 180 rotation will behave like two free-air rotations in a single press.
        2 => {
            // TODO: Implement 180 rotation.
            #[rustfmt::skip]
            let kicks = match piece.shape {
                Tetromino::O | Tetromino::I | Tetromino::S | Tetromino::Z => [(0, 0)].iter(),
                Tetromino::T | Tetromino::L | Tetromino::J => match piece.orientation {
                    N => [( 0,-1), ( 0, 0)].iter(),
                    E => [(-1, 0), ( 0, 0)].iter(),
                    S => [( 0, 1), ( 0, 0)].iter(),
                    W => [( 1, 0), ( 0, 0)].iter(),
                },
            }.copied();
            return piece.first_fit(board, kicks, 2);
        }
        // One left rotation.
        3 => true,
        _ => unreachable!(),
    };
    let (mut mirror, mut shape, mut orientation) = (None, piece.shape, piece.orientation);
    use Orientation::*;
    #[rustfmt::skip]
    let dual_orientation = match orientation {
        N => N, E => W, S => S, W => E,
    };
    #[rustfmt::skip]
    let kicks = 'calculate_kicks: loop {
        match shape {
            Tetromino::O => break [( 0, 0)].iter(),
            Tetromino::I => {
                if !left {
                    let mx = match orientation {
                        N | S => 3, E | W => -3,
                    };
                    (mirror, left) = (Some(mx), !left);
                    continue 'calculate_kicks;
                } else  {
                    break match orientation {
                        N | S => [( 1,-1), ( 1,-2), ( 1,-3), ( 0,-1), ( 2,-1), ( 1, 1)].iter(),
                        E | W => [(-2, 1), (-3, 1), (-1, 1), ( 0, 1), (-2, 0), (-3, 0)].iter(),
                    };
                }
            },
            Tetromino::S => break match orientation {
                N | S => if left { [( 0, 0), ( 0,-1), ( 1, 0), (-1,-1)].iter() }
                            else { [( 1, 0), ( 1,-1), ( 0, 0), ( 0,-1)].iter() },
                E | W => if left { [(-1, 0), ( 0, 0), (-1, 1), ( 0, 1)].iter() }
                            else { [( 0, 0), ( 0,-1), (-1, 0), ( 0, 1), (-1, 1)].iter() },
            },
            Tetromino::Z => {
                let mx = match orientation {
                    N | S => 1, E | W => -1,
                };
                (mirror, shape, left) = (Some(mx), Tetromino::S, !left);
                continue 'calculate_kicks;
            },
            Tetromino::T => {
                if !left {
                    let mx = match orientation {
                        N | S => 1, E | W => -1,
                    };
                    (mirror, orientation, left) = (Some(mx), dual_orientation, !left);
                    continue 'calculate_kicks;
                } else  {
                    break match orientation {
                        N => [( 0,-1), ( 0, 0), ( 1,-1), ( 1, 0)].iter(),
                        E => [(-1, 0), (-1,-1), ( 0, 0), ( 0,-1)].iter(),
                        S => [( 1, 0), ( 0, 0), ( 0,-1), ( 1,-1)].iter(),
                        W => [( 0, 0), (-1, 0), (-1,-1), ( 0,-1)].iter(),
                    };
                }
            },
            Tetromino::L => break match orientation {
                N => if left { [( 0,-1), ( 1,-1), ( 0, 0), ( 0,-1), ( 1, 0)].iter() }
                        else { [( 1,-1), ( 1, 0), ( 2, 0), ( 0, 0)].iter() },
                E => if left { [(-1, 1), (-1, 0), ( 0, 1), (-2, 0), ( 0, 0)].iter() }
                        else { [(-1, 0), ( 0, 0), ( 0,-1), ( 0, 1)].iter() },
                S => if left { [( 1, 0), ( 0, 0), ( 1,-1), ( 0,-1)].iter() }
                        else { [( 0, 0), ( 1, 0), ( 0,-1), (-1,-1), ( 1,-1)].iter() },
                W => if left { [( 0, 0), (-1, 0), ( 0, 1), ( 1, 0), (-1, 1)].iter() }
                        else { [( 0, 1), ( 0, 0), (-1, 1), (-1, 0)].iter() },
            },
            Tetromino::J => {
                let mx = match orientation {
                    N | S => 1, E | W => -1,
                };
                (mirror, shape, orientation, left) = (Some(mx), Tetromino::L, dual_orientation, !left);
                continue 'calculate_kicks;
            }
        }
    }.copied();
    if let Some(mx) = mirror {
        piece.first_fit(board, kicks.map(|(x, y)| (mx - x, y)), right_turns)
    } else {
        piece.first_fit(board, kicks, right_turns)
    }
}

fn rotate_super(piece: &ActivePiece, board: &Board, right_turns: i32) -> Option<ActivePiece> {
    // TODO: Implement SRS.
    todo!();
    let left_rotation = match right_turns.rem_euclid(4) {
        // No rotation occurred.
        0 => return Some(*piece),
        // One right rotation.
        1 => false,
        // Classic didn't define 180 rotation, just check if the "default" 180 rotation fits.
        2 => {
            return piece.fits_at_rotated(board, (0, 0), 2);
        }
        // One left rotation.
        3 => true,
        _ => unreachable!(),
    };
    use Orientation::*;
    #[rustfmt::skip]
    let kick = match piece.shape {
        Tetromino::O => (0, 0), // ⠶
        Tetromino::I => match piece.orientation {
            N | S => (2, -1), // ⠤⠤ -> ⡇
            E | W => (-2, 1), // ⡇  -> ⠤⠤
        },
        Tetromino::S | Tetromino::Z => match piece.orientation {
            N | S => (1, 0),  // ⠴⠂ -> ⠳  // ⠲⠄ -> ⠞
            E | W => (-1, 0), // ⠳  -> ⠴⠂ // ⠞  -> ⠲⠄
        },
        Tetromino::T | Tetromino::L | Tetromino::J => match piece.orientation {
            N => if left_rotation { ( 0,-1) } else { ( 1,-1) }, // ⠺  <- ⠴⠄ -> ⠗  // ⠹  <- ⠤⠆ -> ⠧  // ⠼  <- ⠦⠄ -> ⠏
            E => if left_rotation { (-1, 1) } else { (-1, 0) }, // ⠴⠄ <- ⠗  -> ⠲⠂ // ⠤⠆ <- ⠧  -> ⠖⠂ // ⠦⠄ <- ⠏  -> ⠒⠆
            S => if left_rotation { ( 1, 0) } else { ( 0, 0) }, // ⠗  <- ⠲⠂ -> ⠺  // ⠧  <- ⠖⠂ -> ⠹  // ⠏  <- ⠒⠆ -> ⠼
            W => if left_rotation { ( 0, 0) } else { ( 0, 1) }, // ⠲⠂ <- ⠺  -> ⠴⠄ // ⠖⠂ <- ⠹  -> ⠤⠆ // ⠒⠆ <- ⠼  -> ⠦⠄
        },
    };
    piece.fits_at_rotated(board, kick, right_turns)
}

fn rotate_classic(piece: &ActivePiece, board: &Board, right_turns: i32) -> Option<ActivePiece> {
    let left_rotation = match right_turns.rem_euclid(4) {
        // No rotation occurred.
        0 => return Some(*piece),
        // One right rotation.
        1 => false,
        // Classic didn't define 180 rotation, just check if the "default" 180 rotation fits.
        2 => {
            return piece.fits_at_rotated(board, (0, 0), 2);
        }
        // One left rotation.
        3 => true,
        _ => unreachable!(),
    };
    use Orientation::*;
    #[rustfmt::skip]
    let kick = match piece.shape {
        Tetromino::O => (0, 0), // ⠶
        Tetromino::I => match piece.orientation {
            N | S => (2, -1), // ⠤⠤ -> ⡇
            E | W => (-2, 1), // ⡇  -> ⠤⠤
        },
        Tetromino::S | Tetromino::Z => match piece.orientation {
            N | S => (1, 0),  // ⠴⠂ -> ⠳  // ⠲⠄ -> ⠞
            E | W => (-1, 0), // ⠳  -> ⠴⠂ // ⠞  -> ⠲⠄
        },
        Tetromino::T | Tetromino::L | Tetromino::J => match piece.orientation {
            N => if left_rotation { ( 0,-1) } else { ( 1,-1) }, // ⠺  <- ⠴⠄ -> ⠗  // ⠹  <- ⠤⠆ -> ⠧  // ⠼  <- ⠦⠄ -> ⠏
            E => if left_rotation { (-1, 1) } else { (-1, 0) }, // ⠴⠄ <- ⠗  -> ⠲⠂ // ⠤⠆ <- ⠧  -> ⠖⠂ // ⠦⠄ <- ⠏  -> ⠒⠆
            S => if left_rotation { ( 1, 0) } else { ( 0, 0) }, // ⠗  <- ⠲⠂ -> ⠺  // ⠧  <- ⠖⠂ -> ⠹  // ⠏  <- ⠒⠆ -> ⠼
            W => if left_rotation { ( 0, 0) } else { ( 0, 1) }, // ⠲⠂ <- ⠺  -> ⠴⠄ // ⠖⠂ <- ⠹  -> ⠤⠆ // ⠒⠆ <- ⠼  -> ⠦⠄
        },
    };
    piece.fits_at_rotated(board, kick, right_turns)
}
