use eyre::eyre;
use itertools::Itertools;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Point3d {
    x: isize,
    y: isize,
    z: isize,
}
impl FromStr for Point3d {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((x, y, z)) = s.split(',').filter_map(|v| v.parse().ok()).collect_tuple() {
            return Ok(Self { x, y, z });
        }
        Err(eyre!("parse error on {s}"))
    }
}
impl Display for Point3d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("({}, {}, {})", self.x, self.y, self.z))
    }
}
fn count_faces(points: &[Point3d]) -> HashMap<Point3d, usize> {
    // 1,1,1  (1,2,2)(3,2,2)(2,1,2)
    // 2,1,1  (3,2,2),(5,2,2)(2,1,2)(2,3,2)
    // 1,2,1 (1,4,2)(3,4,2)(2,3,2
    // each cube is translated to 6 faces : (2x - 1,2y,2z), (2x-1,2y,2z)...
    let count_by_faces: HashMap<Point3d, usize> = points
        .iter()
        .flat_map(|&Point3d { x, y, z }| {
            [
                Point3d {
                    x: 2 * x - 1,
                    y: 2 * y,
                    z: 2 * z,
                },
                Point3d {
                    x: 2 * x + 1,
                    y: 2 * y,
                    z: 2 * z,
                },
                Point3d {
                    x: 2 * x,
                    y: 2 * y - 1,
                    z: 2 * z,
                },
                Point3d {
                    x: 2 * x,
                    y: 2 * y + 1,
                    z: 2 * z,
                },
                Point3d {
                    x: 2 * x,
                    y: 2 * y,
                    z: 2 * z - 1,
                },
                Point3d {
                    x: 2 * x,
                    y: 2 * y,
                    z: 2 * z + 1,
                },
            ]
            .into_iter()
        })
        .fold(HashMap::new(), |mut acc, p| {
            acc.entry(p).and_modify(|count| *count += 1).or_insert(1);
            acc
        });
    count_by_faces
}

fn count_free_faces(points: &[Point3d]) -> usize {
    let count_by_faces = count_faces(points);

    count_by_faces
        .iter()
        .filter(|(_, count)| **count == 1)
        .count()
}

fn count_reachable_free_faces(points: &[Point3d]) -> usize {
    let material_points: HashSet<&Point3d> = points.iter().collect();

    let (min_coord, max_coord) = points
        .iter()
        .map(|&Point3d { x, y, z }| (min(min(x, y), z), max(max(x, y), z)))
        .reduce(|(min_c, max_c), (local_min, local_max)| {
            (min(min_c, local_min), max(max_c, local_max))
        })
        .unwrap();

    // let's flood a cube containing all points
    let cube_edge = (max_coord - min_coord + 2) as usize;
    let mut water: HashSet<Point3d> = HashSet::with_capacity(cube_edge * cube_edge * cube_edge);
    for a in min_coord - 1..=max_coord + 1 {
        for b in min_coord - 1..=max_coord + 1 {
            water.insert(Point3d {
                x: min_coord - 1,
                y: a,
                z: b,
            });
            water.insert(Point3d {
                x: max_coord + 1,
                y: a,
                z: b,
            });
            water.insert(Point3d {
                y: min_coord - 1,
                x: a,
                z: b,
            });
            water.insert(Point3d {
                y: max_coord + 1,
                x: a,
                z: b,
            });
            water.insert(Point3d {
                z: min_coord - 1,
                y: a,
                x: b,
            });
            water.insert(Point3d {
                z: max_coord + 1,
                y: a,
                x: b,
            });
        }
    }
    let mut previous_round_water: Vec<Point3d> = water.iter().copied().collect();
    while !previous_round_water.is_empty() {
        let mut new_round_water: Vec<Point3d> = Vec::with_capacity(previous_round_water.len());
        for p in previous_round_water {
            let Point3d { x, y, z } = p;
            for p in [
                Point3d { x: x - 1, y, z },
                Point3d { x: x + 1, y, z },
                Point3d { x, y: y - 1, z },
                Point3d { x, y: y + 1, z },
                Point3d { x, y, z: z - 1 },
                Point3d { x, y, z: z + 1 },
            ] {
                if !(p.x < min_coord
                    || p.x > max_coord
                    || p.y < min_coord
                    || p.y > max_coord
                    || p.z < min_coord
                    || p.z > max_coord
                    || water.contains(&p)
                    || material_points.contains(&p))
                {
                    new_round_water.push(p);
                }
            }
        }
        for p in &new_round_water {
            water.insert(*p);
        }
        previous_round_water = new_round_water;
    }

    let faces: HashSet<Point3d> = count_faces(points)
        .into_iter()
        .filter_map(|(p, count)| if count == 1 { Some(p) } else { None })
        .collect();

    let water: Vec<_> = water.into_iter().collect();
    let water_faces: HashSet<Point3d> = count_faces(&water)
        .into_iter()
        .filter_map(|(p, count)| if count == 1 { Some(p) } else { None })
        .collect();

    faces
        .iter()
        .filter(|face| water_faces.contains(face))
        .count()
}

pub fn observe_boulders() {
    let droplets: Vec<Point3d> = include_str!("../resources/day18_droplets.txt")
        .lines()
        .filter_map(|l| l.parse().ok())
        .collect();
    let free_faces_count = count_free_faces(&droplets);
    println!("free faces : {free_faces_count}");
    let reachable_free_faces_count = count_reachable_free_faces(&droplets);
    println!("reachable free faces : {reachable_free_faces_count}");
}
#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            2,2,2
            1,2,2
            3,2,2
            2,1,2
            2,3,2
            2,2,1
            2,2,3
            2,2,4
            2,2,6
            1,2,5
            3,2,5
            2,1,5
            2,3,5
        "};
        let points: Vec<_> = input.lines().filter_map(|l| l.parse().ok()).collect();
        assert_eq!(64, count_free_faces(&points));
        assert_eq!(58, count_reachable_free_faces(&points));
    }
}
