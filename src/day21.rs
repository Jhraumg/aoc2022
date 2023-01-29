use crate::day21::Source::{Ref, Val};
use eyre::{eyre, ContextCompat};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::iter::once;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Source<'s> {
    Ref(&'s str),
    Val(isize),
}

impl<'s> Source<'s> {
    fn try_new(s: &'s str) -> eyre::Result<Self> {
        s.trim()
            .parse::<isize>()
            .map(|v| Val(v))
            .or_else(|_| Ok(Ref(s.trim())))
    }

    fn get_name(&self) -> Option<&'s str> {
        match self {
            Ref(name) => Some(name),
            Val(_) => None,
        }
    }
}

impl<'s> Display for Source<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match  self {
            Ref(name) => {std::fmt::Display::fmt(&name, f)}
            Val(v) => {std::fmt::Display::fmt(v, f)}
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Shout<'s> {
    Add((Source<'s>, Source<'s>)),
    Sub((Source<'s>, Source<'s>)),
    Mul((Source<'s>, Source<'s>)),
    Div((Source<'s>, Source<'s>)),
    Val(Source<'s>),
}

impl<'s> Display for Shout<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Shout::Add((a,b)) => {f.write_str(&format!("{a} + {b}"))}
            Shout::Sub((a,b)) => {f.write_str(&format!("{a} - {b}"))}
            Shout::Mul((a,b)) => {f.write_str(&format!("{a} * {b}"))}
            Shout::Div((a,b)) => {f.write_str(&format!("{a} / {b}"))}
            Shout::Val(s) => {std::fmt::Display::fmt(s, f)}
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Shouter<'s> {
    name: &'s str,
    shout: Shout<'s>,
}

impl<'s> Display for Shouter<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.name, f)?;
        std::fmt::Display::fmt(" = ",f)?;
        std::fmt::Display::fmt(&self.shout, f)

    }
}


impl<'s> Shouter<'s> {
    fn try_new(s: &'s str) -> eyre::Result<Self> {
        let mut words = s.split_whitespace();
        let name = words
            .next()
            .map(|name| &name[..name.len() - 1])
            .context("reading name")?;

        let source_a: Source = words
            .next()
            .map_or_else(|| Err(eyre!("no first source")), Source::try_new)?;

        match words.next() {
            None => Ok(Self {
                name,
                shout: Shout::Val(source_a),
            }),
            Some(op) => {
                let source_b: Source = words
                    .next()
                    .map_or_else(|| Err(eyre!("no second source")), Source::try_new)?;
                match op {
                    "+" => Ok(Self {
                        name,
                        shout: Shout::Add((source_a, source_b)),
                    }),
                    "-" => Ok(Self {
                        name,
                        shout: Shout::Sub((source_a, source_b)),
                    }),
                    "*" => Ok(Self {
                        name,
                        shout: Shout::Mul((source_a, source_b)),
                    }),
                    "/" => Ok(Self {
                        name,
                        shout: Shout::Div((source_a, source_b)),
                    }),
                    _ => Err(eyre!("'{op}' is not an operator")),
                }
            }
        }
    }

    /// get an equivalent equality, promoting first source
    fn promote_source(&self, source: &'s str) -> Option<Self> {
        println!("{self} provote {source}");
        match self.shout {
            Shout::Add((Ref(name), b)) if name == source => Some(Shouter {
                name,
                shout: Shout::Sub((Ref(self.name), b)),
            }),
            Shout::Add((a, Ref(name))) if name == source => Some(Shouter {
                name,
                shout: Shout::Sub((Ref(self.name), a)),
            }),
            Shout::Sub((Ref(name), b)) if name == source => Some(Shouter {
                name,
                shout: Shout::Add((Ref(self.name), b)),
            }),
            Shout::Sub((a, Ref(name))) if name == source => Some(Shouter {
                name,
                shout: Shout::Sub((a, Ref(self.name))),
            }),
            Shout::Mul((Ref(name), b)) if name == source => Some(Shouter {
                name,
                shout: Shout::Div((Ref(self.name), b)),
            }),
            Shout::Mul((a, Ref(name))) if name == source => Some(Shouter {
                name,
                shout: Shout::Div((Ref(self.name), a)),
            }),
            Shout::Div((Ref(name), b)) if name == source => Some(Shouter {
                name,
                shout: Shout::Mul((Ref(self.name), b)),
            }),
            Shout::Div((a, Ref(name))) if name == source => Some(Shouter {
                name,
                shout: Shout::Div((a, Ref(self.name))),
            }),
            Shout::Val(Ref(name)) if name == source => Some(Shouter {
                name,
                shout: Shout::Val(Ref(self.name)),
            }),
            _ => None,
        }
    }

