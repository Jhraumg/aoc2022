use crate::day22::Direction::{East, North, South, West};
use crate::day22::Step::{Forward, Left, Right};
use crate::day22::Tile::{Open, Wall};
use eyre::{eyre, ContextCompat, WrapErr};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}
impl Direction {
    fn left(self) -> Self {
        match self {
            North => West,
            South => East,
            East => North,
            West => South,
        }
    }
    fn right(self) -> Self {
        match self {
            North => East,
            South => West,
            East => South,
            West => North,
        }
    }

    fn opposite(&self) -> Self {
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Open,
    Wall,
}

impl FromStr for Tile {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.chars().next() {
            Some('.') => Ok(Open),
            Some('#') => Ok(Wall),
            _ => Err(eyre!("cannot convert '{s}' to tile")),
        }
    }
}

#[derive(Debug, Clone)]
struct MapRow {
    offset: usize,
    tiles: Vec<Tile>,
}
impl MapRow {
    fn get_tile(&self, column: usize) -> Option<Tile> {
        if column >= self.offset && column < self.offset + self.tiles.len() {
            Some(self.tiles[column - self.offset])
        } else {
            None
        }
    }
}
impl FromStr for MapRow {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let offset = s.chars().take_while(|c| *c == ' ').count();
        let tiles: Result<Vec<Tile>, _> = s[offset..]
            .chars()
            .map(|c| String::from(c).parse())
            .collect();
        let tiles = tiles?;
        Ok(Self { offset, tiles })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Pos {
    column: usize,
    row: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Step {
    Left,
    Right,
    Forward(usize),
}

impl FromStr for Step {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next().with_context(|| "reading first char ")? {
            'L' => Ok(Left),
            'R' => Ok(Right),
            _ => s
                .parse::<usize>()
                .map(Forward)
                .with_context(|| eyre!("reading value from '{s}'")),
        }
    }
}
#[derive(Debug, Clone)]
struct Map {
    rows: Vec<MapRow>,
    path: Vec<Step>,
}

impl FromStr for Map {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows: Result<Vec<MapRow>, _> = s
            .lines()
            .take_while(|l| l.starts_with([' ', '.', '#']))
            .map(str::parse)
            .collect();
        let rows = rows?;
        let path_line = s
            .lines()
            .skip(rows.len())
            .find(|l| !l.trim().is_empty())
            .with_context(|| "reading path")?;

        let path_indices: Vec<_> = path_line.match_indices(&['L', 'R']).collect();
        let mut path: Vec<Step> = vec![];
        let mut current_index = 0usize;

        for (idx, rotation) in path_indices {
            if idx > current_index {
                path.push(path_line[current_index..idx].parse()?);
            }
            path.push(rotation.parse()?);
            current_index = idx + rotation.len();
        }
        if current_index < path_line.len() {
            path.push(path_line[current_index..].parse()?);
        }

        Ok(Self { rows, path })
    }
}

impl Map {
    fn get_tile(&self, pos: Pos) -> Option<Tile> {
        self.rows[pos.row % self.rows.len()].get_tile(pos.column)
    }
    fn mv(&self, absolute_start: Pos, dir: Direction, len: usize) -> Pos {
        let mut pos = absolute_start;
        for _ in 0..len {
            let new_pos = match dir {
                North => self.mv_up(pos),
                South => self.mv_down(pos),
                East => self.mv_right(pos),
                West => self.mv_left(pos),
            };
            if new_pos == pos {
                break;
            }
            pos = new_pos;
        }
        debug_assert_eq!(
            Some(Open),
            self.get_tile(pos),
            "{absolute_start:?},{dir:?} crashed into wall"
        );
        pos
    }

    fn mv_left(&self, absolute_start: Pos) -> Pos {
        let current_row = &self.rows[absolute_start.row];
        let mut column = absolute_start.column - current_row.offset;
        let new_col = (column + current_row.tiles.len() - 1) % current_row.tiles.len();
        // safety: new_col is built to bound to [0, currrent_row.len [
        if Open == unsafe { *current_row.tiles.get_unchecked(new_col) } {
            column = new_col;
        }
        Pos {
            row: absolute_start.row,
            column: column + current_row.offset,
        }
    }
    fn mv_right(&self, absolute_start: Pos) -> Pos {
        let current_row = &self.rows[absolute_start.row];
        let mut column = absolute_start.column - current_row.offset;
        let new_col = (column + 1) % current_row.tiles.len();
        // safety: new_col is built to bound to [0, currrent_row.len [
        if Open == unsafe { *current_row.tiles.get_unchecked(new_col) } {
            column = new_col;
        }
        Pos {
            row: absolute_start.row,
            column: column + current_row.offset,
        }
    }

