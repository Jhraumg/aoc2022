fn to_index(current_index: usize, offset: isize, len: usize) -> usize {
    assert!(
        (0..len).contains(&current_index),
        "to_index works only from an already bound value"
    );
    assert_ne!(0, offset, "if no offset, no need to move indexes");
    //
    // if current_index + offset >= len
    // - move of len - current_index -1 => current_index = offset is now 0
    //   (directly from last cell)
    //   offset = offset +current_index -len +1
    // - then current_index = 0 + offset%(len-1)

    let len = len as isize;
    let current_index = current_index as isize;
    let mut offset = offset;
    match current_index + offset {
        wrap_right if wrap_right >= len => {
            offset += current_index - len + 1;
            (offset % (len - 1)) as usize
        }
        wrap_left if wrap_left <= 0 => {
            offset += current_index; // going back to len-1
            (len - 1 + offset % (len - 1)) as usize
        }
        _ => (current_index + offset) as usize,
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct Coordinate {
    value: isize,
    rank: usize,
}

fn reorder_numbers(numbers: &mut [Coordinate], factor: usize, times: usize) {
    let len = numbers.len();

    let mut current_index = 0;

    for Coordinate { value, rank: _ } in numbers.iter_mut() {
        *value *= factor as isize;
    }
    for _ in 0..times {
        for i in 0..len {
            if i != numbers[current_index].rank {
                // print!("*");
                current_index = numbers
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.rank == i)
                    .map(|(i, _)| i)
                    .unwrap();
            }

            let number = numbers[current_index];

            if number.value == 0 {
                current_index = (current_index + 1) % len;
                continue;
            }
            let new_index = to_index(current_index, number.value, len);
            // println!("\nmoving {number} ({current_index} => {new_index})");

            if new_index < current_index {
                for i in 0..(current_index - new_index) {
                    numbers[current_index - i] = numbers[current_index - i - 1];
                }
                current_index = (current_index + 1) % len;
            } else {
                for i in 0..(new_index - current_index) {
                    if current_index + i + 1 == len {
                        println!("{current_index} => {new_index} ({i})");
                    }
                    numbers[current_index + i] = numbers[current_index + i + 1];
                }
            }
            numbers[new_index] = number;
        }
    }
}

fn summ_offsets(numbers: &[Coordinate]) -> isize {
    let start_index = numbers
        .iter()
        .enumerate()
        .find(|(_, number)| number.value == 0)
        .map(|(i, _)| i)
        .unwrap();
    let len = numbers.len();
    [1000, 2000, 3000]
        .into_iter()
        .map(|i| numbers[(start_index + i) % len].value)
        .sum()
}

pub fn reach_elves() {
    let encrypted_coordinates: Vec<Coordinate> =
        include_str!("../resources/day20_encrypted_coordinates.txt")
            .lines()
            .enumerate()
            .filter_map(|(rank, l)| {
                l.parse::<isize>()
                    .map(|value| Coordinate { value, rank })
                    .ok()
            })
            .collect();
    {
        let mut encrypted_coordinates = encrypted_coordinates.clone();
        reorder_numbers(&mut encrypted_coordinates, 1, 1);
        let coordinates_sum = summ_offsets(&encrypted_coordinates);
        println!("sum of coordinates : {coordinates_sum}");
    }
    {
        let mut encrypted_coordinates = encrypted_coordinates;
        reorder_numbers(&mut encrypted_coordinates, 811589153, 10);
        let coordinates_sum = summ_offsets(&encrypted_coordinates);
        println!("actual sum of coordinates : {coordinates_sum}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn aoc_example_works() {
        assert!((-1isize % 16).is_negative());
        let numbers: Vec<Coordinate> = [1, 2, -3, 3, -2, 0, 4]
            .into_iter()
            .enumerate()
            .map(|(rank, value)| Coordinate { rank, value })
            .collect();
        {
            let mut numbers = numbers.clone();
            reorder_numbers(&mut numbers, 1, 1);
            assert_eq!(
                vec![1, 2, -3, 4, 0, 3, -2],
                numbers.iter().map(|c| c.value).collect_vec()
            );
            assert_eq!(0, numbers[4 % numbers.len()].value);
            assert_eq!(4, numbers[1004 % numbers.len()].value);
            assert_eq!(-3, numbers[2004 % numbers.len()].value);
            assert_eq!(2, numbers[3004 % numbers.len()].value);
            assert_eq!(3, summ_offsets(&numbers));
        }
        let mut numbers = numbers.clone();
        reorder_numbers(&mut numbers, 811589153, 10);
        assert_eq!(1623178306, summ_offsets(&numbers));
    }
}
