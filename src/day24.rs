use eyre::eyre;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Blizzard {
    start: Point,
    dir: Direction,
}

struct Valley {
    width: usize,
    depth: usize,
    start: Point,
    exit: Point,
    blizz: Vec<Blizzard>,
}

impl Valley {
    fn get_blizz_pos(&self, round: usize) -> HashSet<Point> {
        self.blizz
            .iter()
            .map(|b| match b.dir {
                Direction::North => Point {
                    x: b.start.x,
                    y: 1 + (b.start.y + self.depth - (round % self.depth) - 1) % self.depth,
                },
                Direction::South => Point {
                    x: b.start.x,
                    y: 1 + (b.start.y + self.depth + (round % self.depth) - 1) % self.depth,
                },
                Direction::East => Point {
                    x: 1 + (b.start.x + self.width + (round % self.width) - 1) % self.width,
                    y: b.start.y,
                },
                Direction::West => Point {
                    x: 1 + (b.start.x + self.width - (round % self.width) - 1) % self.width,
                    y: b.start.y,
                },
            })
            .collect()
    }

    fn get_possible_moves(&self, pos: &Point, blizz_pos: &HashSet<Point>) -> Vec<Point> {
        if *pos == self.start {
            let point = Point {
                x: self.start.x,
                y: 1,
            };

            // note : staying on start is not necessary here since going back from start/staying here
            // are also possible next move
            return if !blizz_pos.contains(&point) {
                vec![point]
            } else {
                vec![self.start]
            };
        }
        let &Point { x, y } = pos;
        [(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
            .into_iter()
            .filter_map(|(x, y)| match (x, y) {
                (x, y) if (x, y) == (self.start.x, self.start.y) => Some(self.start),
                (x, y) if (x, y) == (self.exit.x, self.exit.y) => Some(self.exit),
                (0, _) => None,
                (x, _) if x > self.width => None,
                (_, 0) => None,
                (_, y) if y > self.depth => None,
                _ => Some(Point { x, y }),
            })
            .filter(|p| !blizz_pos.contains(p))
            .collect()
    }
}

impl FromStr for Valley {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum Item {
            Point(Point),
            Blizz(Blizzard),
        }
        let items: Vec<Item> = s
            .lines()
            .enumerate()
            .flat_map(|(y, l)| {
                l.chars().enumerate().filter_map(move |(x, c)| match c {
                    '.' => Some(Item::Point(Point { x, y })),
                    '^' => Some(Item::Blizz(Blizzard {
                        start: Point { x, y },
                        dir: Direction::North,
                    })),
                    '>' => Some(Item::Blizz(Blizzard {
                        start: Point { x, y },
                        dir: Direction::East,
                    })),
                    '<' => Some(Item::Blizz(Blizzard {
                        start: Point { x, y },
                        dir: Direction::West,
                    })),
                    'v' => Some(Item::Blizz(Blizzard {
                        start: Point { x, y },
                        dir: Direction::South,
                    })),
                    _ => None,
                })
            })
            .collect();
        let Item::Point(start)= items[0] else {return Err(eyre!("first item should be a point !"))};
        let Item::Point(exit)= items[items.len()-1] else {return Err(eyre!("last item should be a point !"))};

        let blizz = items
            .into_iter()
            .filter_map(|item| {
                if let Item::Blizz(blizz) = item {
                    Some(blizz)
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            start,
            exit,
            blizz,
            width: exit.x,
            depth: exit.y - 1,
        })
    }
}

#[allow(dead_code)]
fn score_pos(pos: &Point, target: &Point) -> usize {
    target.x.abs_diff(pos.x) + target.y.abs_diff(pos.y)
}

fn find_shortest_safe_path(
    valley: &Valley,
    start_round: usize,
    start: &Point,
    end: &Point,
) -> usize {
    // kind of taboo SINCE we explore all solution, this is useless
    // let period = valley.depth * valley.width;
    // let mut already_explored = HashSet::from([(*start, start_round)]);
    let mut current_pos = vec![*start];

    let mut round = start_round;
    loop {
        round += 1;
        let blizz_pos = valley.get_blizz_pos(round);
        let new_pos: HashSet<Point> = current_pos
            .iter()
            .flat_map(|p| valley.get_possible_moves(p, &blizz_pos))
            .collect();
        if new_pos.is_empty() {
            panic!("no more options : should raise  number of kept options");
        }
        if new_pos.contains(end) {
            break;
        }
        current_pos = new_pos
            .into_iter()
            // .filter(|p| !already_explored.contains(&(*p, round % period)))
            // .sorted_by(|p1,p2|score_pos(end, p2).cmp(&score_pos(end, p1))).take(1000000) // there can be no more than 4 * width * depth value !
            .collect();

        // let's both fill the taboo (is it really necessary ?
        // and re check values
        // for p in &current_pos {
        //     already_explored.insert((*p, round % period));
        //     if blizz_pos.contains(p) {
        //         panic!("round {round}: current pos {p:?}  is on a blizzard !");
        //     }
        //     if p.x == 0
        //         || p.x > valley.width
        //         || (p.y == 0 && p.x != valley.start.x)
        //         || (p.y > valley.depth && p.x != valley.exit.x)
        //     {
        //         panic!("current pos {p:?} is outside valley !");
        //     }
        // }
    }

    round
}

fn find_minimum_round_to_exit(valley: &Valley) -> usize {
    find_shortest_safe_path(valley, 0, &valley.start, &valley.exit)
}

fn find_minimum_round_to_exit_with_snack(valley: &Valley) -> usize {
    let len = find_shortest_safe_path(valley, 0, &valley.start, &valley.exit);
    let len = find_shortest_safe_path(valley, len, &valley.exit, &valley.start);
    find_shortest_safe_path(valley, len, &valley.start, &valley.exit)
}

pub fn escape_valley() {
    let valley: Valley = include_str!("../resources/day24_blizzard_valley.txt")
        .parse()
        .unwrap();

    let minimum_escape_rounds = find_minimum_round_to_exit(&valley);
    println!("goal can be reached in  : {minimum_escape_rounds} rounds");
    let full_path = find_minimum_round_to_exit_with_snack(&valley);
    println!("goal can be reached with snack  in  : {full_path} rounds");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            #.######
            #>>.<^<#
            #.<..<<#
            #>v.><>#
            #<^v^^>#
            ######.#
        "};

        let valley: Valley = input.parse().unwrap();
        assert_eq!(Point { x: 1, y: 0 }, valley.start);
        assert_eq!(Point { x: 6, y: 5 }, valley.exit);
        assert_eq!(6, valley.width);
        assert_eq!(4, valley.depth);

        let blizz_pos_round_0 = HashSet::from([
            Point { x: 1, y: 1 },
            Point { x: 2, y: 1 },
            Point { x: 4, y: 1 },
            Point { x: 5, y: 1 },
            Point { x: 6, y: 1 },
            Point { x: 2, y: 2 },
            Point { x: 5, y: 2 },
            Point { x: 6, y: 2 },
            Point { x: 1, y: 3 },
            Point { x: 2, y: 3 },
            Point { x: 4, y: 3 },
            Point { x: 5, y: 3 },
            Point { x: 6, y: 3 },
            Point { x: 1, y: 4 },
            Point { x: 2, y: 4 },
            Point { x: 3, y: 4 },
            Point { x: 4, y: 4 },
            Point { x: 5, y: 4 },
            Point { x: 6, y: 4 },
        ]);

        println!("{:?}", &valley.blizz);
        assert_eq!(blizz_pos_round_0.len(), valley.blizz.len());
        assert_eq!(valley.blizz.len(), valley.get_blizz_pos(0).len());

        assert_eq!(blizz_pos_round_0, valley.get_blizz_pos(0));

        assert_eq!(18, find_minimum_round_to_exit(&valley));
        assert_eq!(54, find_minimum_round_to_exit_with_snack(&valley));
    }
}
