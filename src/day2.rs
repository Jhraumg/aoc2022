use std::str::FromStr;
use eyre::{ContextCompat, eyre};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

fn move_score(mv: Move) -> usize {
    match mv {
        Move::Rock => { 1 }
        Move::Paper => { 2 }
        Move::Scissors => { 3 }
    }
}

const LOSE_SCORE: usize = 0;
const DRAW_SCORE: usize = 3;
const WIN_SCORE: usize = 6;

fn round_score(other_move: Move, your_move: Move) -> usize {
    if other_move == your_move {
        return DRAW_SCORE + move_score(your_move);
    }
    move_score(your_move) + match your_move {
        Move::Rock => { if other_move == Move::Scissors { WIN_SCORE } else { LOSE_SCORE } }
        Move::Paper => { if other_move == Move::Rock { WIN_SCORE } else { LOSE_SCORE } }
        Move::Scissors => { if other_move == Move::Paper { WIN_SCORE } else { LOSE_SCORE } }
    }
}

impl FromStr for Move {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars().next()
            .and_then(|c| match c {
                'A' => Some(Self::Rock),
                'X' => Some(Self::Rock),
                'B' => Some(Self::Paper),
                'Y' => Some(Self::Paper),
                'C' => Some(Self::Scissors),
                'Z' => Some(Self::Scissors),
                _ => None
            })
            .ok_or(eyre!("cannot convert {s} to Move"))
    }
}

fn read_round_score(round: &str) -> usize {
    let mut moves = round.split(' ');
    let other_move: Move = moves.next().and_then(|m| m.parse().ok()).expect("could not convert other move from {move}");
    let your_move: Move = moves.next().and_then(|m| m.parse().ok()).expect("could not convert your move from {move}");
    round_score(other_move, your_move)
}



#[derive(Debug, Clone, Copy)]
enum Strategy {
    Win,
    Draw,
    Loose,
}

impl FromStr for Strategy {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars().next().and_then(|c| match c {
            'X' => Some(Self::Loose),
            'Y' => Some(Self::Draw),
            'Z' => Some(Self::Win),
            _ => None
        }).context("could not convert {s} to Strategy")
    }
}

fn get_move(other_move: Move, s: Strategy) -> Move {
    match s {
        Strategy::Draw => { other_move },
        Strategy::Loose => {
            match other_move {
                Move::Rock => { Move::Scissors }
                Move::Paper => { Move::Rock }
                Move::Scissors => { Move::Paper }
            }
        },
        Strategy::Win => {
            match other_move {
                Move::Rock => { Move::Paper }
                Move::Paper => { Move::Scissors }
                Move::Scissors => { Move::Rock }
            }
        }
    }
}

fn read_round_score_with_strategy(round : &str) -> usize {
    let mut datas = round.split(' ');
    let other_move :Move = datas.next()
        .ok_or(eyre!("no data for other move"))
        .and_then(|d| d.parse::<Move>())
        .unwrap();
    let strategy : Strategy = datas.next()
        .ok_or(eyre!("no data for strategy"))
        .and_then(|d|d.parse::<Strategy>())
        .unwrap();

    round_score(other_move, get_move(other_move,strategy))
}


fn compute_full_score(rounds: &str, round_compute : impl Fn(&str) ->usize) -> usize {
    rounds
        .lines()
        .filter(|l| !l.is_empty())
        .map(|round| round_compute(round))
        .sum()
}


pub fn play_rock_paper_scissors() {
    let rounds = include_str!("../resources/day2_rock_paper_scissors.txt");
    let score = compute_full_score(rounds, read_round_score);
    let score_with_strategy = compute_full_score(rounds, read_round_score_with_strategy);

    println!("full  rock/paper/scissors score : {score}");
    println!("full  rock/paper/scissors score with strateggy : {score_with_strategy}");
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use super::*;

    #[test]
    fn aoc_example_works() {
        let rounds = indoc! {"\
            A Y
            B X
            C Z
        "};
        assert_eq!(15, compute_full_score(rounds, read_round_score));
        assert_eq!(12, compute_full_score(rounds, read_round_score_with_strategy));
    }
}