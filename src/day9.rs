use crate::day9::Direction::{Down, Left, Right, Up};
use eyre::{eyre, ContextCompat};
use itertools::Itertools;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
struct Point {
    x: isize,
    y: isize,
}

#[derive(Debug, Clone, Copy)]
struct Knot {
    head: Point,
    tail: Point,
}
#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn get_d(&self) -> (isize, isize) {
        match self {
            Up => (0, 1),
            Down => (0, -1),
            Left => (-1, 0),
            Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Move {
    dir: Direction,
    len: usize,
}
impl FromStr for Move {
    type Err = eyre::Error;
    fn from_str(s: &str) -> eyre::Result<Self> {
        let mut items = s.split(' ');
        let d = items
            .next()
            .and_then(|item| item.chars().next())
            .context("trying to read dir")?;
        let dir = match d {
            'D' => Ok(Down),
            'U' => Ok(Up),
            'L' => Ok(Left),
            'R' => Ok(Right),
            _ => Err(eyre!("cannot parse Direction from {d}")),
        }?;
        let len: usize = items.next().context("trying to read len")?.parse()?;
        Ok(Self { dir, len })
    }
}

impl Knot {
    pub fn new() -> Self {
        Self {
            head: Point { x: 0, y: 0 },
            tail: Point { x: 0, y: 0 },
        }
    }
    fn move_head_d(&mut self, dir: (isize, isize)) {
        let (dx, dy) = dir;
        assert!(dx.abs() + dy.abs() <= 2);
        self.head.x += dx;
        self.head.y += dy;

        let (dx, dy) = (self.head.x - self.tail.x, self.head.y - self.tail.y);
        if dx.abs() > 1 {
            self.tail.x += dx.signum();
        }
        if dy.abs() > 1 {
            self.tail.y += dy.signum();
        }
        if dx.abs() > dy.abs() {
            self.tail.y = self.head.y;
        }
        if dy.abs() > dx.abs() {
            self.tail.x = self.head.x;
        }
    }
}
impl Display for Knot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "[{},{}] <- [{},{}]",
            self.head.x, self.head.y, self.tail.x, self.tail.y
        ))
    }
}

#[allow(dead_code)]
fn display_knots(knots: &[Knot]) -> String {
    let min_x = min(
        -12,
        knots
            .iter()
            .flat_map(|k| [k.head.x, k.tail.x].into_iter())
            .min()
            .unwrap(),
    );
    let min_y = min(
        -12,
        knots
            .iter()
            .flat_map(|k| [k.head.y, k.tail.y].into_iter())
            .min()
            .unwrap(),
    );
    let max_x = max(
        12,
        knots
            .iter()
            .flat_map(|k| [k.head.x, k.tail.x].into_iter())
            .max()
            .unwrap(),
    );
    let max_y = max(
        12,
        knots
            .iter()
            .flat_map(|k| [k.head.y, k.tail.y].into_iter())
            .max()
            .unwrap(),
    );

    let mut pos = vec![vec![None; (max_x - min_x + 1) as usize]; (max_y - min_y + 1) as usize];
    for (i, knot) in knots.iter().enumerate() {
        if pos[(knot.head.y - min_y) as usize][(knot.head.x - min_x) as usize].is_none() {
            pos[(knot.head.y - min_y) as usize][(knot.head.x - min_x) as usize] =
                Some(i.to_string());
        }
    }
    let knot = knots[knots.len() - 1];
    if pos[(knot.tail.y - min_y) as usize][(knot.tail.x - min_x) as usize].is_none() {
        pos[(knot.tail.y - min_y) as usize][(knot.tail.x - min_x) as usize] = Some("t".to_string());
    }
    if pos[(0 - min_y) as usize][(0 - min_x) as usize].is_none() {
        pos[(0 - min_y) as usize][(0 - min_x) as usize] = Some("s".to_string());
    }

    pos.into_iter()
        .rev()
        .map(|l| {
            l.into_iter()
                .map(|o| o.unwrap_or_else(|| ".".to_string()))
                .join(" ")
        })
        .join("\n")
}

fn move_head(knots: &mut [Knot], dir: (isize, isize)) {
    if knots.len() == 1 {
        return knots[0].move_head_d(dir);
    }

    knots[0].move_head_d(dir);
    let first_knot = knots[0];
    let second_knot = knots[1];

    // movement for second_knot.head
    let (dx, dy) = (
        first_knot.tail.x - second_knot.head.x,
        first_knot.tail.y - second_knot.head.y,
    );
    if dx == 0 && dy == 0 {
        // no need to move remainings knots
        return;
    }
    move_head(&mut knots[1..], (dx, dy));
}

fn count_tail_positions(path: &str, rope: &mut [Knot]) -> usize {
    let mut pos: HashSet<Point> = HashSet::new();
    let rlen = rope.len();
    for mv in path.lines().filter_map(|l| l.parse::<Move>().ok()) {
        for _i in 0..mv.len {
            move_head(rope, mv.dir.get_d());
            pos.insert(rope[rlen - 1].tail);
        }
        // println!("========================\n{}\n===================", display_knots(rope));
    }

    pos.len()
}
pub fn simulate_bridge() {
    let path = include_str!("../resources/day9_path.txt");
    let mut small_rope = [Knot::new()];
    let tail_pos_count = count_tail_positions(path, &mut small_rope);

    println!("number of unique pos of tail : {tail_pos_count}");

    let mut long_rope = [Knot::new(); 9];
    let tail_pos_count = count_tail_positions(path, &mut long_rope);
    println!("number of unique pos of tail : {tail_pos_count}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_examples_works() {
        let path = indoc! {"
            R 4
            U 4
            L 3
            D 1
            R 4
            D 1
            L 5
            R 2
        "};
        let mut knot = [Knot::new()];
        assert_eq!(13, count_tail_positions(path, &mut knot));
        let mut knots = [Knot::new(); 9];
        assert_eq!(1, count_tail_positions(path, &mut knots));

        let path = indoc! {"
            R 5
            U 8
            L 8
            D 3
            R 17
            D 10
            L 25
            U 20
        "};
        let mut knots = [Knot::new(); 9];
        assert_eq!(36, count_tail_positions(path, &mut knots));
    }
}
