use std::cmp::min;
use std::str::FromStr;

struct ForestGrid {
    trees: Vec<Vec<u8>>,
}
impl FromStr for ForestGrid {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        let trees: Vec<_> = s
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                l.chars()
                    .filter_map(|c| match c {
                        '0' => Some(0),
                        '1' => Some(1),
                        '2' => Some(2),
                        '3' => Some(3),
                        '4' => Some(4),
                        '5' => Some(5),
                        '6' => Some(6),
                        '7' => Some(7),
                        '8' => Some(8),
                        '9' => Some(9),
                        _ => None,
                    })
                    .collect()
            })
            .collect();

        Ok(Self { trees })
    }
}

impl ForestGrid {
    fn get_row(&self, row_idx: usize) -> Option<&Vec<u8>> {
        self.trees.get(row_idx)
    }
    fn get_column(&self, col_idx: usize) -> Option<Vec<u8>> {
        self.trees
            .iter()
            .map(|row| row.get(col_idx).copied())
            .collect()
    }

    fn is_hidden(&self, row_idx: usize, col_idx: usize) -> bool {
        let max_row_idx = self.trees.len() - 1;
        let max_col_idx = self.trees.get(0).map(|r| r.len() - 1).unwrap_or(0);

        if row_idx == 0 || row_idx == max_row_idx || col_idx == 0 || col_idx == max_col_idx {
            return false;
        }
        let height = self.trees[row_idx][col_idx];
        let row = self
            .get_row(row_idx)
            .expect("looking for visibility outside forest");
        let col = self
            .get_column(col_idx)
            .expect("looking for visibility outside forest");

        row.iter().take(col_idx).any(|h| *h >= height)
        && row.iter().rev()
                .take(max_row_idx - col_idx)
                .any(|h| *h >= height)
        && col.iter().take(row_idx).any(|h| *h >= height)
        && col.iter().rev()
                .take(max_col_idx - row_idx)
                .any(|h| *h >= height)
    }
    fn count_visible_trees(&self) -> usize {
        let max_row_idx = self.trees.len() - 1;
        let max_col_idx = self.trees.get(0).map(|r| r.len() - 1).unwrap_or(0);

        (0..=max_row_idx)
            .flat_map(move |i| (0..=max_col_idx).map(move |j| (i, j)))
            .filter(|(i, j)| !self.is_hidden(*i, *j))
            .count()
    }

    fn compute_view_score(&self, row_idx: usize, col_idx: usize) -> usize {
        let col = self.get_column(col_idx).expect("too right");
        let row = self.get_row(row_idx).expect("too down");
        let height = self.trees[row_idx][col_idx];

        let left = min(
            col_idx,
            1+ row[0..col_idx].iter().rev().take_while(|h| **h < height).count());
        let right = if col_idx < row.len() - 1 {
            min(
                1 + row[col_idx + 1..col.len()].iter().take_while(|h| **h < height).count(),
                row.len() - 1 - col_idx)
        } else {
            0
        };

        let up = min(
            row_idx,
            1 + col[0..row_idx].iter().rev().take_while(|h| **h < height).count());
        let down = if row_idx < col.len() - 1 {
            min(
                col.len() - 1 - row_idx,
                1 + col[row_idx + 1..row.len()].iter().take_while(|h| **h < height).count())
        } else {
            0
        };

        left * right * up * down
    }

    fn compute_best_view_score(&self) -> usize {
        (0..self.trees.len())
            .flat_map(move |row_idx| {
                (0..self.trees[0].len())
                    .map(move |col_idx| self.compute_view_score(row_idx, col_idx))
            })
            .max()
            .unwrap_or(0)
    }
}

pub fn build_tree_house() {
    let forest: ForestGrid = include_str!("../resources/day8_trees_heights.txt")
        .parse()
        .expect("could not parse forest");
    let visible_trees_count = forest.count_visible_trees();
    println!("number of visible trees : {visible_trees_count}");

    let best_view_score = forest.compute_best_view_score();
    println!("best view score  : {best_view_score}");
}
#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let forest: ForestGrid = indoc! {"
            30373
            25512
            65332
            33549
            35390
        "}
        .parse()
        .expect("could not parse tree grid");

        assert!(forest.is_hidden(1, 3));
        assert!(forest.is_hidden(3, 1));

        assert_eq!(21, forest.count_visible_trees());

        assert_eq!(4, forest.compute_view_score(1, 2));
        assert_eq!(8, forest.compute_view_score(3, 2));
    }
}
