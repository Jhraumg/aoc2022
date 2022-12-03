use itertools::Itertools;

fn get_most_carrying_elves_charge(foods: &str, number_of_elves: usize) -> usize {
    foods
        .lines()
        .group_by(|l| l.is_empty())
        .into_iter()
        .filter(|(is_sep, _)| !*is_sep)
        .map(|(_, charges)| {
            charges
                .into_iter()
                .map(|c| c.parse::<usize>().unwrap_or(0))
                .sum::<usize>()
        })
        .sorted()
        .rev()
        .take(number_of_elves)
        .sum()
}

pub fn handle_elves_food() {
    let foods = include_str!("../resources/day1_calories.txt");
    let max_charge = get_most_carrying_elves_charge(foods, 1);
    println!("Calories carried by most charged elf : {max_charge}");
    let three_max_charges = get_most_carrying_elves_charge(foods, 3);
    println!("Calories carried by 3 most charged elves : {three_max_charges}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_1_works() {
        let foods = indoc! {"
        1000
        2000
        3000
        
        4000
        
        5000
        6000
        
        7000
        8000
        9000
        
        10000"};

        assert_eq!(24000, get_most_carrying_elves_charge(foods, 1));
        assert_eq!(45000, get_most_carrying_elves_charge(foods, 3));
    }
}
