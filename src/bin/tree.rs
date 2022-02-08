use std::collections::HashMap;
use std::rc::Rc;
use std::fs;
use std::io::Write;
use argh::FromArgs;

use wordle_pokemon::{pokemon::*, judge::*};

#[derive(Default)]
struct Node {
    guess: usize,
    rem: Vec<Pokemon>,
    edges: HashMap<Judge,Rc<Node>>,
}

#[derive(Default)]
struct DecisionTree {
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

    pub fn build(&self, rem: &Vec<Pokemon>, depth: usize) -> Rc<Node> {
        if rem.len() == 1 {
            assert!(self.guess_seq[rem[0]].len() == depth + 1);
            assert!(self.guess_seq[rem[0]][depth] == rem[0]);
            return Rc::new(Node{ guess: rem[0], rem: rem.clone(), ..Default::default() });
        }

        let guess = self.guess_seq[rem[0]][depth];
        for i in 1..rem.len() {
            assert!(self.guess_seq[rem[i]].len() > depth);
            assert!(guess == self.guess_seq[rem[i]][depth]);
        }

        let edges: HashMap<Judge,Rc<Node>> = self.judge_table.partition(rem, &guess).iter().map(|(judge, s)| {
            (*judge, self.build(s, depth + 1))
        }).collect();

        return Rc::new(Node{ guess, edges, rem: rem.clone() });
    }


    pub fn next(node: &Rc<Node>, history: &Vec<Judge>) -> Rc<Node> {
        // 今まで正しくguessしてきたことを要求

        let mut v = Rc::clone(node);
        for judge in history {
            v = Rc::clone(&v.edges[judge]);
        }

        v
    }
}


#[derive(FromArgs)]
/// Build decision tree
struct Args {
    /// the number of pokemons
    #[argh(option, short='n')]
    num_pokemons: usize,
}

fn main() {
    let args: Args = argh::from_env();
    let n = args.num_pokemons;

    let filepath = format!("tree_n={}.txt", n);

    let root = DecisionTree::new(&filepath).build(&(0..n).collect(), 0);

    let mut history = vec![];

    let nxt = DecisionTree::next(&root, &history);
    println!("(残り{}匹) {}", nxt.rem.len(), POKEMONS[nxt.guess].iter().collect::<String>());

    while true {
        print!("-> ");
        std::io::stdout().flush().unwrap();
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).unwrap();
        let s = s.trim().to_string();
        if s.len() != 5 {
            println!("s.len(): {}", s.len());
        }

        assert!(s.len() == 5);
        history.push({
            let judge = s.chars().enumerate().map(|(i, c)| {
                c.to_digit(10).unwrap() << 2*i
            }).sum::<u32>() as Judge;
            if judge == ALL_CORRECT {
                println!("Congratulations!!!");
                break;
            }
            judge
        });

        let nxt = DecisionTree::next(&root, &history);
        println!("(残り{}匹) {}", nxt.rem.len(), POKEMONS[nxt.guess].iter().collect::<String>());
    }
}