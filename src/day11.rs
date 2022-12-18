use eyre::{eyre, Context, ContextCompat};
use itertools::Itertools;
use std::collections::VecDeque;
use std::str::FromStr;

#[derive(Debug)]
enum Operation {
    Plus(usize),
    Mult(usize),
    Square,
}
impl Operation {
    fn apply(&self, item: usize) -> usize {
        match self {
            Operation::Plus(i) => item + *i,
            Operation::Mult(i) => item * *i,
            Operation::Square => item * item,
        }
    }
}

const OPERATION_PREFIX: &str = "  Operation: new = old ";
impl FromStr for Operation {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let remain = s
            .strip_prefix(OPERATION_PREFIX)
            .with_context(|| format!("'{s}' does not start with '{OPERATION_PREFIX}'"))?;
        let char = remain.chars().next().context("reading operator")?;
        match char {
            '+' => remain[1..]
                .trim()
                .parse()
                .map(Self::Plus)
                .with_context(|| format!("reading operand fom '{}'", &remain[1..])),
            '*' => {
                if &remain[1..] == " old" {
                    Ok(Self::Square)
                } else {
                    remain[1..]
                        .trim()
                        .parse()
                        .map(Self::Mult)
                        .with_context(|| format!("reading operand fom '{}'", &remain[1..]))
                }
            }
            _ => Err(eyre!("wrong operator !")),
        }
    }
}

#[derive(Debug)]
struct Monkey {
    items: VecDeque<usize>,
    oper: Operation,
    divisor: usize,
    if_true: usize,
    if_false: usize,
    inspects: usize,
}
const DIVISOR_PREFIX: &str = "  Test: divisible by ";
impl FromStr for Monkey {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let header = lines.next().context("reading Monkey line")?;
        if !header.starts_with("Monkey ") {
            return Err(eyre!("bad header '{s}'"));
        }
        let items_line = lines.next().context("reading items").and_then(|d| {
            d.trim()
                .strip_prefix("Starting items:")
                .context("reading items")
        })?;
        let items: Result<VecDeque<_>, _> = items_line
            .split(',')
            .map(|v| v.trim().parse::<usize>())
            .collect();
        let oper: Operation = lines
            .next()
            .map_or_else(|| Err(eyre!("no operation")), |d| d.parse())
            .context("reading operation")?;
        let divisor: usize = lines
            .next()
            .and_then(|d| d.strip_prefix(DIVISOR_PREFIX))
            .and_then(|l| l.parse().ok())
            .context("reading divisor")?;
        let if_true: usize = lines
            .next()
            .and_then(|d| d.strip_prefix("    If true: throw to monkey "))
            .and_then(|d| d.parse().ok())
            .context("reading if true")?;
        let if_false: usize = lines
            .next()
            .and_then(|d| d.strip_prefix("    If false: throw to monkey "))
            .and_then(|d| d.parse().ok())
            .context("reading if false")?;
        Ok(Self {
            items: items?,
            oper,
            divisor,
            if_true,
            if_false,
            inspects: 0,
        })
    }
}

#[derive(Debug)]
struct Game {
    monkeys: Vec<Monkey>,
    divisors_ppcm: usize,
}

struct Throw {
    item: usize,
    dest: usize,
}
impl FromStr for Game {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let separator = if s.contains("\r\n") {
            "\r\n\r\n"
        } else {
            "\n\n"
        };
        let monkeys: Result<Vec<Monkey>, _> = s.split(separator).map(|m| m.parse()).collect();
        let monkeys = monkeys?;
        let divisors_ppcm: usize = monkeys.iter().map(|m| m.divisor).product();
        Ok(Self {
            monkeys,
            divisors_ppcm,
        })
    }
}

impl Game {
    fn single_play(&mut self, m_idx: usize, decrease_factor: usize) {
        let monkey = &mut self.monkeys[m_idx];
        let mut throws: Vec<Throw> = vec![];
        while let Some(mut item) = monkey.items.pop_front() {
            item = (monkey.oper.apply(item) / decrease_factor) % self.divisors_ppcm;
            monkey.inspects += 1;
            throws.push(Throw {
                item,
                dest: if item % monkey.divisor == 0 {
                    monkey.if_true
                } else {
                    monkey.if_false
                },
            });
        }

        for throw in throws {
            self.monkeys[throw.dest].items.push_back(throw.item);
        }
    }
    fn play_round(&mut self, decrease_factor: usize) {
        let len = self.monkeys.len();
        for i in 0..len {
            self.single_play(i, decrease_factor);
        }
    }
}

pub fn chase_monkeys() {
    let input = include_str!("../resources/day11_monkeys.txt");

    let mut game: Game = input.parse().expect("reading input");
    for _ in 0..20 {
        game.play_round(3);
    }
    let level: usize = game
        .monkeys
        .iter()
        .map(|m| m.inspects)
        .sorted()
        .rev()
        .take(2)
        .product();
    println!("level of monkeys after 20 round is {level}");

    let mut game: Game = input.parse().expect("reading input");
    for _ in 0..10000 {
        game.play_round(1);
    }
    let level: usize = game
        .monkeys
        .iter()
        .map(|m| m.inspects)
        .sorted()
        .rev()
        .take(2)
        .product();
    println!("level of monkeys after 10000 round is {level}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            Monkey 0:
              Starting items: 79, 98
              Operation: new = old * 19
              Test: divisible by 23
                If true: throw to monkey 2
                If false: throw to monkey 3

            Monkey 1:
              Starting items: 54, 65, 75, 74
              Operation: new = old + 6
              Test: divisible by 19
                If true: throw to monkey 2
                If false: throw to monkey 0

            Monkey 2:
              Starting items: 79, 60, 97
              Operation: new = old * old
              Test: divisible by 13
                If true: throw to monkey 1
                If false: throw to monkey 3

            Monkey 3:
              Starting items: 74
              Operation: new = old + 3
              Test: divisible by 17
                If true: throw to monkey 0
                If false: throw to monkey 1
        "};

        let mut game: Game = input.parse().unwrap();
        for _ in 0..20 {
            game.play_round(3);
        }
        assert_eq!(
            10605usize,
            game.monkeys
                .iter()
                .map(|m| m.inspects)
                .sorted()
                .rev()
                .take(2)
                .product()
        );

        let mut game: Game = input.parse().unwrap();
        for _ in 0..10000 {
            game.play_round(1);
        }
        // for m in &game.monkeys{
        //     println!("monkey inspects {}", m.inspects);
        // }
        assert_eq!(
            2713310158usize,
            game.monkeys
                .iter()
                .map(|m| m.inspects)
                .sorted()
                .rev()
                .take(2)
                .product()
        );
    }
}
