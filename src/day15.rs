use eyre::{eyre, Context};
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

    let freq = find_beacon(input, 4000000, 4000000).unwrap();

    println!("beacon freq : {freq}");
}

pub fn find_beacon(input: &str, max_x: isize, max_y: isize) -> Option<isize> {
    let sensors_beacons = read_pos(input);
    let sensors_dist: Vec<(Point, isize)> = sensors_beacons
        .iter()
        .map(|(s, b)| (*s, s.distance(b)))
        .collect();
    for y in 0..=max_y {
        let mut x = 0;
        while x <= max_x {
            let margin = sensors_dist
                .iter()
                .map(|(s, d)| *d - Point { x, y }.distance(s))
                .max()
                .unwrap();

            if margin < 0 {
                return Some(x * 4000000 + y);
            }
            x += margin + 1;
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
        assert_eq!(56000011, find_beacon(input, 20, 20).unwrap());
    }
}
