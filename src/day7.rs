use std::collections::HashMap;

use eyre::{Context, ContextCompat};

#[derive(Debug)]
struct Dir {
    direct_files_size: usize,
}
#[derive(Debug)]
struct File {
    //    name : &'static [u8],
    size: usize,
}

struct FileSystem {
    dirs: HashMap<Vec<&'static [u8]>, Dir>, // use a Trie
    current_dir: Vec<&'static [u8]>,
}
enum Line {
    Cd(&'static [u8]),
    Ls,
    D(&'static [u8]),
    F(File),
}

impl Line {
    fn try_from_str(s: &'static str) -> eyre::Result<Self> {
        if s.starts_with("$ cd ") {
            return Ok(Self::Cd(&s.trim().as_bytes()[5..]));
        }
        if s.starts_with("$ ls") {
            return Ok(Self::Ls);
        }

        if s.starts_with("dir ") {
            return Ok(Self::D(&s.trim().as_bytes()[4..]));
        }

        let size: usize = s
            .split(' ')
            .next()
            .context("reading size")?
            .parse()
            .context("parsing size")?;
        Ok(Self::F(File { size }))
    }
}

impl FileSystem {
    fn new() -> Self {
        Self {
            dirs: HashMap::new(),
            current_dir: vec![],
        }
    }
    fn change_dir(&mut self, target: &'static [u8]) {
        match target {
            t if t == "..".as_bytes() => {
                self.current_dir.pop();
            }
            t if t == "/".as_bytes() => {
                self.current_dir.clear();
            }
            t => {
                for subdir in t.split(|c| *c == b'/') {
                    self.current_dir.push(subdir);
                }
            }
        }
    }
    fn add_file(&mut self, file: File) {
        let dir = self.dirs.get_mut(&self.current_dir);
        if let Some(d) = dir {
            d.direct_files_size += file.size;
        } else {
            self.dirs.insert(
                self.current_dir.clone(),
                Dir {
                    direct_files_size: file.size,
                },
            );
        }
    }

    fn parse_line(&mut self, raw_line: &'static str) -> eyre::Result<()> {
        let line: Line = Line::try_from_str(raw_line)?;
        match line {
            Line::Cd(cd) => {
                self.change_dir(cd);
            }
            Line::Ls => {}
            Line::D(_) => {}
            Line::F(f) => self.add_file(f),
        }
        Ok(())
    }

    fn get_total_size(&self, dir: &Vec<&[u8]>) -> usize {
        self.dirs
            .iter()
            .filter(|(path, _)| {
                path.len() >= dir.len() && path.iter().zip(dir.iter()).all(|(p, d)| **p == **d)
            })
            .map(|(_, d)| d.direct_files_size)
            .sum()
    }

    fn sum_size_under_threshold(&self, threshold: usize) -> usize {
        let direct_dirs: Vec<_> = self
            .dirs
            .iter()
            .filter(|(_, d)| d.direct_files_size <= threshold)
            .filter(|(p, _)| self.get_total_size(p) <= threshold)
            .collect();

        // FIXME : get_total_size is not as efficient as summing sizes all the way down (expect if threshold is low)
        let mut filtered_dir_to_size: HashMap<Vec<&[u8]>, usize> =
            HashMap::with_capacity(direct_dirs.len());
        'main: for d in direct_dirs {
            let path = d.0.clone();
            if filtered_dir_to_size.contains_key(&path) {
                continue 'main;
            }
            let size = self.get_total_size(&path);
            filtered_dir_to_size.insert(path.clone(), size);

            let mut path = path.clone();
            while !path.is_empty() {
                path.pop();
                if filtered_dir_to_size.contains_key(&path) {
                    continue 'main;
                }
                let size = self.get_total_size(&path);
                if size > threshold {
                    continue 'main;
                }
                filtered_dir_to_size.insert(path.clone(), size);
            }
        }
        filtered_dir_to_size.values().sum()
    }
}

pub fn update_handled() {
    let input = include_str!("../resources/day7_file_system.txt");
    let mut fs = FileSystem::new();
    for l in input.lines() {
        fs.parse_line(l).expect("could parse {l}");
    }
    let size_under_100000_sum = fs.sum_size_under_threshold(100000);
    println!("sum of dir under 100000 size {size_under_100000_sum}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            $ cd /
            $ ls
            dir a
            14848514 b.txt
            8504156 c.dat
            dir d
            $ cd a
            $ ls
            dir e
            29116 f
            2557 g
            62596 h.lst
            $ cd e
            $ ls
            584 i
            $ cd ..
            $ cd ..
            $ cd d
            $ ls
            4060174 j
            8033020 d.log
            5626152 d.ext
            7214296 k
        "};
        let mut fs = FileSystem::new();
        for l in input.lines() {
            fs.parse_line(l).expect("could parse {l}");
        }
        assert_eq!(95437, fs.sum_size_under_threshold(100000));
    }
}
