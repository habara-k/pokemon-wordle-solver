use wordle_pokemon::consts::*;
use std::collections::BTreeMap;
use std::time::Instant;
use ord_subset::{OrdSubsetIterExt,OrdSubsetSliceExt};

#[derive(Clone,Copy)]
enum Status {
    Nowhere = 0,
    Wrong = 1,
    Correct = 2,
}

const ALL_CORRECT: usize = ((Status::Correct as usize) << 2*0) +
                           ((Status::Correct as usize) << 2*1) +
                           ((Status::Correct as usize) << 2*2) +
                           ((Status::Correct as usize) << 2*3) +
                           ((Status::Correct as usize) << 2*4);

const INFTY: i32 = i32::MAX / 2;

use argh::FromArgs;

#[derive(FromArgs)]
/// arguments
struct Args {
    /// the number of pokemons
    #[argh(option)]
    n: usize,
    /// the limit depth of lower_bound dfs
    #[argh(option)]
    lb_depth_limit: usize,
}


#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
struct SetId(usize);


#[derive(Default)]
struct Solver {
    n: usize,
    lb_depth_limit: usize,

    judge_table: Vec<Vec<usize>>,
    memo: BTreeMap<SetId, (i32, usize, BTreeMap<usize, Vec<usize>>)>,
    best: BTreeMap<SetId, i32>,
    lb_memo: BTreeMap<SetId, (usize, i32)>,

    set_id: BTreeMap<Vec<usize>,SetId>,
    cnt: usize,
}

impl Solver {
    pub fn new() -> Self {
        let args: Args = argh::from_env();

        let n = args.n;
        let lb_depth_limit = args.lb_depth_limit;

        let judge_table = (0..args.n).map(|guess| {
            (0..args.n).map(|ans| {
                Self::judge(guess, ans)
            }).collect()
        }).collect();

        Self { n, judge_table, lb_depth_limit, ..Default::default() }
    }

    fn get_set_id(&mut self, st: &Vec<usize>) -> SetId {
        if let Some(id) = self.set_id.get(st) {
            return *id;
        }
        let id = SetId(self.cnt);
        self.set_id.insert(st.clone(), id);
        self.cnt += 1;
        return id;
    }

    fn judge(guess: usize, ans: usize) -> usize {
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
    }

    fn partition(&self, rem: &Vec<usize>, guess: &usize) -> BTreeMap<usize,Vec<usize>> {
        let mut ret: BTreeMap<usize,Vec<usize>> = BTreeMap::new();
        for ans in rem.iter() {
            let judge = self.judge_table[*guess][*ans];
            if judge == ALL_CORRECT {
                continue;
            }
            ret.entry(judge).or_insert(Vec::new()).push(*ans);
        }
        ret
    }

    pub fn build_good_solution(&mut self) {
        let all = (0..self.n).collect();
        println!("期待回数(貪欲): {:?}", self.dfs_good_solution(&all) as f64 / self.n as f64);
        println!("{:?}", self.dfs_good_solution(&all));
    }

    fn dfs_good_solution(&mut self, rem: &Vec<usize>) -> i32 {
        assert!(rem.len() > 0);
        if rem.len() == 1 {
            return 1;
        }

        let rem_id = self.get_set_id(rem);

        if let Some((val, ..)) = self.memo.get(&rem_id) {
            return *val;
        }

        let partitions: Vec<BTreeMap<usize,Vec<usize>>> = (0..self.n).map(|guess| {
            self.partition(rem, &guess)
        }).collect();

        let good_guess = (0..self.n).ord_subset_min_by_key(|guess| -> f64 {
            // maximize entropy
            // n=511: 3.85127
            //partitions[*guess].values().map(|s| (s.len() as f64).log2() * s.len() as f64).sum()

            // minimize maximum size
            // n=511: 3.92944
            //partitions[*guess].values().map(|s| s.len()).max().unwrap() as f64

            // minimize average size
            // n=511: 3.70646
            //partitions[*guess].values().map(|s| s.len() as f64 * s.len() as f64).sum()

            // minimize average size, mazimize entropy
            // n=511: 3.69080
            partitions[*guess].values().map(|s| (s.len() as f64 + (s.len() as f64).log2()) * s.len() as f64).sum()
        }).unwrap();

        let val: i32 = rem.len() as i32 + partitions[good_guess].values().map(|s| self.dfs_good_solution(s)).sum::<i32>();

        self.memo.insert(rem_id, (val, good_guess, partitions[good_guess].clone()));

        val
    }

