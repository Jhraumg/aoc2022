use std::cmp::max;
use std::collections::HashSet;

use std::str::FromStr;

#[derive(Debug)]
struct Map {
    elevations: Vec<Vec<u8>>,
    start: (usize, usize),
    end: (usize, usize),
}

impl FromStr for Map {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut start = (0, 0);
        let mut end = (0, 0);

        let elevations = s
            .lines()
            .enumerate()
            .map(|(j, l)| {
                l.chars()
                    .enumerate()
                    .filter_map(|(i, c)| match c {
                        'a'..='z' => Some(c as u8 - b'a' + 1),
                        'S' => {
                            start = (i, j);
                            Some(1)
                        }
                        'E' => {
                            end = (i, j);
                            Some(26)
                        }
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        Ok(Self {
            elevations,
            start,
            end,
        })
    }
}

#[derive(Debug, Clone)]
struct Path<'m> {
    map: &'m Map,
    lengths: Vec<Vec<Option<usize>>>,
    done: HashSet<(usize, usize)>,
    current: (usize, usize),
}

impl<'m> Path<'m> {
    fn new(map: &'m Map, start: (usize, usize)) -> Self {
        let mut lengths: Vec<Vec<Option<usize>>> =
            vec![vec![None; map.elevations[0].len()]; map.elevations.len()];
        lengths[start.1][start.0] = Some(0);
        Self {
            map,
            lengths,
            done: HashSet::new(),
            current: start,
        }
    }

    fn get_elevation(&self, x: usize, y: usize) -> u8 {
        self.map.elevations[y][x]
    }

    fn next_moves(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = pos;
        let current = self.get_elevation(x, y);

        let up = if y == 0 { None } else { Some((x, y - 1)) };
        let down = if y == self.map.elevations.len() - 1 {
            None
        } else {
            Some((x, y + 1))
        };
        let left = if x == 0 { None } else { Some((x - 1, y)) };
        let right = if x == self.map.elevations[0].len() - 1 {
            None
        } else {
            Some((x + 1, y))
        };
        [up, down, left, right]
            .iter()
            .filter_map(|e| {
                e.filter(|(x, y)| !self.done.contains(&(*x, *y)))
                    .filter(|(x, y)| self.get_elevation(*x, *y) <= current + 1)
            })
            .collect()
    }

    fn find_shortest_path_len(mut self) -> Option<usize> {
        // djikirsta
        loop {
            let current_len = self.lengths[self.current.1][self.current.0].unwrap();
            for (next_x, next_y) in self.next_moves(self.current) {
                if (next_x, next_y) == self.map.end {
                    return Some(current_len + 1);
                }

                if self.lengths[next_y][next_x]
                    .map(|l| l > current_len + 1)
                    .unwrap_or(true)
                {
                    self.lengths[next_y][next_x] = Some(current_len + 1);
                }
            }
            self.done.insert(self.current);
            if let Some(new_current) = self
                .done
                .iter()
                .filter(|(x, y)| self.lengths[*y][*x].unwrap() == max(1, current_len) - 1)
                .flat_map(|(x, y)| self.next_moves((*x, *y)).into_iter())
                .next()
            {
                self.current = new_current;
            } else if let Some(new_current) = self
                .done
                .iter()
                .filter(|(x, y)| self.lengths[*y][*x].unwrap() == max(1, current_len))
                .flat_map(|(x, y)| self.next_moves((*x, *y)).into_iter())
                .next()
            {
                self.current = new_current;
            } else {
                break;
            }
        }
        None
    }
}

fn find_shortest_slope(map: &Map) -> usize {
    let starts: Vec<_> = map
        .elevations
        .iter()
        .enumerate()
        .flat_map(|(y, l)| {
            l.iter()
                .enumerate()
                .filter(|(_, e)| **e == 1)
                .map(move |(x, _)| (x, y))
        })
        .collect();
    starts
        .iter()
        .filter_map(|s| {
            let path = Path::new(map, *s);
            path.find_shortest_path_len()
        })
        .min()
        .unwrap()
}
pub fn climb_hills() {
    let elevations = include_str!("../resources/day12_elevations.txt");
    let map: Map = elevations.parse().expect("could not parse map");
    {
        let path = Path::new(&map, map.start);
        let shortest = path.find_shortest_path_len().unwrap();
        println!("shortest path {shortest}");

        let shortest_slope = find_shortest_slope(&map);
        println!("shortest slope {shortest_slope}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_examples_work() {
        let elevations = indoc! {"
            Sabqponm
            abcryxxl
            accszExk
            acctuvwj
            abdefghi
        "};

        let map: Map = elevations.parse().expect("could not parse map");
        let path = Path::new(&map, map.start);
        assert_eq!(31, path.find_shortest_path_len().unwrap());
        assert_eq!(29, find_shortest_slope(&map));
    }
}
