use eyre::{Context, ContextCompat};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::iter::once;

#[derive(Debug, Clone)]
struct Valve<'v> {
    name: &'v str,
    flow: usize,
    nexts: Vec<&'v str>,
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

        Ok(Self { name, flow, nexts })
    }
}

#[derive(Debug, Clone)]
struct OpeningPath<'v> {
    opened_at: Vec<(&'v str, usize)>,
}

impl<'v> OpeningPath<'v> {
    fn score(&self, volcano: &Volcano<'v>) -> usize {
        let total_time = volcano.time_left;
        self.opened_at
            .iter()
            .map(|(name, dist)| {
                total_time.saturating_sub(*dist)
                    * volcano
                        .valves_by_name
                        .get(name)
                        .map(|v| v.flow)
                        .unwrap_or(0)
            })
            .sum()
    }

    // TODO : take an IntoIterator<Item=Self> ?
    fn dual_score(&self, other: &Self, volcano: &Volcano<'v>) -> usize {
        let mut opened_at: HashMap<&'v str, usize> = HashMap::new();
        for (name, time) in self.opened_at.iter().chain(other.opened_at.iter()) {
            if let Some(otime) = opened_at.get(name) {
                if otime > time {
                    opened_at.insert(name, *time);
                }
            } else {
                opened_at.insert(name, *time);
            }
        }
        // does only work because OpeningPath does not consider order
        Self {
            opened_at: opened_at.into_iter().collect(),
        }
        .score(volcano)
    }

    fn new() -> Self {
        Self { opened_at: vec![] }
    }
}