    fn mv_up(&self, absolute_start: Pos) -> Pos {
        // if previous row match current column : keep it
        let immediate_up_row_index = (absolute_start.row + self.rows.len() - 1) % self.rows.len();

        // safety : index forced to valid range
        let immediate_up_row = unsafe { self.rows.get_unchecked(immediate_up_row_index) };
        match immediate_up_row.get_tile(absolute_start.column) {
            Some(Wall) => absolute_start,
            Some(Open) => Pos {
                column: absolute_start.column,
                row: immediate_up_row_index,
            },
            None => {
                // lets found the last one going down
                let mut last_down_row_index = absolute_start.row;
                loop {
                    let next_down_index = (last_down_row_index + 1) % self.rows.len();
                    if self
                        .rows
                        .get(next_down_index)
                        .and_then(|row| row.get_tile(absolute_start.column))
                        .is_some()
                    {
                        last_down_row_index = next_down_index;
                    } else {
                        break;
                    }
                }
                match self.rows[last_down_row_index].get_tile(absolute_start.column) {
                    Some(Wall) => absolute_start,
                    Some(Open) => Pos {
                        column: absolute_start.column,
                        row: last_down_row_index,
                    },
                    _ => panic!("should not append, we checked for it juste before"),
                }
            }
        }
    }

    // FIXME : factorize
    fn mv_down(&self, absolute_start: Pos) -> Pos {
        // if previous row match current column : keep it
        let immediate_down_row_index = (absolute_start.row + 1) % self.rows.len();

        // safety : index forced to valid range
        let immediate_down_row = unsafe { self.rows.get_unchecked(immediate_down_row_index) };
        match immediate_down_row.get_tile(absolute_start.column) {
            Some(Wall) => absolute_start,
            Some(Open) => Pos {
                column: absolute_start.column,
                row: immediate_down_row_index,
            },
            None => {
                // lets found the last one going up
                let mut last_up_row_index = absolute_start.row;
                loop {
                    let next_up_index = (last_up_row_index + self.rows.len() - 1) % self.rows.len();
                    if self
                        .rows
                        .get(next_up_index)
                        .and_then(|row| row.get_tile(absolute_start.column))
                        .is_some()
                    {
                        last_up_row_index = next_up_index;
                    } else {
                        break;
                    }
                }
                match self.rows[last_up_row_index].get_tile(absolute_start.column) {
                    Some(Wall) => absolute_start,
                    Some(Open) => Pos {
                        column: absolute_start.column,
                        row: last_up_row_index,
                    },
                    _ => panic!("should not append, we checked for it juste before"),
                }
            }
        }
    }

    fn password(self) -> usize {
        let mut pos = Pos {
            column: self.rows[0].offset,
            row: 0,
        };
        let mut dir = East;

        for step in &self.path {
            match step {
                Left => {
                    dir = dir.left();
                }
                Right => {
                    dir = dir.right();
                }
                Forward(fwd) => {
                    pos = self.mv(pos, dir, *fwd);
                }
            }
        }

        1000 * (1 + pos.row)
            + 4 * (1 + pos.column)
            + match dir {
                North => 3,
                South => 1,
                East => 0,
                West => 2,
            }
    }
}

#[derive(Debug)]
struct CubeFace {
    top_left: Pos,
    bottom_right: Pos,
}
/// I guess the wrap up could be computed
///
#[derive(Debug)]
struct Cube<'m> {
    map: &'m Map,
    faces: [CubeFace; 6],
    seams_by_face_and_dir: HashMap<(usize, Direction), (usize, Direction)>,
}

impl<'m> Cube<'m> {
    fn new(
        map: &'m Map,
        faces: [CubeFace; 6],
        seams_by_face_and_dir: HashMap<(usize, Direction), (usize, Direction)>,
    ) -> Self {
        let mut new_seams_by_face_and_dir = HashMap::with_capacity(2 * seams_by_face_and_dir.len());
        for ((from, from_dir), (to, to_dir)) in seams_by_face_and_dir {
            new_seams_by_face_and_dir.insert((from, from_dir), (to, to_dir));
            new_seams_by_face_and_dir.insert((to, to_dir.opposite()), (from, from_dir.opposite()));
        }

        Self {
            map,
            faces,
            seams_by_face_and_dir: new_seams_by_face_and_dir,
        }
    }

