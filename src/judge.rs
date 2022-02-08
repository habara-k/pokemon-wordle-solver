use std::collections::HashMap;

use super::pokemon::*;

pub type Guess = Pokemon;
pub type Answer = Pokemon;
pub type Judge = usize;
pub type Partition = HashMap<Judge,Vec<Pokemon>>;

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
}
impl JudgeTable {
    pub fn new(n: usize) -> Self {
        let judge = |guess: Guess, ans: Answer| -> Judge {
            let (mut ret, mut guess_used, mut ans_used) = (0, 0, 0);
            for i in 0..5 {
                if POKEMONS[guess][i] == POKEMONS[ans][i] {
                    ret |= (Status::Correct as usize) << 2*i;
                    guess_used |= 1 << i;
                    ans_used |= 1 << i;
                }
            }

            for i in 0..5 {
                if (guess_used >> i & 1) > 0 {
                    continue;
                }
                for j in 0..5 {
                    if (ans_used >> j & 1) > 0 {
                        continue;
                    }
                    if POKEMONS[guess][i] == POKEMONS[ans][j] {
                        ret |= (Status::Wrong as usize) << 2*i;
                        guess_used |= 1 << i;
                        ans_used |= 1 << j;
                    }
                }
            }
            ret
        };
        let data = (0..n).map(|guess| {
            (0..n).map(|ans| {
                judge(guess, ans)
            }).collect()
        }).collect();

        Self { data }
    }

    pub fn judge(&self, guess: &Guess, ans: &Answer) -> Judge {
        return self.data[*guess][*ans];
    } 

    pub fn partition(&self, rem: &Vec<Pokemon>, guess: &Guess) -> Partition {
        let mut ret: Partition = HashMap::new();
        for ans in rem.iter() {
            let judge = self.judge(guess, ans);
            if judge == ALL_CORRECT {
                continue;
            }
            ret.entry(judge).or_insert(Vec::new()).push(*ans);
        }
        ret
    }
}
