use eyre::eyre;
use std::fmt::{Debug, Display, Formatter, Write};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SnafuDigit {
    Zero = 0,
    One = 1,
    Two = 2,
    Minus = -1,
    DoubleMinus = -2,
}

impl TryFrom<char> for SnafuDigit {
    type Error = eyre::Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Zero),
            '1' => Ok(Self::One),
            '2' => Ok(Self::Two),
            '-' => Ok(Self::Minus),
            '=' => Ok(Self::DoubleMinus),

            _ => Err(eyre!("cannot convert '{c}' to SnafuDigit")),
        }
    }
}

impl TryFrom<isize> for SnafuDigit {
    type Error = eyre::Error;

    fn try_from(v: isize) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            -1 => Ok(Self::Minus),
            -2 => Ok(Self::DoubleMinus),

            _ => Err(eyre!("cannot convert '{v}' to (single) SnafuDigit")),
        }
    }
}

impl Display for SnafuDigit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            SnafuDigit::Zero => '0',
            SnafuDigit::One => '1',
            SnafuDigit::Two => '2',
            SnafuDigit::Minus => '-',
            SnafuDigit::DoubleMinus => '=',
        })
    }
}

const SNAFU_DIGIT_VALUES_COUNT: usize = 5;

#[derive(Debug, Eq, PartialEq)]
struct SnafuNumber {
    digits: Vec<SnafuDigit>,
}
impl Display for SnafuNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for snafu_digit in self.digits.iter().rev() {
            std::fmt::Display::fmt(&snafu_digit, f)?;
        }
        Ok(())
    }
}

impl FromStr for SnafuNumber {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits: Result<Vec<SnafuDigit>, _> = s.chars().rev().map(|c| c.try_into()).collect();
        let digits = digits?;
        Ok(Self { digits })
    }
}

impl From<usize> for SnafuNumber {
    fn from(value: usize) -> Self {
        if value == 0 {
            return Self {
                digits: vec![SnafuDigit::Zero],
            };
        }
        let mut digits = vec![];
        let mut remaining = value;
        let mut carry = 0isize;
        while remaining > 0 || carry != 0 {
            let mut digit = (remaining % SNAFU_DIGIT_VALUES_COUNT) as isize + carry;
            carry = 0;
            if digit > 2 {
                carry = 1;
                digit -= 5;
            }
            remaining /= 5;
            digits.push(digit.try_into().expect("digit MUST be in [-2,2] here !"));
        }
        Self { digits }
    }
}

impl From<&SnafuNumber> for isize {
    fn from(value: &SnafuNumber) -> Self {
        let mut result = 0;
        let mut factor = 1;
        for digit in &value.digits {
            result += factor * *digit as isize;
            factor *= 5;
        }
        result
    }
}

fn sum_snafus(snafus: &str) -> SnafuNumber {
    let sum: isize = snafus
        .lines()
        .filter_map(|l| l.parse::<SnafuNumber>().ok())
        .map(|snafu| isize::from(&snafu))
        .sum();
    assert!(sum >= 0, "negative number snafu repr not implemented yet !");

    SnafuNumber::from(sum as usize)
}

pub fn calibrate_bob() {
    let snafus = include_str!("../resources/day25_snafus.txt");
    let sum = sum_snafus(snafus);

    println!("type {sum} to calibrate Bob");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn snafu_can_be_converted() {
        let snafu = SnafuNumber::from(99);
        assert_eq!(isize::from(&snafu), 99);
        let numbers: Vec<isize> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20, 2022, 12345, 314159265, 1747, 906, 198, 11, 201,
            31, 1257, 32, 353, 107, 7, 3, 37,
        ];
        let snafus: Vec<SnafuNumber> = indoc! {"
            1
            2
            1=
            1-
            10
            11
            12
            2=
            2-
            20
            1=0
            1-0
            1=11-2
            1-0---0
            1121-1110-1=0
            1=-0-2
            12111
            2=0=
            21
            2=01
            111
            20012
            112
            1=-1=
            1-12
            12
            1=
            122
        "}
        .lines()
        .filter_map(|l| l.parse().ok())
        .collect();
        assert_eq!(numbers.len(), snafus.len());
        for (number, snafu) in numbers.into_iter().zip(snafus.into_iter()) {
            let from_snafu = isize::from(&snafu);
            assert_eq!(
                number, from_snafu,
                "{snafu} converted to {from_snafu} instead of {number}"
            );

            let to_snafu = SnafuNumber::from(number as usize);
            assert_eq!(
                to_snafu, snafu,
                "{number} converted to {to_snafu} instead of {snafu}"
            );
        }
    }

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            1=-0-2
            12111
            2=0=
            21
            2=01
            111
            20012
            112
            1=-1=
            1-12
            12
            1=
            122
        "};
        let sum = sum_snafus(input);
        assert_eq!(isize::from(&sum), 4890);
        assert_eq!(sum, "2=-1=0".parse::<SnafuNumber>().unwrap());
    }
}
