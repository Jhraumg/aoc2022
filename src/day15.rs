use eyre::{eyre, Context};
use std::cmp::max;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Point {
    x: isize,
    y: isize,
}

impl Point {
    fn distance(&self, other: &Point) -> isize {
        (other.x - self.x).abs() + (other.y - self.y).abs()
    }
}

impl FromStr for Point {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // "x=231045, y=2977983"
        let mut parts = s.split(", ").map(|p| &p[2..]);
        let x: isize = parts
            .next()
            .map_or_else(|| Err(eyre!("no x")), |p| p.parse().context("reading x"))?;
        let y: isize = parts
            .next()
            .map_or_else(|| Err(eyre!("no y")), |p| p.parse().context("reading y"))?;
        Ok(Self { x, y })
    }
}

fn read_pos(input: &str) -> Vec<(Point, Point)> {
    input
        .lines()
        .map(|l| {
            let sensor: Point = l["Sensor at ".len()..l.find(':').expect("':' is mandatory")]
                .parse()
                .expect("sensor parse error");
            let beacon: Point = l[("closest beacon is at ".len()
                + l.find("closest beacon is at ").expect("no beacon part"))..]
                .parse()
                .expect("beacon parse error");
            (sensor, beacon)
        })
        .collect()
}

fn count_impossible_sport(input: &str, y: isize) -> usize {
    let sensors_beacons = read_pos(input);

    let beacons: HashSet<Point> = sensors_beacons.iter().map(|(_, b)| b).copied().collect();

    let max_dist = sensors_beacons
        .iter()
        .map(|(s, b)| s.distance(b))
        .max()
        .expect("no max distance");
    println!("max dist {max_dist}");
    let min_x = sensors_beacons
        .iter()
        .map(|(s, _)| s.x)
        .min()
        .expect("no max distance");
    let max_x = sensors_beacons
        .iter()
        .map(|(s, _)| s.x)
        .max()
        .expect("no max distance");

    (min_x - max_dist..=max_x + max_dist)
        .filter(|x| {
            let spot = Point { x: *x, y };
            !beacons.contains(&spot)
                && sensors_beacons
                    .iter()
                    .any(|(p, b)| p.distance(b) >= p.distance(&spot))
        })
        .count()
}

pub fn find_beacons() {
    let input = include_str!("../resources/day15_sensors_beacons.txt");
    let count = count_impossible_sport(input, 2000000);
    println!("number of impossible spots at 2000000 : {count}");

    for f in 0..4000 {
        // search is faster by vertical bands
        if let Some(freq) = find_beacon(input, 1000 * f, 1000 * (f + 1), 4000000) {
            println!("beacon freq : {freq}");
            break;
        }
    }
}

pub fn find_beacon(input: &str, min_x: isize, max_x: isize, max_y: isize) -> Option<isize> {
    let sensors_beacons = read_pos(input);
    let sensors_dist: Vec<(Point, isize)> = sensors_beacons
        .iter()
        .map(|(s, b)| (*s, s.distance(b)))
        .collect();
    let mut y = 0;
    while y <= max_y {
        let mut step = (min_x..=max_x)
            .map(|x| {
                let margin = sensors_dist
                    .iter()
                    .map(move |(s, d)| {
                        let new_d = Point { x, y }.distance(s);
                        if *d == new_d && s.y > y {
                            2 * (s.y - y)
                        } else {
                            *d - new_d
                        }
                    })
                    .max()
                    .unwrap();
                margin
            })
            .min()
            .unwrap();
        if step < 0 {
            break;
        }
        if step == 0 {
            // should not happen ?
            step = 1;
        }
        y += max(step, 1);
    }
    for x in min_x..=max_x {
        let p = Point { x, y };
        if sensors_dist.iter().all(|(s, d)| s.distance(&p) > *d) {
            return Some(x * 4000000 + y);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_examples_ok() {
        let input = indoc! {"
            Sensor at x=2, y=18: closest beacon is at x=-2, y=15
            Sensor at x=9, y=16: closest beacon is at x=10, y=16
            Sensor at x=13, y=2: closest beacon is at x=15, y=3
            Sensor at x=12, y=14: closest beacon is at x=10, y=16
            Sensor at x=10, y=20: closest beacon is at x=10, y=16
            Sensor at x=14, y=17: closest beacon is at x=10, y=16
            Sensor at x=8, y=7: closest beacon is at x=2, y=10
            Sensor at x=2, y=0: closest beacon is at x=2, y=10
            Sensor at x=0, y=11: closest beacon is at x=2, y=10
            Sensor at x=20, y=14: closest beacon is at x=25, y=17
            Sensor at x=17, y=20: closest beacon is at x=21, y=22
            Sensor at x=16, y=7: closest beacon is at x=15, y=3
            Sensor at x=14, y=3: closest beacon is at x=15, y=3
            Sensor at x=20, y=1: closest beacon is at x=15, y=3
        "};
        assert_eq!(26, count_impossible_sport(input, 10));
        assert_eq!(56000011, find_beacon(input, 20, 20));
    }
}
