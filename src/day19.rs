use crate::day19::Material::{Clay, Geode, Obsidian, Ore};
use eyre::{eyre, ContextCompat, WrapErr};
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[repr(usize)]
enum Material {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}
const MATERIALS: [Material; 4] = [Ore, Clay, Obsidian, Geode];
const MATERIAL_COUNT: usize = MATERIALS.len();

impl FromStr for Material {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        match s {
            "ore" => Ok(Self::Ore),
            "clay" => Ok(Self::Clay),
            "obsidian" => Ok(Obsidian),
            "geode" => Ok(Self::Geode),
            _ => Err(eyre!("'{s}' is not a Material")),
        }
    }
}
impl Display for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Ore => "ore",
            Clay => "clay",
            Obsidian => "obsidian",
            Geode => "geode",
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct MatQuantities([usize; MATERIAL_COUNT]);

impl Display for MatQuantities {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        for m in MATERIALS {
            f.write_str(&format!("{:5} : {}, ", m, self.0[m as usize]))?;
        }
        f.write_char(']')
    }
}
impl Index<usize> for MatQuantities {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
impl IndexMut<usize> for MatQuantities {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[derive(Debug)]
struct Blueprint {
    id: usize,
    cost_by_robot: [MatQuantities; MATERIAL_COUNT],
}

fn parse_robot_cost(line: &str) -> Result<(Material, MatQuantities), eyre::Error> {
    let mut words = line[..line.len()].split_whitespace().skip(1);
    let product: Material = words.next().map_or_else(
        || Err(eyre!("no product from '{line}'")),
        |w| w.parse::<Material>(),
    )?;
    words.nth(1); // skip robot costs
    let mut costs = [0; MATERIAL_COUNT];

    while let Some(cost) = words.next() {
        let material = words
            .next()
            .with_context(|| eyre!("reading material associated to cost"))?;
        let _and = words.next();
        costs[material.parse::<Material>()? as usize] = cost.parse()?;
    }

    Ok((product, MatQuantities(costs)))
}
impl FromStr for Blueprint {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.split(&[':', '.'][..]);
        let id = lines.next().map_or_else(
            || Err(eyre!("No blueprint id line in '{s}'")),
            |line| {
                line["Blueprint ".len()..]
                    .parse()
                    .with_context(|| eyre!("reading BP id from {line}"))
            },
        )?;
        let costs_by_robot_unordered: Result<HashMap<Material, MatQuantities>, _> = lines
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(parse_robot_cost)
            .collect();
        let costs_by_robot_unordered = costs_by_robot_unordered?;
        let mut cost_by_robot = [MatQuantities::default(); MATERIAL_COUNT];
        for (m, q) in costs_by_robot_unordered.into_iter() {
            cost_by_robot[m as usize] = q;
        }
        Ok(Self { id, cost_by_robot })
    }
}

#[derive(Debug, Clone, Copy)]
struct Production<'f> {
    extracted: MatQuantities,
    robots: MatQuantities,
    rounds_left: usize,

    factory: &'f Blueprint,
}

impl<'f> Production<'f> {
    fn score(&self, relative_costs: &MatQuantities) -> usize {
        // available Geode *N
        // productible geode * left_rounds
        // - (obsidian/geode ratio * available)*(left_rounds - 1) + (obsidian/geode ratio * productible obs)*(left_rounds - 2)
        // ...
        (0..MATERIAL_COUNT)
            .into_iter()
            .map(|m_idx| {
                //  RELATIVE_COST             EXTRACTED              PRODUCTION_SPEED
                relative_costs[m_idx]
                    * relative_costs[m_idx]
                    * (self.extracted[m_idx]
                        + (self.robots[m_idx]
                            + usize::from(self.will_build_robot(MATERIALS[m_idx])))
                            * self.rounds_left)
            })
            .sum()
    }