    /// panics if point outside cube surface
    fn get_face(&self, pos: &Pos) -> usize {
        self.faces
            .iter()
            .enumerate()
            .find_map(|(i, f)| {
                if f.top_left.row <= pos.row
                    && f.bottom_right.row >= pos.row
                    && f.top_left.column <= pos.column
                    && f.bottom_right.column >= pos.column
                {
                    Some(i)
                } else {
                    None
                }
            })
            .expect("{pos} is not on cube surface")
    }

    fn mv(&self, absolute_start: Pos, dir: Direction, len: usize) -> (Pos, Direction) {
        let mut current_face_idx = self.get_face(&absolute_start);
        let mut current_dir = dir;
        let mut current_pos = absolute_start;
        // let's keep it naive for a start
        'main: for _ in 0..len {
            let current_face = &self.faces[current_face_idx];

            let face_cross = match current_dir {
                North => current_pos.row == current_face.top_left.row,
                South => current_pos.row == current_face.bottom_right.row,
                East => current_pos.column == current_face.bottom_right.column,
                West => current_pos.column == current_face.top_left.column,
            };
            let seam_cross = face_cross
                && self
                    .seams_by_face_and_dir
                    .contains_key(&(current_face_idx, current_dir));
            if !seam_cross {
                let new_pos = self.map.mv(current_pos, current_dir, 1);
                if new_pos == current_pos {
                    break 'main;
                }
                if face_cross {
                    current_face_idx = self.get_face(&new_pos);
                }
                current_pos = new_pos;
                continue 'main;
            }
            if let Some((new_face_idx, new_dir)) = self
                .seams_by_face_and_dir
                .get(&(current_face_idx, current_dir))
            {
                let relative_pos = match current_dir {
                    North => current_pos.column - current_face.top_left.column,
                    South => current_face.bottom_right.column - current_pos.column,
                    East => current_pos.row - current_face.top_left.row,
                    West => current_face.bottom_right.row - current_pos.row,
                };

                let new_face = &self.faces[*new_face_idx];
                let new_pos = match new_dir {
                    North => Pos {
                        column: new_face.top_left.column + relative_pos,
                        row: new_face.bottom_right.row,
                    },
                    South => Pos {
                        column: new_face.bottom_right.column - relative_pos,
                        row: new_face.top_left.row,
                    },
                    East => Pos {
                        column: new_face.top_left.column,
                        row: new_face.top_left.row + relative_pos,
                    },
                    West => Pos {
                        column: new_face.bottom_right.column,
                        row: new_face.bottom_right.row - relative_pos,
                    },
                };

                match self.map.get_tile(new_pos) {
                    Some(Tile::Open) => {
                        current_pos = new_pos;
                        current_dir = *new_dir;
                        current_face_idx = *new_face_idx;
                    }
                    Some(Tile::Wall) => {
                        break 'main;
                    }
                    None => {
                        panic!("{current_pos:?}, {dir:?} => {new_pos:?} which is not in map !");
                    }
                }
            } else {
                panic!("should not happens, we were supposed to cross a seam with ({current_face_idx}, {current_dir:?})" );
            }
        }
        // dbg!((pos,current_dir))
        (current_pos, current_dir)
    }

    fn apply_path(&self) -> usize {
        let max_fwd = self
            .map
            .path
            .iter()
            .filter_map(|step| {
                if let Forward(fwd) = *step {
                    Some(fwd)
                } else {
                    None
                }
            })
            .max()
            .unwrap();
        let face_size = self.faces[0].bottom_right.column - self.faces[0].top_left.column + 1;
        assert!(
            max_fwd <= face_size,
            "{max_fwd} > {face_size} :move from more than one face at once must be split"
        );

        let mut pos = self.faces[0].top_left;
        let mut dir = Direction::East;

        let max_column = self
            .map
            .rows
            .iter()
            .map(|r| r.offset + r.tiles.len())
            .max()
            .unwrap();
        let max_row = self.map.rows.len();
        let mut repr = vec![vec![' '; max_column]; max_row];
        for (j, row) in self.map.rows.iter().enumerate() {
            for (i, t) in row.tiles.iter().enumerate() {
                repr[j][i + row.offset] = match t {
                    Open => '.',
                    Wall => '#',
                }
            }
        }

        for step in &self.map.path {
            match step {
                Left => {
                    dir = dir.left();
                }
                Right => {
                    dir = dir.right();
                }
                &Forward(fwd) => {
                    (pos, dir) = self.mv(pos, dir, fwd);
                }
            }
        }

        1000 * (1 + pos.row)
            + 4 * (1 + pos.column)
            + match dir {
                North => 3,
                South => 1,
                East => 0,
                West => 2,
            }
    }
}

