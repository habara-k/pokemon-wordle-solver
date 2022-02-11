use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::rc::Rc;

use super::{judge::*, pokemon::*};

pub enum Node {
    Terminal,
    NonTerminal {
        guess: usize,
        rem_ans: Vec<Answer>,
        edges: BTreeMap<Judge, Rc<Node>>,
    },
}

impl Node {
    pub fn write(&self, out: &mut fs::File) {
        match self {
            Node::NonTerminal {
                guess,
                edges,
                rem_ans,
            } => {
                out.write_all(
                    format!(
                        "{{\"guess\":\"{}\",\"rem\":{},\"edges\":{{",
                        POKEMONS[*guess],
                        rem_ans.len(),
                    )
                    .as_bytes(),
                )
                .unwrap();
                for (i, (judge, ch)) in edges.iter().enumerate() {
                    let judge = (0..5)
                        .map(|i| (judge >> 2 * i & 0b11).to_string())
                        .collect::<Vec<String>>()
                        .join("");
                    out.write_all(format!("\"{}\":", judge).as_bytes()).unwrap();
                    ch.write(out);
                    if i + 1 < edges.len() {
                        out.write_all(",".as_bytes()).unwrap();
                    }
                }
                out.write_all("}}".as_bytes()).unwrap();
            }
            Node::Terminal => {
                out.write_all("{}".as_bytes()).unwrap();
            }
        }
    }
    pub fn next(&self, judge: &Judge) -> Rc<Node> {
        if let Node::NonTerminal { edges, .. } = self {
            return Rc::clone(&edges[judge]);
        }
        panic!("Incorrect judge.");
    }
}

#[derive(Default)]
pub struct DecisionTree {
    pub guess_seq: Vec<Vec<Guess>>,
    pub judge_table: JudgeTable,
}

impl DecisionTree {
    pub fn new(filepath: &str) -> Self {
        let guess_seq: Vec<Vec<Guess>> = fs::read_to_string(filepath)
            .unwrap()
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .map(|s| s.parse::<Guess>().unwrap())
                    .collect()
            })
            .collect();

        let ans_until = guess_seq.len();
        let guess_until = *guess_seq
            .iter()
            .map(|seq| seq.iter().max().unwrap_or(&0))
            .max()
            .unwrap()
            + 1;

        Self {
            guess_seq,
            judge_table: JudgeTable::new(ans_until, guess_until),
        }
    }

    pub fn build(&self, rem_ans: &Vec<Answer>, depth: usize) -> Rc<Node> {
        assert!(rem_ans.len() > 0);

        let guess = self.guess_seq[rem_ans[0]][depth];
        for i in 1..rem_ans.len() {
            assert!(self.guess_seq[rem_ans[i]].len() > depth);
            assert!(guess == self.guess_seq[rem_ans[i]][depth]);
        }

        let mut edges: BTreeMap<Judge, Rc<Node>> = self
            .judge_table
            .partition(rem_ans, &guess)
            .iter()
            .map(|(judge, s)| (*judge, self.build(s, depth + 1)))
            .collect();

        if rem_ans.contains(&guess) {
            edges.insert(ALL_CORRECT, Rc::new(Node::Terminal));
        }

        return Rc::new(Node::NonTerminal {
            guess,
            edges,
            rem_ans: rem_ans.clone(),
        });
    }
}
