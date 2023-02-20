use crate::day23::Direction::{East, North, South, West};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Pos {
    x: isize,
    y: isize,
}

struct Grove {
    elves: HashSet<Pos>,
    round: usize,
}
impl FromStr for Grove {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elves = s
            .lines()
            .enumerate()
            .flat_map(|(y, l)| {
                l.chars().enumerate().filter_map(move |(x, c)| {
                    if c == '#' {
                        Some(Pos {
                            x: x as isize,
                            y: y as isize,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();
        Ok(Self { elves, round: 0 })
    }
}
#[derive(Debug, Copy, Clone)]
enum Direction {
    North,
    South,
    West,
    East,
}

const DIRECTIONS: [Direction; 4] = [North, South, West, East];
impl Direction {
    fn get(offset: usize) -> Self {
        DIRECTIONS[offset % DIRECTIONS.len()]
    }
}

impl Grove {
    fn count_neighbors(&self, pos: &Pos, dir: Option<Direction>) -> usize {
        let (xrange, yrange) = match dir {
            Some(North) => ((-1..2), (-1..0)),
            Some(South) => ((-1..2), (1..2)),
            Some(East) => ((1..2), (-1..2)),
            Some(West) => ((-1..0), (-1..2)),
            _ => ((-1..2), (-1..2)),
        };
        xrange
            .flat_map(|dx| yrange.clone().map(move |dy| (dx, dy)))
            .filter_map(|(dx, dy)| {
                if dx == 0 && dy == 0 {
                    None
                } else {
                    self.elves.get(&Pos {
                        x: pos.x + dx,
                        y: pos.y + dy,
                    })
                }
            })
            .count()
    }

    // return an updated Grove and the number of moves
    fn next_round(self) -> (Self, usize) {
        let tentative_next_pos: Vec<Pos> = self
            .elves
            .iter()
            .map(|p| {
                if self.count_neighbors(p, None) == 0 {
                    *p
                } else {
                    (0..4)
                        .map(|offset| Direction::get(self.round + offset))
                        .find_map(|dir| {
                            if 0 == self.count_neighbors(p, Some(dir)) {
                                Some(match dir {
                                    North => Pos { x: p.x, y: p.y - 1 },
                                    South => Pos { x: p.x, y: p.y + 1 },
                                    West => Pos { x: p.x - 1, y: p.y },
                                    East => Pos { x: p.x + 1, y: p.y },
                                })
                            } else {
                                None
                            }
                        })
                        .unwrap_or(*p)
                }
            })
            .collect();

        let count_by_next_pos: HashMap<Pos, usize> =
            tentative_next_pos
                .iter()
                .fold(HashMap::new(), |mut acc, p| {
                    acc.entry(*p).and_modify(|count| *count += 1).or_insert(1);
                    acc
                });

        // since we check for place availablility before trying to move, no need to take move failure into account for other elves
        let elves = self
            .elves
            .iter()
            .zip(tentative_next_pos.into_iter())
            .map(|(previous, next)| {
                if *count_by_next_pos.get(&next).unwrap() > 1 {
                    *previous
                } else {
                    next
                }
            })
            .collect();
        (
            Self {
                elves,
                round: self.round + 1,
            },
            count_by_next_pos
                .keys()
                .filter(|p| !self.elves.contains(*p))
                .count(),
        )
    }

    fn count_empty_ground(&self) -> usize {
        let min_x = self.elves.iter().map(|p| p.x).min().unwrap();
        let max_x = self.elves.iter().map(|p| p.x).max().unwrap();
        let min_y = self.elves.iter().map(|p| p.y).min().unwrap();
        let max_y = self.elves.iter().map(|p| p.y).max().unwrap();

        (max_x - min_x + 1) as usize * (max_y - min_y + 1) as usize - self.elves.len()
    }
}
impl Display for Grove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let min_x = self.elves.iter().map(|p| p.x).min().unwrap();
        let max_x = self.elves.iter().map(|p| p.x).max().unwrap();
        let min_y = self.elves.iter().map(|p| p.y).min().unwrap();
        let max_y = self.elves.iter().map(|p| p.y).max().unwrap();
        let round = self.round;

        f.write_str(&format!("{round}: ({min_x},{min_y})\n"))?;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                f.write_char(if self.elves.contains(&Pos { x, y }) {
                    '#'
                } else {
                    '.'
                })?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

pub fn plant_grove() {
    let mut grove: Grove = include_str!("../resources/day23_grove.txt")
        .parse()
        .unwrap();
    for _ in 0..10 {
        let (new_grove, _) = grove.next_round();
        grove = new_grove;
    }
    let free_tiles_count = grove.count_empty_ground();
    println!("free ground after 10 rounds : {free_tiles_count}");

    let mut grove: Grove = include_str!("../resources/day23_grove.txt")
        .parse()
        .unwrap();
    loop {
        let (new_grove, count) = grove.next_round();
        grove = new_grove;
        if count == 0 {
            break;
        }
    }
    println!("no move after  {} rounds", grove.round);
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            ..............
            ..............
            .......#......
            .....###.#....
            ...#...#.#....
            ....#...##....
            ...#.###......
            ...##.#.##....
            ....#..#......
            ..............
            ..............
            ..............
         "};

        let mut grove: Grove = input.parse().unwrap();
        for _ in 0..10 {
            let (new_grove, _) = grove.next_round();
            grove = new_grove;
        }
        assert_eq!(110, grove.count_empty_ground())
    }
}
