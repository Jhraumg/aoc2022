use crate::day16::Action::{Move, Open};
use eyre::{Context, ContextCompat};
use itertools::Itertools;
use std::cmp::max;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct Valve<'v> {
    name: &'v str,
    flow: usize,
    nexts: Vec<&'v str>
}

impl<'v> Valve<'v> {
    fn from_str(s: &'v str) -> Result<Self, eyre::Error> {
        // Valve EF
        let name = &s[6..8];
        let flow: usize = s[s.find('=').unwrap() + 1..s.find(';').unwrap()]
            .parse()
            .context("reading flow")?;
        let nexts: Vec<_> = s[s
            .find("to valve")
            .with_context(|| format!("reading '{s}'"))?
            + 9..]
            .split(',')
            .map(str::trim)
            .collect();

        Ok(Self {
            name,
            flow,
            nexts,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Action<'v> {
    Open(&'v str),
    Move(&'v str),
}

#[derive(Debug, Clone)]
struct Path<'v> {
    actions: Vec<Action<'v>>,
}
impl<'v> Path<'v> {
    fn score(&self, volcano: &Volcano) -> usize {
        let time = volcano.time_left;
        self.actions
            .iter()
            .take(time)
            .enumerate()
            .map(|(i, j)| match j {
                Action::Open(n) => {
                    (volcano.time_left - i - 1)
                        * volcano.valves_by_name.get(n).map(|v| v.flow).unwrap_or(0)
                }
                Action::Move(_) => 0,
            })
            .sum()
    }
    fn nexts(&self, volcano: &'v Volcano) -> Vec<Self> {
        if self.actions.len() > volcano.time_left {
            return vec![];
        }
        let opened: HashSet<_> = self
            .actions
            .iter()
            .filter_map(|a| match a {
                Open(v) => Some(*v),
                Move(_) => None,
            })
            .collect();
        if let Some(last_action) = self.actions.last() {
            let last_node = match last_action {
                Open(last_node) => last_node,
                Move(last_node) => last_node,
            };
            if let Some(last_node) = volcano.valves_by_name.get(last_node) {
                let mut result: Vec<_> = last_node
                    .nexts
                    .iter()
                    .map(|n| {
                        let mut actions = self.actions.clone();
                        actions.push(Move(n));
                        Self { actions }
                    })
                    .collect();
                if !opened.contains(last_node.name) {
                    let mut actions = self.actions.clone();
                    actions.push(Open(last_node.name));
                    result.push(Self { actions });
                }
                result
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
    fn get_open(&self) -> HashSet<&'v str> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                Open(n) => Some(*n),
                Move(_) => None,
            })
            .collect()
    }

    fn score_by_valve(&self, volcano: &Volcano) -> HashMap<&'v str, usize> {
        let time = volcano.time_left;

        self.actions
            .iter()
            .take(time)
            .enumerate()
            .filter_map(|(i, j)| match j {
                Open(n) => Some((
                    *n,
                    (volcano.time_left - i - 1)
                        * volcano.valves_by_name.get(n).map(|v| v.flow).unwrap_or(0),
                )),
                Move(_) => None,
            })
            .collect()
    }

    fn dual_score(&self, other: &Self, volcano: &Volcano) -> usize {
        let score_by_valve = self.score_by_valve(volcano);
        let other_score_by_valve = other.score_by_valve(volcano);

        volcano
            .valves_by_name
            .keys()
            .map(|name| {
                max(
                    score_by_valve.get(name).unwrap_or(&0),
                    other_score_by_valve.get(name).unwrap_or(&0),
                )
            })
            .sum()
    }
}

#[derive(Debug, Clone)]
struct Volcano<'v> {
    valves_by_name: HashMap<&'v str, Valve<'v>>,
    time_left: usize,
}
impl<'v> Volcano<'v> {

    fn from_str(s: &'v str) -> Result<Self, eyre::Error> {
        let valves = s
            .lines()
            .filter_map(|l| match Valve::from_str(l) {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("error {e}");
                    None
                }
            });

        Ok(Self {
            valves_by_name: valves.map(|v| (v.name, v)).collect(),
            time_left: 30,
        })
    }

    fn max_score(&self) -> usize {
        const MAX_POP: usize = 1000;
        let mut bests = vec![Path {
            actions: vec![Move("AA")],
        }]; //fixme : vec is one siez too long
        for _ in 0..self.time_left {
            let next_bests = bests
                .iter()
                .flat_map(|b| b.nexts(self).into_iter())
                .sorted_by(|p1, p2| p1.score(self).cmp(&p2.score(self)))
                .rev()
                .take(MAX_POP)
                .collect();
            bests = next_bests;
        }
        if let Some(best) = bests
            .iter()
            .max_by(|p1, p2| p1.score(self).cmp(&p2.score(self)))
        {
            // println!("best {best:?}");
            Path {
                actions: best.actions[1..].to_owned(),
            }
            .score(self)
        } else {
            0
        }
    }

    fn max_dual_score(&self) -> usize {
        // FIXME :
        // * implement a proper heuristic algo, à la  taboo
        // * use [Action;30] instead of Vec<Action>
        //   in fact, use [[Action;30]; MAX_POP]
        //

        const MAX_POP: usize = 30000;
        let mut bests = vec![(
            Path {
                actions: vec![Move("AA")],
            },
            Path {
                actions: vec![Move("AA")],
            },
        )]; //fixme : vec is one size too long
        for _ in 0..self.time_left {
            let next_bests = bests
                .iter()
                .flat_map(|(b1, b2)| {
                    let n2 = b2.nexts(self);
                    b1.nexts(self)
                        .into_iter()
                        .flat_map(move |n1| n2.clone().into_iter().map(move |n2| (n1.clone(), n2)))
                })
                .filter(|(p1, p2)| {
                    let open1 = p1.get_open();
                    let open2 = p2.get_open();
                    open1.is_disjoint(&open2)
                })
                .sorted_by(|(p1, p2), (o1, o2)| {
                    p1.dual_score(p2, self).cmp(&o1.dual_score(o2, self))
                })
                .rev()
                .take(MAX_POP)
                .collect();
            bests = next_bests;
        }
        if let Some((b1, b2)) = bests
            .iter()
            .max_by(|(p1, p2), (o1, o2)| p1.dual_score(p2, self).cmp(&o1.dual_score(o2, self)))
        {
            let b1 = Path {
                actions: b1.actions[1..].to_owned(),
            };
            let b2 = Path {
                actions: b2.actions[1..].to_owned(),
            };

            // println!("dual best {b1:?}\n{b2:?}");
            b1.dual_score(&b2, self)
        } else {
            0
        }
    }
}

pub fn escape_volcano() {
    let input = include_str!("../resources/day16_volcano.txt");
    let mut volcano = Volcano::from_str(input).unwrap();
    let max_pressure = volcano.max_score();
    println!("max pressure {max_pressure}");

    volcano.time_left = 26;
    let max_pressure = /*2967; // FIXME*/ volcano.max_dual_score();
    println!("max dual pressure {max_pressure}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day16::Action::{Move, Open};
    use indoc::indoc;

    #[test]
    fn aoc_example_work() {
        let input = indoc! {"
            Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
            Valve BB has flow rate=13; tunnels lead to valves CC, AA
            Valve CC has flow rate=2; tunnels lead to valves DD, BB
            Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
            Valve EE has flow rate=3; tunnels lead to valves FF, DD
            Valve FF has flow rate=0; tunnels lead to valves EE, GG
            Valve GG has flow rate=0; tunnels lead to valves FF, HH
            Valve HH has flow rate=22; tunnel leads to valve GG
            Valve II has flow rate=0; tunnels lead to valves AA, JJ
            Valve JJ has flow rate=21; tunnel leads to valve II
        "};

        let mut volcano = Volcano::from_str(input).unwrap();
        let path = Path {
            actions: vec![
                Move("DD"),
                Open("DD"),
                Move("CC"),
                Move("BB"),
                Open("BB"),
                Move("AA"),
                Move("II"),
                Move("JJ"),
                Open("JJ"),
                Move("II"),
                Move("AA"),
                Move("DD"),
                Move("EE"),
                Move("FF"),
                Move("GG"),
                Move("HH"),
                Open("HH"),
                Move("GG"),
                Move("FF"),
                Move("EE"),
                Open("EE"),
                Move("DD"),
                Move("CC"),
                Open("CC"),
            ],
        };
        assert_eq!(1651, 20 * 28 + 13 * 25 + 21 * 21 + 22 * 13 + 3 * 9 + 2 * 6);

        assert_eq!(1651, path.score(&volcano));
        println!("best path {path:?}");

        assert_eq!(1651, volcano.max_score());
        volcano.time_left = 26;
        assert_eq!(1707, volcano.max_dual_score());
    }
}
