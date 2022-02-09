use std::collections::HashMap;

use super::pokemon::*;

pub type Guess = Pokemon;
pub type Answer = Pokemon;
pub type Judge = usize;
pub type Partition = HashMap<Judge,Vec<Answer>>;

pub const ALL_CORRECT: Judge = ((Status::Correct as usize) << 2*0) +
                           ((Status::Correct as usize) << 2*1) +
                           ((Status::Correct as usize) << 2*2) +
                           ((Status::Correct as usize) << 2*3) +
                           ((Status::Correct as usize) << 2*4);

#[derive(Clone,Copy)]
pub enum Status {
    Nowhere = 0,
    Wrong = 1,
    Correct = 2,
}

#[derive(Default)]
pub struct JudgeTable {
    data: Vec<Vec<Judge>>,
    pub all_ans: Vec<Answer>,
    pub all_guess: Vec<Guess>,
}
impl JudgeTable {
    pub fn new(n: usize) -> Self {

        let is_valid_ans: Vec<bool> = (0..n)
        .map(|i| POKEMONS[i].chars().collect::<Vec<char>>().len() == 5)
        .collect();
        let is_valid_guess: Vec<bool> = vec![true; n];

        let all_ans: Vec<Answer> = (0..n)
        .filter(|&i| is_valid_ans[i])
        .collect();
        let all_guess: Vec<Guess> = (0..n)
        .filter(|&i| is_valid_guess[i])
        .collect();

        let judge = |guess: &Answer, ans: &Guess| -> Judge {
            assert!(is_valid_guess[*guess]);
            assert!(is_valid_ans[*ans]);

            let guess: Vec<char> = POKEMONS[*guess].chars().collect();
            let ans: Vec<char> = POKEMONS[*ans].chars().collect();

            let (mut ret, mut guess_used, mut ans_used) = (0, 0, 0);
            for i in 0..guess.len() {
                if guess[i] == ans[i] {
                    ret |= (Status::Correct as usize) << 2*i;
                    guess_used |= 1 << i;
                    ans_used |= 1 << i;
                }
            }

            for i in 0..guess.len() {
                if (guess_used >> i & 1) > 0 {
                    continue;
                }
                for j in 0..5 {
                    if (ans_used >> j & 1) > 0 {
                        continue;
                    }
                    if guess[i] == ans[j] {
                        ret |= (Status::Wrong as usize) << 2*i;
                        guess_used |= 1 << i;
                        ans_used |= 1 << j;
                        break;
                    }
                }
            }
            ret
        };


        let data = (0..n).map(|ans| {
            if is_valid_ans[ans] {
                (0..n).map(|guess| {
                    judge(&guess, &ans)
                }).collect()
            } else {
                vec![]
            }
        }).collect();

        Self { data, all_guess, all_ans }
    }

    pub fn judge(&self, guess: &Guess, ans: &Answer) -> Judge {
        return self.data[*ans][*guess];
    } 

    pub fn partition(&self, ans_rem: &Vec<Answer>, guess: &Guess) -> Partition {
        let mut ret: Partition = HashMap::new();
        for ans in ans_rem.iter() {
            let judge = self.judge(guess, ans);
            if judge == ALL_CORRECT {
                continue;
            }
            ret.entry(judge).or_insert(Vec::new()).push(*ans);
        }
        ret
    }
}
