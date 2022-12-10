use eyre::{eyre, Context};
use std::str::FromStr;

struct CleanupRange {
    start: usize,
    end: usize,
}

impl CleanupRange {
    pub fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end &&  self.end >= other.start
    }
}

impl FromStr for CleanupRange {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        let mut sections = s.trim().split('-');
        let start: usize = sections
            .next()
            .ok_or(eyre!("no start data"))
            .and_then(|s| s.parse().context("trying to parse range start"))?;
        let end: usize = sections
            .next()
            .ok_or(eyre!("no end data"))
            .and_then(|s| s.parse().context("trying to parse range end"))?;
        assert!(start <= end);
        Ok(Self { start, end })
    }
}

fn count_overlapping_ranges(
    assignments: &str,
    filter: impl Fn(&CleanupRange, &CleanupRange) -> bool,
) -> usize {
    assignments
        .lines()
        .filter_map(|l| {
            let mut ranges = l.split(',');
            let left: Option<CleanupRange> = ranges.next().and_then(|r| r.parse().ok());
            let right: Option<CleanupRange> = ranges.next().and_then(|r| r.parse().ok());
            left.and_then(|l| right.map(|r| (l, r)))
        })
        .filter(|(l, r)| filter(l, r))
        .count()
}

fn count_fully_overlapping_ranges(assignments: &str) -> usize {
    count_overlapping_ranges(assignments, |l, r| l.contains(r) || r.contains(l))
}

fn count_partially_overlapping_ranges(assignments: &str) -> usize {
    count_overlapping_ranges(assignments, |l, r| l.overlaps(r))
}

pub fn clean_camp() {
    let assignments = include_str!("../resources/day4_assignments.txt");
    let full_overlap_count = count_fully_overlapping_ranges(assignments);
    println!("there are {full_overlap_count} fully overlapping assignments");

    let partial_overlap_count = count_partially_overlapping_ranges(assignments);
    println!("there are {partial_overlap_count} partially overlapping assignments");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let assignments = indoc! {"\
            2-4,6-8
            2-3,4-5
            5-7,7-9
            2-8,3-7
            6-6,4-6
            2-6,4-8
        "};

        assert_eq!(2, count_fully_overlapping_ranges(assignments));
        assert_eq!(4, count_partially_overlapping_ranges(assignments));
    }
}
