use crate::day17::Move::{Left, Right};
use eyre::eyre;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
struct Point {
    x: isize,
    y: isize,
}

// 0,0 : lower_left
const PIECES: [&[Point]; 5] = [
    &[
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 2, y: 0 },
        Point { x: 3, y: 0 },
    ],
    &[
        Point { x: 1, y: 0 },
        Point { x: 0, y: 1 },
        Point { x: 1, y: 1 },
        Point { x: 2, y: 1 },
        Point { x: 1, y: 2 },
    ],
    &[
        Point { x: 2, y: 2 },
        Point { x: 2, y: 1 },
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 2, y: 0 },
    ],
    &[
        Point { x: 0, y: 0 },
        Point { x: 0, y: 1 },
        Point { x: 0, y: 2 },
        Point { x: 0, y: 3 },
    ],
    &[
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 0, y: 1 },
        Point { x: 1, y: 1 },
    ],
];
const PIECES_NB: usize = PIECES.len();

#[derive(Debug, Copy, Clone)]
enum Move {
    Left,
    Right,
}

impl FromStr for Move {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some(l) if l == '<' => Ok(Left),
            Some(r) if r == '>' => Ok(Right),
            _ => Err(eyre!("cannot parse {s} as Move")),
        }
    }
}

#[derive(Debug)]
struct TetrisPiece<'t> {
    // upper_left : 0,0
    rocks: &'t [Point],
    pos: Point,
}

#[derive(Debug, Clone)]
struct Cave {
    rocks: VecDeque<[bool; 7]>,
    moves: Vec<Move>,
    offset: usize,
}

impl FromStr for Cave {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let moves: Result<Vec<Move>, _> = s.chars().map(|c| c.to_string().parse()).collect();
        Ok(Self {
            moves: moves?,
            rocks: VecDeque::from(vec![]),
            offset: 0,
        })
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for rocks in self.rocks.iter().rev() {
            for r in rocks {
                f.write_char(if *r { '#' } else { ' ' })?
            }
            f.write_char('\n')?
        }

        f.write_str(&format!("--- missing {} lines ---", self.offset))
    }
}

impl Cave {
    fn move_left(&self, piece: &mut TetrisPiece) -> bool {
        for r in piece.rocks {
            let x = r.x + piece.pos.x;
            let y = piece.pos.y + r.y;
            if x == 0 || ((y as usize) < self.rocks.len() && self.rocks[y as usize][x as usize - 1])
            {
                return false;
            }
        }
        piece.pos.x -= 1;
        true
    }
    fn move_right(&self, piece: &mut TetrisPiece) -> bool {
        for r in piece.rocks {
            let x = r.x + piece.pos.x;
            let y = piece.pos.y + r.y;
            if x == 6 || ((y as usize) < self.rocks.len() && self.rocks[y as usize][x as usize + 1])
            {
                return false;
            }
        }
        piece.pos.x += 1;
        true
    }
    fn move_down(&self, piece: &mut TetrisPiece) -> bool {
        for r in piece.rocks {
            let x = r.x + piece.pos.x;
            let y = piece.pos.y + r.y;
            if y == 0 || (y as usize <= self.rocks.len() && self.rocks[y as usize - 1][x as usize])
            {
                return false;
            }
        }
        piece.pos.y -= 1;
        true
    }
    fn freeze_piece(&mut self, piece: TetrisPiece) {
        // println!("freeze piece : {piece:?}");
        let max_y = (piece.pos.y + piece.rocks.iter().map(|r| r.y).max().unwrap_or(0)) as usize;
        for _ in self.rocks.len()..=max_y {
            self.rocks.push_back([false; 7])
        }
        for r in piece.rocks {
            self.rocks[(piece.pos.y + r.y) as usize][(piece.pos.x + r.x) as usize] = true;
        }
    }

    fn stack_pieces(mut self, limit: usize) -> usize {
        let mut move_index = 0usize;
        let moves_mod = self.moves.len();
        let mut piece_index = 0usize;
        let mut period_found = false;

        // height, fall, next_move_index
        let mut stacked: Vec<(usize, Point, usize)> = vec![];

        let mut i = 0;

        while i < limit {
            // println!("i {i}");
            let pos = Point {
                x: 2,
                y: self.rocks.len() as isize + 3,
            };
            let rocks = PIECES[piece_index];
            piece_index = (piece_index + 1) % PIECES_NB;
            let mut piece = TetrisPiece { pos, rocks };
            loop {
                let next_move = &self.moves[move_index];
                move_index = (move_index + 1) % moves_mod;
                match *next_move {
                    Left => self.move_left(&mut piece),
                    Right => self.move_right(&mut piece),
                };
                if !self.move_down(&mut piece) {
                    break;
                }
            }

            let fall = Point {
                x: pos.x - piece.pos.x,
                y: pos.y - piece.pos.y,
            };
            self.freeze_piece(piece);
            let height = self.offset + self.rocks.len();
            stacked.push((height, fall, move_index));

            if !period_found {
                let proof_len = PIECES_NB * moves_mod;
                'search: for (j, (h, p, m_index)) in stacked.iter().enumerate().skip(proof_len) {
                    if j != i
                        && *p == fall
                        && *m_index == move_index
                        && (0..proof_len).all(|k| {
                            let (_, previous_p, previous_m_index) = stacked[j - k];
                            let (_, new_p, new_m_index) = stacked[stacked.len() - 1 - k];
                            previous_p == new_p && previous_m_index == new_m_index
                        })
                    {
                        period_found = true;
                        let period = i - j;
                        // println!("period checked against {} values : {reason}", proof_len);

                        let skip_runs: usize = (limit - i) / period;
                        i += skip_runs * period;
                        self.offset += (height - h) * (skip_runs);
                        break 'search;
                    }
                }
            }
            i += 1;
        }
        // println!("{}", self);
        self.offset + self.rocks.len()
    }
}

pub fn tetris_rock() {
    let input = include_str!("../resources/day17_moves.txt");
    let cave: Cave = input.parse().unwrap();

    let height = cave.clone().stack_pieces(2022);
    println!("height after 2022 rocks {height}");
    let height = cave.stack_pieces(1000000000000);
    println!("height after 1000000000000 rocks {height}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aoc_example_works() {
        let input = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";
        let cave: Cave = input.parse().unwrap();
        assert_eq!(1, cave.clone().stack_pieces(1));
        assert_eq!(4, cave.clone().stack_pieces(2));
        assert_eq!(3068, cave.clone().stack_pieces(2022));
        assert_eq!(1514285714288, cave.clone().stack_pieces(1000000000000));
    }
}