    fn replace_known_vals(&self, known_vals_by_name: &mut HashMap<&'s str, isize>) -> Self{

        fn replace_source<'s>(source : Source<'s>, known_vals_by_name: &HashMap<&'s str, isize>) -> Source<'s> {
            match source {
                Ref(name) => known_vals_by_name.get(name).map(|val|Val(*val)).unwrap_or(source),
                _ => source
            }
        }
        fn replace_sources<'s>(sources : (Source<'s>,Source<'s>), known_vals_by_name: &mut HashMap<&'s str, isize>) -> (Source<'s>,Source<'s>) {
            (replace_source(sources.0, known_vals_by_name),replace_source(sources.1, known_vals_by_name))
        }

        if let Some ((name,val)) = resolve(self, known_vals_by_name) {
            known_vals_by_name.insert(name, val);
            return Self{name, shout:Shout::Val(Val(val))};
        }

        let shout = match self.shout {
            Shout::Add((a,b)) => {Shout::Add(replace_sources((a,b),known_vals_by_name))},
            Shout::Sub((a,b)) => {Shout::Sub(replace_sources((a,b),known_vals_by_name))},
            Shout::Mul((a,b)) => {Shout::Mul(replace_sources((a,b),known_vals_by_name))},
            Shout::Div((a,b)) => {Shout::Div(replace_sources((a,b),known_vals_by_name))},
            Shout::Val(src) => {Shout::Val(replace_source(src, known_vals_by_name))}
        };
        let shout = match shout {
            Shout::Add((Ref(a),Ref(b)))  if a == b => { Shout::Mul((Val(2), Ref(a)))}
            Shout::Add((Ref(a),Ref(b)))  if a == b => { Shout::Val(Val(0))}
            Shout::Div((Ref(a),Ref(b))) if a == b => { Shout::Val(Val(1))} // ...
            _ => shout,
        };
        let result = Self{name:self.name, shout};
        if let Shout::Val (Val(val)) = result.shout {
            known_vals_by_name.insert(result.name, val);
        }
        result
    }
}

struct Shouting<'s> {
    shouters_by_name: HashMap<&'s str, Shouter<'s>>,
}
impl<'s> Shouting<'s> {
    fn new(shouters: &[Shouter<'s>]) -> Self {
        let shouters_by_name = shouters.into_iter().map(|s| (s.name, s.clone())).collect();
        Self { shouters_by_name }
    }
}

impl<'s> Shouting<'s> {