pub fn decode_password() {
    let input = include_str!("../resources/day22_map_password.txt");
    let map: Map = input.parse().unwrap();
    let password = map.clone().password();
    println!("password : {password}");

    let cube = Cube::new(
        &map,
        [
            CubeFace {
                top_left: Pos {
                    row: 0,
                    column: 100,
                },
                bottom_right: Pos {
                    row: 49,
                    column: 149,
                },
            },
            CubeFace {
                top_left: Pos { row: 0, column: 50 },
                bottom_right: Pos {
                    row: 49,
                    column: 99,
                },
            },
            CubeFace {
                top_left: Pos {
                    row: 50,
                    column: 50,
                },
                bottom_right: Pos {
                    row: 99,
                    column: 99,
                },
            },
            CubeFace {
                top_left: Pos {
                    row: 100,
                    column: 50,
                },
                bottom_right: Pos {
                    row: 149,
                    column: 99,
                },
            },
            CubeFace {
                top_left: Pos {
                    row: 100,
                    column: 0,
                },
                bottom_right: Pos {
                    row: 149,
                    column: 49,
                },
            },
            CubeFace {
                top_left: Pos {
                    row: 150,
                    column: 0,
                },
                bottom_right: Pos {
                    row: 199,
                    column: 49,
                },
            },
        ],
        HashMap::from([
            ((0, North), (5, North)),
            ((0, East), (3, West)),
            ((0, South), (2, West)),
            ((1, North), (5, East)),
            ((1, West), (4, East)),
            ((2, West), (4, South)),
            ((3, South), (5, West)),
        ]),
    );

    let cube_password = cube.apply_path();
    println!("cube_password : {cube_password}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
                    ...#
                    .#..
                    #...
                    ....
            ...#.......#
            ........#...
            ..#....#....
            ..........#.
                    ...#....
                    .....#..
                    .#......
                    ......#.

            10R5L5R10L4R5L5
        "};

        let map: Map = input.parse().unwrap();

        assert_eq!(
            vec![
                Forward(10),
                Right,
                Forward(5),
                Left,
                Forward(5),
                Right,
                Forward(10),
                Left,
                Forward(4),
                Right,
                Forward(5),
                Left,
                Forward(5)
            ],
            map.path
        );
        assert_eq!(6032, map.clone().password());

        let cube = Cube::new(
            &map,
            [
                CubeFace {
                    top_left: Pos { row: 0, column: 8 },
                    bottom_right: Pos { row: 3, column: 11 },
                },
                CubeFace {
                    top_left: Pos { row: 4, column: 8 },
                    bottom_right: Pos { row: 7, column: 11 },
                },
                CubeFace {
                    top_left: Pos { row: 4, column: 4 },
                    bottom_right: Pos { row: 7, column: 7 },
                },
                CubeFace {
                    top_left: Pos { row: 4, column: 0 },
                    bottom_right: Pos { row: 7, column: 3 },
                },
                CubeFace {
                    top_left: Pos { row: 8, column: 8 },
                    bottom_right: Pos {
                        row: 11,
                        column: 11,
                    },
                },
                CubeFace {
                    top_left: Pos { row: 8, column: 12 },
                    bottom_right: Pos {
                        row: 11,
                        column: 15,
                    },
                },
            ],
            HashMap::from([
                ((0, North), (3, South)),
                ((0, West), (2, South)),
                ((0, East), (5, West)),
                ((1, East), (5, South)),
                ((2, South), (4, East)),
                ((3, West), (5, North)),
                ((3, South), (4, North)),
            ]),
        );
        dbg!(&cube);
        assert_eq!(5031, cube.apply_path());
    }
}