    fn lower_bound(&mut self, rem: &Vec<usize>, depth: usize) -> i32 {
        assert!(rem.len() > 0);
        if depth == 0 || rem.len() <= 2 {
            return 2 * rem.len() as i32 - 1;
        }

        let rem_id = self.get_set_id(rem);

        if let Some((d, lb)) = self.lb_memo.get(&rem_id) {
            if *d >= depth {
                return *lb
            }
        }

        let partitions: Vec<BTreeMap<usize,Vec<usize>>> = (0..self.n).map(|guess| {
            self.partition(rem, &guess)
        }).collect();

        let ret: i32 = rem.len() as i32 + partitions.iter().map(|part| {
            part.values().map(|s| {
                self.lower_bound(s, depth-1)
            }).sum::<i32>()
        }).ord_subset_min().unwrap();

        assert!(ret >= 2 * rem.len() as i32 - 1);

        self.lb_memo.insert(rem_id, (depth, ret));
        ret
    }

    pub fn build_best_solution(&mut self) {
        let all = (0..self.n).collect();
        println!("期待回数(最適): {:?}", self.dfs_best_solution(&all, INFTY) as f64 / self.n as f64);
        println!("{:?}", self.dfs_best_solution(&all, INFTY));
    }

    fn dfs_best_solution(&mut self, rem: &Vec<usize>, ub: i32) -> i32 {
        assert!(rem.len() > 0);
        if rem.len() == 1 {
            return 1;
        }

        let rem_id = self.get_set_id(rem);

        if let Some(val) = self.best.get(&rem_id) {
            return *val;
        }

        if self.lower_bound(rem, self.lb_depth_limit) >= ub {
            return INFTY;
        }

        let mut val = self.dfs_good_solution(rem);

        let partitions: Vec<BTreeMap<usize,Vec<usize>>> = (0..self.n).map(|guess| {
            self.partition(rem, &guess)
        }).collect();

        let penalty: Vec<f64> = partitions.iter().map(|part| {
            // minimize "average size - entropy"
            //part.values().map(|s| (s.len() as f64 + (s.len() as f64).log2()) * s.len() as f64).sum::<f64>()
            // maximize "entropy"
            part.values().map(|s| (s.len() as f64).log2() * s.len() as f64).sum::<f64>()
        }).collect();

        let mut order: Vec<usize> = (0..self.n).collect();
        order.ord_subset_sort_by_key(|i| penalty[*i]);

    
        for guess in order.iter() {
            let part = &partitions[*guess];

            let lb: i32 = rem.len() as i32 + part.values().map(|s| {
                self.lower_bound(s, self.lb_depth_limit)
            }).sum::<i32>();

            if lb >= val {
                continue
            }

            let mut tmp: i32 = rem.len() as i32;
            for s in part.values() {
                tmp += self.dfs_best_solution(s, val - tmp);
                if tmp >= val {
                    break;
                }
            }

            if tmp < val {
                val = tmp;
                self.memo.insert(rem_id.clone(), (val, *guess, part.clone()));
            }
        }

        self.best.insert(rem_id, val);

        val
    }
}


fn main() {
    let start = Instant::now();

    let mut solver = Solver::new();
    solver.build_best_solution();
    //solver.build_good_solution();
    println!("best.len(): {:?}", solver.best.len());
    println!("memo.len(): {:?}", solver.memo.len());
    println!("lb_memo.len(): {:?}", solver.lb_memo.len());

    println!(
        "elapsed time: {:?} [sec]",
        start.elapsed().as_nanos() as f64 / 1_000_000_000 as f64
    );
}
