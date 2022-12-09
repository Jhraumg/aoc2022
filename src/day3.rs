use itertools::Itertools;
use std::collections::HashSet;

fn get_priority(item_type: char) -> usize {
    match item_type {
        c if ('a'..='z').contains(&c) => c as usize - 'a' as usize + 1,
        c if ('A'..='Z').contains(&c) => c as usize - 'A' as usize + 27,
        _ => 0,
    }
}
fn get_rucksack_unranged_item_type(rucksack: &str) -> Option<char> {
    let len = rucksack.len();
    assert_eq!(0, len % 2, "rucksacks must be of even size");
    let lefties: HashSet<_> = rucksack[0..len / 2].chars().collect();
    let righties: HashSet<_> = rucksack[len / 2..len].chars().collect();

    lefties.intersection(&righties).next().copied()
}

fn sum_unarranged_item_type(rucksacks: &str) -> usize {
    rucksacks
        .lines()
        .filter_map(get_rucksack_unranged_item_type)
        .map(get_priority)
        .sum()
}

fn sum_groups_badge_priorities(rucksacks: &str) -> usize {
    let mut sum = 0;
    let mut lines = rucksacks.lines();
    while let (Some(first), Some(second), Some(third)) = (lines.next(), lines.next(), lines.next())
    {
        let firsts: HashSet<_> = first.chars().collect();
        let seconds: HashSet<_> = second.chars().collect();
        let thirds: HashSet<_> = third.chars().collect();

        let first_and_seconds: HashSet<_> = firsts.intersection(&seconds).copied().collect();

        sum += first_and_seconds
            .intersection(&thirds)
            .next()
            .copied()
            .map(get_priority)
            .unwrap_or(0);
    }
    sum
}

pub fn arrange_ruckacks() {
    let rucksacks = include_str!("../resources/day3_rucksacks.txt");
    let unarranged_sum = sum_unarranged_item_type(rucksacks);
    println!("sum of the unarranged item types priorities : {unarranged_sum}");

    let badges_sum = sum_groups_badge_priorities(rucksacks);
    println!("sum of the badges priorities : {badges_sum}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn char_can_be_converted_to_preiorities() {
        assert_eq!(get_priority('b'), 2);
        assert_eq!(get_priority('B'), 28);
    }

    #[test]
    fn aoc_example_works() {
        let rucksacks = indoc! {"\
            vJrwpWtwJgWrhcsFMMfFFhFp
            jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
            PmmdzqPrVvPwwTWBwg
            wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
            ttgJtRGJQctTZtZT
            CrZsJsPPZsGzwwsLwLmpwMDw
        "};
        assert_eq!(sum_unarranged_item_type(rucksacks), 157);
        assert_eq!(sum_groups_badge_priorities(rucksacks), 70);
    }
}
