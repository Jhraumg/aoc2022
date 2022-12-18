use crate::day10::Command::Noop;
use eyre::{eyre, Context, ContextCompat};
use itertools::Itertools;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum Command {
    Noop,
    Add(isize),
}

impl Command {
    fn len(&self) -> usize {
        match self {
            Noop => 1,
            Command::Add(_) => 2,
        }
    }
}
impl FromStr for Command {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        let mut items = s.split(' ');
        let cmd = items.next().context("trying to read cmd")?;
        match cmd {
            "noop" => Ok(Noop),
            "addx" => items
                .next()
                .context("trying to read value")
                .and_then(|v| v.parse::<isize>().context("parsing {v}"))
                .map(Command::Add),
            _ => Err(eyre!("cannot parse '{s}'")),
        }
    }
}
struct Computer {
    x: isize,
    time: usize,
    program: Vec<Command>,
    sp: usize,
}

impl Computer {
    fn new(source: &str) -> Self {
        let program = source
            .lines()
            .filter_map(|inst| inst.parse().ok())
            .collect();
        Self {
            x: 1,
            time: 0,
            program,
            sp: 0,
        }
    }
    fn execute_next(&mut self) -> usize {
        let cmd = self.program[self.sp];
        match cmd {
            Noop => {}
            Command::Add(v) => {
                self.x += v;
            }
        }
        self.time += cmd.len();
        self.sp += 1;

        cmd.len()
    }
    fn power_at_time(&mut self, time: usize) -> isize {
        let mut margin = time - self.time;
        while margin > self.program[self.sp].len() {
            margin -= self.execute_next();
        }
        // println!("last update time is {}, last value is {}", self.time, self.x);
        self.x * time as isize
    }

    fn display_screen(mut self) -> String {
        (0..6)
            .into_iter()
            .map(|r| {
                (0..40)
                    .into_iter()
                    .map(|c| {
                        let pixel_time = r * 40 + c + 1;
                        self.power_at_time(pixel_time);
                        if self.x - 1 <= c as isize && self.x + 1 >= c as isize {
                            "#"
                        } else {
                            " "
                        }
                    })
                    .join("")
            })
            .join("\n")
    }
}

pub fn decode_signal() {
    let source = include_str!("../resources/day10_source.txt");
    let mut computer = Computer::new(source);
    let strength_sum: isize = [20, 60, 100, 140, 180, 220]
        .into_iter()
        .map(|v| computer.power_at_time(v))
        .sum();
    println!("strength_sum : {strength_sum}");
    let computer = Computer::new(source);
    println!("{}", computer.display_screen());
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let source = indoc! {"
            addx 15
            addx -11
            addx 6
            addx -3
            addx 5
            addx -1
            addx -8
            addx 13
            addx 4
            noop
            addx -1
            addx 5
            addx -1
            addx 5
            addx -1
            addx 5
            addx -1
            addx 5
            addx -1
            addx -35
            addx 1
            addx 24
            addx -19
            addx 1
            addx 16
            addx -11
            noop
            noop
            addx 21
            addx -15
            noop
            noop
            addx -3
            addx 9
            addx 1
            addx -3
            addx 8
            addx 1
            addx 5
            noop
            noop
            noop
            noop
            noop
            addx -36
            noop
            addx 1
            addx 7
            noop
            noop
            noop
            addx 2
            addx 6
            noop
            noop
            noop
            noop
            noop
            addx 1
            noop
            noop
            addx 7
            addx 1
            noop
            addx -13
            addx 13
            addx 7
            noop
            addx 1
            addx -33
            noop
            noop
            noop
            addx 2
            noop
            noop
            noop
            addx 8
            noop
            addx -1
            addx 2
            addx 1
            noop
            addx 17
            addx -9
            addx 1
            addx 1
            addx -3
            addx 11
            noop
            noop
            addx 1
            noop
            addx 1
            noop
            noop
            addx -13
            addx -19
            addx 1
            addx 3
            addx 26
            addx -30
            addx 12
            addx -1
            addx 3
            addx 1
            noop
            noop
            noop
            addx -9
            addx 18
            addx 1
            addx 2
            noop
            noop
            addx 9
            noop
            noop
            noop
            addx -1
            addx 2
            addx -37
            addx 1
            addx 3
            noop
            addx 15
            addx -21
            addx 22
            addx -6
            addx 1
            noop
            addx 2
            addx 1
            noop
            addx -10
            noop
            noop
            addx 20
            addx 1
            addx 2
            addx 2
            addx -6
            addx -11
            noop
            noop
            noop
        "};
        let mut computer = Computer::new(source);
        assert_eq!(420, computer.power_at_time(20));
        assert_eq!(1140, computer.power_at_time(60));
        assert_eq!(1800, computer.power_at_time(100));
        assert_eq!(2940, computer.power_at_time(140));
        assert_eq!(2880, computer.power_at_time(180));
        assert_eq!(3960, computer.power_at_time(220));

        let mut computer = Computer::new(source);
        assert_eq!(
            13140isize,
            [20, 60, 100, 140, 180, 220]
                .into_iter()
                .map(|v| computer.power_at_time(v))
                .sum()
        );

        let computer = Computer::new(source);
        println!("{}", computer.display_screen());
    }
}
