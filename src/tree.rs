use std::collections::HashMap;
use std::rc::Rc;
use std::fs;
use std::io::Write;

use super::{pokemon::*, judge::*};

#[derive(Default)]
pub struct Node {
    pub guess: usize,
    pub rem_ans: Vec<Answer>,
    pub edges: HashMap<Judge,Rc<Node>>,
}
impl Node {
    pub fn write(&self, out: &mut fs::File) {
        if self.edges.len() == 0 {
            out.write_all(format!("{{\"guess\":\"{}\"}}", POKEMONS[self.guess]).as_bytes()).unwrap();
            return;
        }
        out.write_all(format!("{{\"guess\":\"{}\",\"edges\":{{", POKEMONS[self.guess]).as_bytes()).unwrap();
        for (i, (judge, ch)) in self.edges.iter().enumerate() {
            let judge = (0..5).map(|i| (judge >> 2*i & 0b11).to_string()).collect::<Vec<String>>().join("");
            out.write_all(format!("\"{}\":", judge).as_bytes()).unwrap();
            ch.write(out);
            if i+1 < self.edges.len() {
                out.write_all(",".as_bytes()).unwrap();
            }
        }
        out.write_all("}}".as_bytes()).unwrap();
    }
}

#[derive(Default)]
pub struct DecisionTree {
    guess_seq: Vec<Vec<Guess>>,
    judge_table: JudgeTable,
}

impl DecisionTree {

    pub fn new(filepath: &str) -> Self {
        let guess_seq: Vec<Vec<Guess>> = fs::read_to_string(filepath).unwrap().lines().map(|line| {
            line.split_whitespace().map(|s| s.parse::<Guess>().unwrap()).collect()
        }).collect();

        let n = guess_seq.len();

        Self { guess_seq, judge_table: JudgeTable::new(n) }
    }

    pub fn build(&self, rem_ans: &Vec<Answer>, depth: usize) -> Rc<Node> {
        if rem_ans.len() == 1 {
            assert!(self.guess_seq[rem_ans[0]].len() == depth + 1);
            assert!(self.guess_seq[rem_ans[0]][depth] == rem_ans[0]);
            return Rc::new(Node{ guess: rem_ans[0], rem_ans: rem_ans.clone(), ..Default::default() });
        }

        let guess = self.guess_seq[rem_ans[0]][depth];
        for i in 1..rem_ans.len() {
            assert!(self.guess_seq[rem_ans[i]].len() > depth);
            assert!(guess == self.guess_seq[rem_ans[i]][depth]);
        }

        let edges: HashMap<Judge,Rc<Node>> = self.judge_table.partition(rem_ans, &guess).iter().map(|(judge, s)| {
            (*judge, self.build(s, depth + 1))
        }).collect();

        return Rc::new(Node{ guess, edges, rem_ans: rem_ans.clone() });
    }


    pub fn next(node: &Rc<Node>, history: &Vec<Judge>) -> Rc<Node> {
        let mut v = Rc::clone(node);
        for judge in history {
            v = Rc::clone(&v.edges[judge]);
        }
        v
    }
}