#[derive(Debug, Clone)]
struct Volcano<'v> {
    valves_by_name: HashMap<&'v str, Valve<'v>>,
    time_left: usize,
}
impl<'v> Volcano<'v> {
    fn from_str(s: &'v str) -> Result<Self, eyre::Error> {
        let valves = s.lines().filter_map(|l| match Valve::from_str(l) {
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

    fn distances(&self) -> HashMap<(&'v str, &'v str), usize> {
        let names: Vec<_> = self
            .valves_by_name
            .keys()
            .copied()
            .sorted_by(|n1, n2| n1.cmp(n2))
            .collect();
        let mut result: HashMap<(&'v str, &'v str), usize> = HashMap::new();
        for s in names
            .iter()
            .filter_map(|name| self.valves_by_name.get(name))
        {
            let mut explored: HashSet<&str> = HashSet::new();
            explored.insert(s.name);
            let mut nexts = s.nexts.clone();
            let mut dist = 1;
            while !nexts.is_empty() {
                let next_nexts: Vec<_> = nexts
                    .iter()
                    .filter(|n| !explored.contains(*n))
                    .filter_map(|n| self.valves_by_name.get(n))
                    .flat_map(|v| v.nexts.iter())
                    .copied()
                    .collect();
                for n in nexts {
                    explored.insert(n);
                    result.entry((s.name, n)).or_insert(dist);
                }
                nexts = next_nexts;
                dist += 1;
            }
        }
        result
    }

    fn max_score_optimized(&self) -> usize {
        const MAX_POP: usize = 100;
        let distances_by_edges = self.distances();

        let mut bests: Vec<_> = vec![OpeningPath::new()];
        let all_valves: Vec<_> = self.valves_by_name.keys().copied().collect();
        let mut max_score = 0usize;

        while !bests.is_empty() {
            let distances = &distances_by_edges;

            let new_bests: Vec<_> = bests
                .iter()
                .flat_map(|b| {
                    let already_opened: HashSet<_> = b.opened_at.iter().map(|(n, _)| *n).collect();
                    let (current, time) = b.opened_at.last().unwrap_or(&("AA", 0));
                    all_valves
                        .iter()
                        .filter(move |v| !already_opened.contains(*v))
                        .filter_map(move |v| {
                            if let Some(extra_dist) = distances.get(&(current, *v)) {
                                if time + extra_dist < self.time_left {
                                    let mut opened_at = b.opened_at.clone();
                                    opened_at.push((*v, time + extra_dist + 1));
                                    Some(OpeningPath { opened_at })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                })
                .sorted_by(|op1, op2| op1.score(self).cmp(&op2.score(self)))
                .rev()
                .take(MAX_POP)
                .collect();

            if !new_bests.is_empty() {
                if let Some(local_best) = new_bests.first() {
                    let local_best_score = local_best.score(self);

                    if local_best_score > max_score {
                        // println!(
                        //     "new best : {:?} => {local_best_score}",
                        //     new_bests.first().unwrap()
                        // );
                        max_score = local_best_score;
                    }
                }
            }
            bests = new_bests;
        }

        max_score
    }

    fn max_dual_score_optimized(&self) -> usize {
        const MAX_POP: usize = 300;
        let distances_by_edges = self.distances();

        let mut bests: Vec<_> = vec![(OpeningPath::new(), OpeningPath::new())];
        let all_valves: Vec<_> = self.valves_by_name.keys().copied().collect();
        let mut max_score = 0usize;

        let try_extend = |path: &OpeningPath<'v>, v_name: &'v str| {
            let (current, time) = path.opened_at.last().unwrap_or(&("AA", 0));
            distances_by_edges
                .get(&(*current, v_name))
                .and_then(|extra_dist| {
                    if *time + *extra_dist < self.time_left {
                        let mut opened_at = path.opened_at.clone();
                        opened_at.push((v_name, time + extra_dist + 1));
                        Some(OpeningPath { opened_at })
                    } else {
                        None
                    }
                })
        };

        while !bests.is_empty() {
            let new_bests: Vec<_> = bests
                .iter()
                .flat_map(|(b1, b2)| {
                    // FIXME : could store already_opened with each entry, to avoid building it each time
                    let already_opened: HashSet<_> = b1
                        .opened_at
                        .iter()
                        .chain(b2.opened_at.iter())
                        .map(|(name, _)| *name)
                        .collect();
                    all_valves
                        .iter()
                        .filter(move |v| !already_opened.contains(*v))
                        .flat_map(|v| {
                            let next1 = try_extend(b1, v);
                            let next2 = try_extend(b2, v);

                            once(next1.map(|n| (n, b2.clone())))
                                .chain(once(next2.map(|n| (b1.clone(), n))))
                                .flatten()
                        })
                })
                .sorted_by(|(op11, op12), (op21, op22)| {
                    op11.dual_score(op12, self)
                        .cmp(&op21.dual_score(op22, self))
                })
                .rev()
                .take(MAX_POP)
                .collect();

            if !new_bests.is_empty() {
                if let Some((local_best1, local_best2)) = new_bests.first() {
                    let local_best_score = local_best1.dual_score(local_best2, self);

                    if local_best_score > max_score {
                        // println!(
                        //     "new best : {:?} => {local_best_score}",
                        //     new_bests.first().unwrap()
                        // );
                        max_score = local_best_score;
                    }
                }
            }
            bests = new_bests;
        }

        max_score
    }
}

pub fn escape_volcano() {
    let input = include_str!("../resources/day16_volcano.txt");
    let mut volcano = Volcano::from_str(input).unwrap();
    let max_pressure = volcano.max_score_optimized();
    println!("max pressure {max_pressure}");

    volcano.time_left = 26;
    let max_pressure = /*2967; // FIXME*/ volcano.max_dual_score_optimized();
    println!("max dual pressure {max_pressure}");
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let opening_path = OpeningPath {
            opened_at: vec![
                ("DD", 2),
                ("BB", 5),
                ("JJ", 9),
                ("HH", 17),
                ("EE", 21),
                ("CC", 24),
            ],
        };
        assert_eq!(1651, opening_path.score(&volcano));
        assert_eq!(1651, volcano.max_score_optimized());

        volcano.time_left = 26;
        assert_eq!(1707, volcano.max_dual_score_optimized());
    }
}