    fn get_defined_on(&self, shouter : &'s str) -> Vec<&'s str> {

        let mut defined_on:Vec<&'s str> = Vec::with_capacity(self.shouters_by_name.len());
        let mut already_scanned:HashSet<&str> = HashSet::with_capacity(self.shouters_by_name.len());
        already_scanned.insert(shouter);
        let mut round_inquiry = vec![shouter];
        while !round_inquiry.is_empty() {
            round_inquiry = round_inquiry.iter().flat_map(|s|self.shouters_by_name.values().filter_map(|d|match d.shout{
                Shout::Add((Ref(name),_)) if name == *s => {Some(d.name)}
                Shout::Add((_,Ref(name))) if name == *s => {Some(d.name)}
                Shout::Sub((Ref(name),_)) if name == *s => {Some(d.name)}
                Shout::Sub((_,Ref(name))) if name == *s => {Some(d.name)}
                Shout::Mul((Ref(name),_)) if name == *s => {Some(d.name)}
                Shout::Mul((_,Ref(name))) if name == *s => {Some(d.name)}
                Shout::Div((Ref(name),_)) if name == *s => {Some(d.name)}
                Shout::Div((_,Ref(name))) if name == *s => {Some(d.name)}
                Shout::Val(Ref(name)) if name == *s => {Some(d.name)}
                _ => None,
            })).filter(|d|already_scanned.insert(d))
                .collect();
            for defined in &round_inquiry {
                defined_on.push(*defined);
            }
        }

        defined_on
    }


    fn get_sources(&self, shouter: &str) -> HashSet<&'s str> {
        let shouter = self.shouters_by_name.get(shouter);
        return shouter.map_or_else(HashSet::new, |shouter| {
            let mut sources: HashSet<&'s str> = HashSet::new();

            let mut round_inquiry = vec![shouter];
            while !round_inquiry.is_empty() {
                round_inquiry = round_inquiry
                    .iter()
                    .filter_map(|source| {
                        if !sources.contains(source.name) {
                            sources.insert(source.name);
                            match source.shout {
                                Shout::Add((a, b))
                                | Shout::Sub((a, b))
                                | Shout::Mul((a, b))
                                | Shout::Div((a, b)) => Some(
                                    [a, b]
                                        .iter()
                                        .filter_map(|f| {
                                            f.get_name()
                                                .and_then(|name| self.shouters_by_name.get(name))
                                        })
                                        .collect_vec(),
                                ),
                                Shout::Val(_) => None,
                            }
                        } else {
                            None
                        }
                    })
                    .flat_map(Vec::into_iter)
                    .collect();
            }
            sources
        });
    }
}

fn deref(source: Source, vals: &HashMap<&str, isize>) -> Option<isize> {
    match source {
        Source::Ref(name) => vals.get(name).copied(),
        Source::Val(v) => Some(v),
    }
}


fn resolve<'s>(shouter: &Shouter<'s>, vals: &HashMap<&str, isize>) -> Option<(&'s str, isize)> {
    let name = shouter.name;

    let solve = |a, b, f: fn(isize, isize) -> isize| {
        deref(a, vals).and_then(|a| deref(b, vals).map(|b| (name, f(a, b))))
    };
    match shouter.shout {
        Shout::Val(s) => deref(s, vals).map(|v| (name, v)),
        Shout::Add((a, b)) => solve(a, b, isize::add),
        Shout::Sub((a, b)) => solve(a, b, isize::sub),
        Shout::Mul((a, b)) => solve(a, b, isize::mul),
        Shout::Div((a, b)) => solve(a, b, isize::div),
    }
}

fn reduce_shouts<'s>(shouting: &Shouting<'s>, vals: &mut HashMap<&'s str, isize>, source: &str) {
    let mut sources = shouting.get_sources(source);
    loop {
        let to_remove: Vec<_> = sources
            .iter()
            .filter_map(|source| {
                shouting
                    .shouters_by_name
                    .get(source)
                    .and_then(|shouter| resolve(shouter, &vals))
            })
            .collect();
        if to_remove.is_empty() {
            break;
        }
        for (name, val) in to_remove {
            vals.insert(&name, val);
            sources.remove(&name);
        }
    }
}