    // has enough robot to build this robot in the future
    fn will_build_robot(&self, product: Material) -> bool {
        let cost = self.factory.cost_by_robot[product as usize];
        for (m, _) in cost.0.iter().enumerate() {
            if 0 == self.robots[m] {
                return false;
            }
        }
        true
    }
    fn can_build_robot(&self, product: Material) -> bool {
        let cost = self.factory.cost_by_robot[product as usize];
        for (m, c) in cost.0.iter().enumerate() {
            if *c > self.extracted[m] {
                return false;
            }
        }
        true
    }
    fn next_round(self) -> Vec<Self> {
        if self.rounds_left == 0 {
            return vec![];
        }

        let mut extracted: MatQuantities = self.extracted;
        for (extracted, robots) in extracted.0.iter_mut().zip(self.robots.0.iter()) {
            *extracted += *robots;
        }
        let mut noop = self;
        noop.rounds_left -= 1;
        noop.extracted = extracted;
        let mut result: Vec<Self> = Vec::with_capacity(MATERIAL_COUNT + 1);
        for p in std::iter::once(noop).chain(MATERIALS.into_iter().filter_map(|m| {
            if self.can_build_robot(m) {
                let mut next = self;
                next.extracted = extracted;
                next.rounds_left -= 1;
                next.robots.0[m as usize] += 1;
                for (extracted, cost) in next
                    .extracted
                    .0
                    .iter_mut()
                    .zip(self.factory.cost_by_robot[m as usize].0.iter())
                {
                    *extracted -= *cost;
                }
                Some(next)
            } else {
                None
            }
        })) {
            result.push(p);
        }

        result
    }
}

impl<'p> Display for Production<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "extracted : {}, robots: {}",
            self.extracted, self.robots
        ))
    }
}

impl Blueprint {
    fn costs(&self) -> MatQuantities {
        let mut costs = MatQuantities::default();
        costs[Ore as usize] = self.cost_by_robot[Ore as usize][Ore as usize];
        for m in [Clay, Obsidian, Geode] {
            let cost = self.cost_by_robot[m as usize]
                .0
                .iter()
                .enumerate()
                .map(|(i, c)| *c * costs[i])
                .sum();
            costs[m as usize] = cost
        }
        costs
    }

    fn get_score(&self, rounds: usize) -> usize {
        let costs = self.costs();
        let mut bests = vec![Production {
            extracted: Default::default(),
            robots: {
                let mut robots = MatQuantities::default();
                robots[Ore as usize] = 1;
                robots
            },
            rounds_left: rounds,
            factory: self,
        }];

        let max_pop = (costs[costs.0.len() - 1] / 70).pow(2);

        for i in 0..rounds {
            let max_pop = max_pop
                * (1 + min(
                    i.saturating_sub(i.saturating_sub(rounds / 2)),
                    (rounds / 2).saturating_sub(i.saturating_sub(rounds / 2)),
                ));
            bests = bests.par_iter().flat_map(|b| b.next_round()).collect();
            bests.par_sort_unstable_by(|b1, b2| b2.score(&costs).cmp(&b1.score(&costs)));
            bests = bests.into_iter().take(max_pop).collect()
        }

        bests.sort_by_key(|b| std::cmp::Reverse(b.score(&costs)));
        debug_assert_eq!(0, bests.get(0).unwrap().rounds_left, "all rounds should have been run");

        let best = bests.get(0).unwrap();

        best.extracted[Geode as usize]
    }
    fn get_quality(&self, rounds: usize) -> usize {
        self.id * self.get_score(rounds)
    }
}

impl Display for Blueprint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Blueprint[{} : {}]",
            self.id,
            self.cost_by_robot.iter().join("/")
        ))
    }
}

pub fn collect_geodes() {
    let blueprints: Vec<Blueprint> = include_str!("../resources/day19_blueprints.txt")
        .lines()
        .filter_map(|l| l.parse().ok())
        .collect();
    // println!(
    //     "{} blueprints : {}",
    //     blueprints.len(),
    //     blueprints.iter().map(|bp| bp.id).join(",")
    // );

    let quality_sum: usize = blueprints.par_iter().map(|b| b.get_quality(24)).sum();
    println!("quality_sum : {quality_sum}");

    let geodes_product: usize = blueprints[..3]
        .par_iter()
        .map(|bp| bp.get_score(32))
        .product();
    println!("geodes_product : {geodes_product}");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aoc_example_works() {
        let input="Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.";
        let bp: Blueprint = input.parse().unwrap();
        assert_eq!(9, bp.get_quality(24));
        let bp2 :Blueprint = "Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian."
            .parse().unwrap();

        assert_eq!(24, bp2.get_quality(24));

        assert_eq!(56, bp.get_score(32));
        assert_eq!(62, bp2.get_score(32));
    }
}
