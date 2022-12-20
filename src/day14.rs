use crate::day14::Material::{Air, Rock, Sand};
use eyre::{eyre, Context};
use std::cmp::{max, min};
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Material {
    Air,
    Rock,
    Sand, // Should be considered as Rock for 1st round
}
impl Display for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Air => ' ',
            Rock => '#',
            Sand => 'o',
        })
    }
}

struct Scene {
    materials: Vec<Vec<Material>>,

    max_x: usize,
    max_y: usize,

    last_fall: Vec<(usize, usize)>,
}

impl FromStr for Scene {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rocks = s.lines();

        // FIXME : error_handling

        // note : max_x is max for the build only
        let max_x: usize = rocks
            .clone()
            .flat_map(|l| l.split(" -> "))
            .filter_map(|coord| {
                coord[0..coord.find(',').expect("no ',' in coord")]
                    .parse()
                    .ok()
            })
            .max()
            .unwrap();
        let max_y: usize = rocks
            .clone()
            .flat_map(|l| l.split(" -> "))
            .filter_map(|coord| {
                let coord = &coord[(1 + coord.find(',').expect("no ',' in coord"))..];
                coord.parse::<usize>().ok()
            })
            .max()
            .unwrap();

        let mut materials = vec![vec![Air; max_x + 1]; max_y + 2];
        for rock in rocks {
            let coords: Result<Vec<(usize, usize)>, Self::Err> = rock
                .split(" -> ")
                .map(|coord_str| {
                    let mut parts = coord_str.split(',');
                    let x: usize = parts.next().map_or_else(
                        || Err(eyre!("no x in {coord_str}")),
                        |c| c.parse().context("reading x"),
                    )?;
                    let y: usize = parts.next().map_or_else(
                        || Err(eyre!("no x in {coord_str}")),
                        |c| c.parse().context("reading y"),
                    )?;
                    Ok((x, y))
                })
                .collect();

            let mut previous = None;
            for (x, y) in coords? {
                if let Some((prev_x, prev_y)) = previous {
                    if x == prev_x {
                        for m in materials
                            .iter_mut()
                            .take(max(prev_y, y) + 1)
                            .skip(min(prev_y, y))
                        {
                            m[x] = Rock;
                        }
                    } else if y == prev_y {
                        for m in materials[y]
                            .iter_mut()
                            .take(max(prev_x, x) + 1)
                            .skip(min(prev_x, x))
                        {
                            *m = Rock;
                        }
                    } else {
                        panic!("no strait line from ({prev_x},{prev_y}) to ({x},{y})");
                    }
                }
                previous = Some((x, y));
            }
        }
        Ok(Self {
            materials,
            max_x,
            max_y,
            last_fall: vec![],
        })
    }
}

impl Display for Scene {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let min_non_air = self
            .materials
            .iter()
            .map(|l| {
                l.iter()
                    .enumerate()
                    .find(|(_, m)| **m != Air)
                    .map(|(i, _)| i)
                    .unwrap_or(self.max_x)
            })
            .min()
            .unwrap_or(0);
        f.write_str(&format!(
            "Scene[{min_non_air}-{},{} / {}]",
            self.max_x,
            self.max_y,
            self.materials.len()
        ))?;
        let mut fall = self.last_fall.iter();
        for l in &self.materials {
            let f_iter = fall.next();
            for (i, m) in l.iter().enumerate() {
                if i >= min_non_air {
                    if let Some((x, _)) = f_iter {
                        if i == *x { f.write_char('~') } else { m.fmt(f) }?;
                    } else {
                        m.fmt(f)?;
                    }
                }
            }
            f.write_char('\n')?;
        }
        for _ in 0..=(self.max_x - min_non_air) {
            f.write_char('_')?;
        }
        f.write_char('\n')
    }
}

impl Scene {
    fn get_material(&self, point: (usize, usize)) -> Material {
        let (x, y) = point;

        // Floor
        if y == self.max_y + 2 {
            return Rock;
        }

        if x >= self.materials[y].len() {
            return Air;
        }
        self.materials[y][x]
    }

    fn set_material(&mut self, material: Material, point: (usize, usize)) {
        let (x, y) = point;

        if y > self.max_y + 1 {
            /*panic!("no point in going further down than {y}");*/
            return;
        }
        let line = &mut self.materials[y];
        let len = line.len();
        if x >= len {
            for _ in line.len()..=(1 + x) {
                line.push(Air);
            }
        }
        line[x] = material;
        if self.max_x < x {
            self.max_x = x;
        }
    }
}

fn pour_sand(scene: &mut Scene, pouring_point: (usize, usize)) -> Vec<(usize, usize)> {
    assert_eq!(
        scene.get_material(pouring_point),
        Air,
        "Cannot pour sand in non Air"
    );
    let (mut x, mut y) = pouring_point;
    let mut fall = vec![(x, y)];
    while y <= scene.max_y {
        if Air == scene.get_material((x, y + 1)) {
            y += 1;
        } else if Air == scene.get_material((x - 1, y + 1)) {
            y += 1;
            x -= 1;
        } else if Air == scene.get_material((x + 1, y + 1)) {
            y += 1;
            x += 1;
        } else {
            break;
        }
        fall.push((x, y));
    }
    scene.set_material(Sand, (x, y));

    scene.last_fall = fall.clone();
    fall
}

fn pour_max_sand_at_rest(input: &str) -> usize {
    let mut scene: Scene = input.parse().unwrap();

    let max_y = scene.max_y;
    let mut count = 0;
    while pour_sand(&mut scene, (500, 0)).len() <= max_y {
        count += 1;
    }
    pour_sand(&mut scene, (500, 0));
    // println!("{}", scene);
    count
}

fn pour_max_sand(input: &str) -> usize {
    let mut scene: Scene = input.parse().unwrap();

    let mut count = 0;
    while scene.get_material((500, 0)) == Air {
        pour_sand(&mut scene, (500, 0)).len();
        count += 1;
    }

    count
}

pub fn avoid_sand() {
    let input = include_str!("../resources/day14_rocks.txt");

    let sand_count = pour_max_sand_at_rest(input);
    println!("max poured sand before falldown : {sand_count}");

    let max_sand_count = pour_max_sand(input);
    println!("max poured sand : {max_sand_count}");
}
#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_ok() {
        let input = indoc! {"
            498,4 -> 498,6 -> 496,6
            503,4 -> 502,4 -> 502,9 -> 494,9
        "};
        let scene: Scene = input.parse().unwrap();
        assert_eq!(9, scene.max_y);
        assert_eq!(24, pour_max_sand_at_rest(input));
        assert_eq!(93, pour_max_sand(input));
    }
}