fn root_shout<'s>(shouters: &[Shouter<'s>]) -> isize {
    let shouting = Shouting::new(shouters);
    let mut vals: HashMap<&str, isize> = HashMap::with_capacity(shouters.len());

    reduce_shouts(&shouting, &mut vals, "root");
    *vals.get("root").unwrap()
}
fn equality_human_shout<'s>(shouters: &[Shouter<'s>]) -> isize {
    let mut shouting = Shouting::new(shouters);

    // removed non existant rule
    shouting.shouters_by_name.remove("humn");

    let mut vals: HashMap<&str, isize> = HashMap::with_capacity(shouters.len());
    // reduce_shouts(&shouting, &mut vals, "root");
    //
    // shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|(name, shouter.replace_known_vals(&mut vals))).collect();
    let root = shouting.shouters_by_name.remove("root").unwrap();

    let (mut equal_a,mut equal_b)= match root.shout {
        Shout::Add((a,b))|Shout::Sub((a,b)) | Shout::Mul((a,b))|Shout::Div((a,b))=> {(a,b)}
        _ => panic!("root cannot be immediatly defined, would mean any number is a solution")
    };

    if let Val(_v) = equal_a  {
        let new_equal_a = equal_b ;
        equal_b = equal_a;
        equal_a = new_equal_a;
    }
    let replaced_source = if let Ref (replaced_source) = equal_a  {
        replaced_source
    } else {
        panic!("{root} : let's try to handle double Ref for now");
    };
    if let Val(v) = equal_b {
        vals.insert(replaced_source, v);
    }



    shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|{
        (name, if name == replaced_source {shouter} else {
            match shouter.shout {
                Shout::Add((Ref(sname),b)) if sname == replaced_source=> {Shouter { name, shout: Shout::Add((equal_b, b)) } },
                Shout::Add((a,Ref(sname))) if sname == replaced_source=> {Shouter { name, shout: Shout::Add((a, equal_b )) } },
                Shout::Sub((Ref(sname),b)) if sname == replaced_source => {Shouter { name, shout: Shout::Sub((equal_b, b)) }}
                Shout::Sub((a,Ref(sname))) if sname == replaced_source => {Shouter { name, shout: Shout::Sub((a, equal_b )) }}
                Shout::Mul((Ref(sname),b)) if sname == replaced_source => {Shouter { name, shout: Shout::Mul((equal_b, b)) }}
                Shout::Mul((a,Ref(sname))) if sname == replaced_source => {Shouter { name, shout: Shout::Mul((a, equal_b )) }}
                Shout::Div((Ref(sname),b)) if sname == replaced_source => {Shouter { name, shout: Shout::Div((equal_b, b)) }}
                Shout::Div((a,Ref(sname))) if sname == replaced_source => {Shouter { name, shout: Shout::Div((a, equal_b )) }}

                _ => {shouter}
            }
        })
    }).collect();
    reduce_shouts(&shouting, &mut vals, replaced_source);
    shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|(name, shouter.replace_known_vals(&mut vals))).collect();

    // let human_rules: Vec<Shouter> = shouters
    //     .iter()
    //     .filter_map(|s| s.promote_source("humn"))
    //     .collect();
    // println!("{} rules directly reference 'humn'", human_rules.len());
    // shouting.shouters_by_name.insert("humn", human_rules[0]);

    // shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|(name, shouter.replace_known_vals(&mut vals))).collect();
    // println!("\n*** reduce({replaced_source})");
    // for (name,s) in &shouting.shouters_by_name {
    //     println!("{s} = {:?}",vals.get(name));
    // }

    let defined_on_human =shouting.get_defined_on("humn");
    println!("\n*** human sources {defined_on_human:?}");

    let new_rules : Vec<_> = defined_on_human.iter().enumerate().rev().filter_map(|(i, defined_on)|{
        let old = shouting.shouters_by_name.get(defined_on).unwrap();
        println!("     {old}");
        defined_on_human[0..i].iter().rev().chain(once(&"humn")).filter_map(|source|old.promote_source(source))
            .next()
    }).collect();

    for new_rule in  new_rules {
        println!("new rule : {new_rule}");

        if ! vals.contains_key(new_rule.name) {
            shouting.shouters_by_name.insert(new_rule.name, new_rule);
        }
    }
    if let Ref(name_b) = equal_b {
        shouting.shouters_by_name.insert(replaced_source, Shouter{name: replaced_source, shout: Shout::Val(Ref(name_b))});
    }
    shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|(name, shouter.replace_known_vals(&mut vals))).collect();

    reduce_shouts(&shouting, &mut vals, "humn");
    shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|(name, shouter.replace_known_vals(&mut vals))).collect();
    reduce_shouts(&shouting, &mut vals, "humn");
    shouting.shouters_by_name = shouting.shouters_by_name.into_iter().map(|(name, shouter)|(name, shouter.replace_known_vals(&mut vals))).collect();
    println!(
        "total shouts : {}, resolved : {}",
        shouting.shouters_by_name.len(),
        vals.len()
    );


    for (name,s) in shouting.shouters_by_name {
            println!("{s} = {:?}",vals.get(name));
    }

    *vals.get("humn").unwrap()

}

pub fn solve_riddles() {
    let input = include_str!("../resources/day21_shouting.txt");
    let shoutings: Vec<Shouter> = input
        .lines()
        .filter_map(|l| Shouter::try_new(l).ok())
        .collect();

    let root_shout = root_shout(&shoutings);
    println!("root shouts {root_shout}");

    let human_shout = equality_human_shout(&shoutings);
    println!("human equality shout {human_shout}");
}
#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn aoc_example_works() {
        let input = indoc! {"
            root: pppw + sjmn
            dbpl: 5
            cczh: sllz + lgvd
            zczc: 2
            ptdq: humn - dvpt
            dvpt: 3
            lfqf: 4
            humn: 5
            ljgn: 2
            sjmn: drzm * dbpl
            sllz: 4
            pppw: cczh / lfqf
            lgvd: ljgn * ptdq
            drzm: hmdt - zczc
            hmdt: 32
        "};
        let shoutings: Vec<Shouter> = input
            .lines()
            .filter_map(|l| Shouter::try_new(l).ok())
            .collect();
        assert_eq!(152, root_shout(&shoutings));
        assert_eq!(301, equality_human_shout(&shoutings));
    }
}
