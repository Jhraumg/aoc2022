use eyre::{Context, ContextCompat};
use std::cmp::max;
use std::str::FromStr;

#[derive(Debug)]
struct Stock {
    crates: Vec<Vec<char>>,
}

//[B] [R] [B] [C] [D] [H] [D] [C] [N]
//.1...5...9...13
impl FromStr for Stock {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        let stock: Vec<Vec<(usize, char)>> = s
            .lines()
            .rev()
            .filter(|l| l.contains('['))
            .map(|l| {
                let crates: Vec<_> = l
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| c.is_alphabetic())
                    .map(|(i, c)| ((i - 1) / 4, c))
                    .collect();
                crates
            })
            .collect();
        let stack_nb: usize = 1 + stock
            .get(0)
            .and_then(|line| line.last().map(|(i, _)| *i))
            .unwrap_or(0);
        let mut crates = vec![vec![]; stack_nb];

        for line in stock {
            for (i, cr) in line {
                crates.get_mut(i).unwrap().push(cr);
            }
        }

        Ok(Self { crates })
    }
}
impl Stock {
    fn rearange_step(
        &mut self,
        source_stack_index: usize,
        target_stack_index: usize,
        quantity: usize,
        keep_order: bool,
    ) {
        let len = self.crates.len();
        let max_index = max(source_stack_index, target_stack_index);
        if max_index >= len {
            self.crates.append(&mut vec![vec![]; max_index + 1 - len]);
        }

        let source = self.crates.get_mut(source_stack_index).unwrap();
        let mut moved_crates: Vec<_> = (0..quantity).filter_map(|_| source.pop()).collect();
        if keep_order {
            moved_crates.reverse();
        }
        self.crates
            .get_mut(target_stack_index)
            .unwrap()
            .append(&mut moved_crates);
    }
}

#[derive(Debug)]
struct CrateMove {
    quantity: usize,
    from: usize,
    to: usize,
}

impl FromStr for CrateMove {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        let mut command = s.trim().split(' ');
        // partial parse
        command.next();
        let quantity = command
            .next()
            .context("reading move")
            .and_then(|q| q.parse().context("parsing quantity"))?;
        command.next();
        let from = command
            .next()
            .context("reading from")
            .and_then(|q| q.parse().context("parsing from"))?;
        command.next();
        let to = command
            .next()
            .context("reading to")
            .and_then(|q| q.parse().context("parsing to"))?;

        Ok(Self { quantity, from, to })
    }
}

fn arrange_stock(stock_and_moves: &str, keep_order: bool) -> String {
    let mut stock: Stock = stock_and_moves.parse().expect("could not parse stock");
    let moves: Vec<CrateMove> = stock_and_moves
        .lines()
        .filter(|l| l.starts_with("move "))
        .map(|l| l.parse().unwrap())
        .collect();

    for mv in moves {
        stock.rearange_step(mv.from - 1, mv.to - 1, mv.quantity, keep_order);
    }

    stock
        .crates
        .iter()
        .filter_map(|stack| stack.iter().rev().next())
        .collect()
}

pub fn supply_stack() {
    let stock_and_moves = include_str!("../resources/day5_stocks_and_moves.txt");
    let tops_9000 = arrange_stock(stock_and_moves, false);
    println!("top crates 9000 : {tops_9000}");
    let tops_9001 = arrange_stock(stock_and_moves, true);
    println!("top crates 9001 : {tops_9001}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let stock_and_moves = indoc! {"
            [D]
        [N] [C]
        [Z] [M] [P]
         1   2   3

        move 1 from 2 to 1
        move 3 from 1 to 3
        move 2 from 2 to 1
        move 1 from 1 to 2
        "};
        assert_eq!("CMZ", arrange_stock(stock_and_moves, false));
        assert_eq!("MCD", arrange_stock(stock_and_moves, true));
    }
}
