use itertools::{merge, Itertools};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Write};

#[derive(PartialEq, Eq, Clone, Debug)]
enum PacketData {
    List(Vec<PacketData>),
    Int(usize),
}

impl Display for PacketData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketData::List(datas) => {
                f.write_char('[')?;
                for d in datas {
                    f.write_str(&format!("{d},"))?;
                }
                f.write_char(']')
            }
            PacketData::Int(v) => f.write_str(&v.to_string()),
        }
    }
}

impl PacketData {
    fn cmp_list(first: &Vec<PacketData>, other: &Vec<PacketData>) -> Ordering {
        let length_independant_ord = first
            .iter()
            .zip(other)
            .map(|(s, o)| s.cmp(o))
            .find(|c| *c != Ordering::Equal)
            .unwrap_or(Ordering::Equal);
        match length_independant_ord {
            Ordering::Equal => first.len().cmp(&other.len()),
            _ => length_independant_ord,
        }
    }

    fn new(input: &str) -> (Self, usize) {
        match input.as_bytes()[0] {
            b'[' => {
                let mut datas: Vec<PacketData> = vec![];
                let mut start_idx = 1usize;

                'main: while let Some(end_idx) = input[start_idx..].find(&['[', ',', ']'][..]) {
                    if end_idx > 0 {
                        datas.push(Self::new(&input[start_idx..(start_idx + end_idx)]).0);
                    }
                    match input.as_bytes()[start_idx + end_idx] {
                        b'[' => {
                            let (data, len) = Self::new(&input[(start_idx + end_idx)..]);

                            datas.push(data);
                            start_idx += len - 1;
                        }
                        b']' => {
                            start_idx += end_idx + 1;
                            break 'main;
                        }
                        b',' => {}
                        _ => {}
                    }
                    let end = start_idx + end_idx + 1;
                    start_idx = end;
                }
                (Self::List(datas), start_idx)
            }
            b']' => {
                panic!("unexpected char ']'");
            }
            _ => {
                let len = input
                    .as_bytes()
                    .iter()
                    .enumerate()
                    .find(|(_, c)| !c.is_ascii_digit())
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| input.len());
                let val: usize = input[0..len].parse().unwrap();
                (Self::Int(val), len)
            }
        }
    }
}

impl PartialOrd<Self> for PacketData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PacketData {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            PacketData::List(datas) => match other {
                PacketData::List(other_datas) => PacketData::cmp_list(datas, other_datas),
                PacketData::Int(ov) => PacketData::cmp_list(datas, &vec![PacketData::Int(*ov)]),
            },
            PacketData::Int(v) => match other {
                PacketData::List(other_datas) => {
                    PacketData::cmp_list(&vec![PacketData::Int(*v)], other_datas)
                }
                PacketData::Int(ov) => v.cmp(ov),
            },
        }
    }
}

fn parse_pairs(input: &str) -> Vec<Vec<PacketData>> {
    let sep = if input.contains("\r\n") {
        "\r\n\r\n"
    } else {
        "\n\n"
    };
    input
        .split(sep)
        .map(|input| input.lines().map(|l| PacketData::new(l).0).collect())
        .collect()
}

fn sum_ordered_pairs(input: &str) -> usize {
    let pairs = parse_pairs(input);

    pairs
        .into_iter()
        .enumerate()
        .filter(|(_, p)| p[0] < p[1])
        .map(|(i, _)| i + 1)
        .sum()
}

fn decode(input: &str) -> usize {
    let packets = merge(
        input.lines().filter(|l| !l.is_empty()),
        "[[2]]\n[[6]]".lines(),
    )
    .map(|l| PacketData::new(l).0)
    .sorted();

    let div1 = PacketData::new("[[2]]").0;
    let div2 = PacketData::new("[[6]]").0;

    packets
        .enumerate()
        .filter(|(_, p)| p == &div1 || p == &div2)
        .map(|(i, _)| i + 1)
        .product()
}
pub fn distress_signal() {
    let input = include_str!("../resources/day13_pairs.txt");
    let sum = sum_ordered_pairs(input);
    println!("valid pairs sum {sum}");

    let key = decode(input);
    println!("decoder key {key}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            [1,1,3,1,1]
            [1,1,5,1,1]

            [[1],[2,3,4]]
            [[1],4]

            [9]
            [[8,7,6]]

            [[4,4],4,4]
            [[4,4],4,4,4]

            [7,7,7,7]
            [7,7,7]

            []
            [3]

            [[[]]]
            [[]]

            [1,[2,[3,[4,[5,6,7]]]],8,9]
            [1,[2,[3,[4,[5,6,0]]]],8,9]
        "};
        assert_eq!(13, sum_ordered_pairs(input));
        assert_eq!(140, decode(input));
    }
}
