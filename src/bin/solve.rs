use wordle_pokemon::consts::*;
use std::collections::BTreeMap;
use std::time::Instant;
use std::fs;
use std::io::Write;
use ord_subset::{OrdSubsetIterExt,OrdSubsetSliceExt};
use rayon::prelude::*;

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
    #[argh(option, short='n')]
    num_pokemons: usize,
    /// the limit depth of lower_bound dfs
    #[argh(option, default="default_lb_depth_limit()")]
    lb_depth_limit: usize,
}
fn default_lb_depth_limit() -> usize { 1 }


#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
struct SetId(usize);

#[derive(Default)]
struct Cache {
    memo: BTreeMap<SetId, (i32, usize, BTreeMap<usize, Vec<usize>>)>,
    best: BTreeMap<SetId, i32>,
    lb_memo: BTreeMap<SetId, (usize, i32)>,

    set_id: BTreeMap<Vec<usize>,SetId>,
    cnt: usize,
}

#[derive(Default)]
struct Solver {
    n: usize,
    all: Vec<usize>,
    lb_depth_limit: usize,

    judge_table: Vec<Vec<usize>>,
    //cache: Arc<Mutex<Cache>>,
    cache: Cache,
}

impl Solver {
    pub fn new() -> Self {
        let args: Args = argh::from_env();

        let n = args.num_pokemons;
        let all = (0..n).collect::<Vec<usize>>();
        let lb_depth_limit = args.lb_depth_limit;

        let judge_table = all.iter().map(|guess| {
            all.iter().map(|ans| {
                Self::judge(guess, ans)
            }).collect()
        }).collect();

        Self { n, all, lb_depth_limit, judge_table, ..Default::default() }
    }

    fn judge(guess: &usize, ans: &usize) -> usize {
        let (mut ret, mut guess_used, mut ans_used) = (0, 0, 0);
        for i in 0..5 {
            if POKEMONS[*guess][i] == POKEMONS[*ans][i] {
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
                if POKEMONS[*guess][i] == POKEMONS[*ans][j] {
                    ret |= (Status::Wrong as usize) << 2*i;
                    guess_used |= 1 << i;
                    ans_used |= 1 << j;
                }
            }
        }
        ret
    }

    fn get_set_id(&mut self, st: &Vec<usize>) -> SetId {
        //if let Some(id) = self.cache.lock().unwrap().set_id.get(st) {
        //    return *id;
        //}
        //let mut cache = self.cache.lock().unwrap();
        //let id = SetId(cache.cnt);
        //cache.cnt += 1;
        //cache.set_id.insert(st.clone(), id);

        //return id;
        if let Some(id) = self.cache.set_id.get(st) {
            return *id;
        }
        let id = SetId(self.cache.cnt);
        self.cache.cnt += 1;
        self.cache.set_id.insert(st.clone(), id);

        return id;
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
        println!("期待回数(貪欲): {:?}", self.dfs_good_solution(&all) as f32 / self.n as f32);
        println!("{:?}", self.dfs_good_solution(&all));
    }

    fn dfs_good_solution(&mut self, rem: &Vec<usize>) -> i32 {
        assert!(rem.len() > 0);
        if rem.len() == 1 {
            return 1;
        }

        let rem_id = self.get_set_id(rem);

        if let Some((val, ..)) = self.cache.memo.get(&rem_id) {
            return *val;
        }

        // parallel
        let partitions: Vec<BTreeMap<usize,Vec<usize>>> = self.all.par_iter().map(|guess| {
            self.partition(rem, &guess)
        }).collect::<Vec<_>>();

        let good_guess = *self.all.iter().ord_subset_min_by_key(|&guess| -> f32 {
            // minimize average size, maximize entropy
            // n=511: 3.68688
            partitions[*guess].values().map(|s| {
                let x = s.len() as f32;
                (0.1*x + x.log2()) * x
            }).sum()
        }).unwrap();

        let val = rem.len() as i32 + partitions[good_guess].iter().map(
            |(_, s)| self.dfs_good_solution(s)).sum::<i32>();

        self.cache.memo.insert(rem_id, (val, good_guess, partitions[good_guess].clone()));

        val
    }

    fn lower_bound(&mut self, rem: &Vec<usize>, depth: usize) -> i32 {
        assert!(rem.len() > 0);
        if depth == 0 || rem.len() <= 2 {
            return 2 * rem.len() as i32 - 1;
        }

        let rem_id = self.get_set_id(rem);

        if let Some((d, lb)) = self.cache.lb_memo.get(&rem_id) {
            if *d >= depth {
                return *lb
            }
        }

        // parallel
        let partitions: Vec<BTreeMap<usize,Vec<usize>>> = self.all.par_iter().map(|guess| {
            self.partition(rem, &guess)
        }).collect::<Vec<_>>();

        let ret: i32 = rem.len() as i32 + partitions.iter().map(|part| {
            part.values().map(|s| {
                self.lower_bound(s, depth-1)
            }).sum::<i32>()
        }).min().unwrap();

        assert!(ret >= 2 * rem.len() as i32 - 1);

        self.cache.lb_memo.insert(rem_id, (depth, ret));
        ret
    }

    pub fn build_best_solution(&mut self) {
        let all = (0..self.n).collect();
        println!("期待回数(最適): {:?}", self.dfs_best_solution(&all, INFTY) as f32 / self.n as f32);
        println!("{:?}", self.dfs_best_solution(&all, INFTY));
    }

    fn dfs_best_solution(&mut self, rem: &Vec<usize>, ub: i32) -> i32 {
        assert!(rem.len() > 0);
        if rem.len() == 1 {
            return 1;
        }

        let rem_id = self.get_set_id(rem);

        if let Some(val) = self.cache.best.get(&rem_id) {
            return *val;
        }

        if self.lower_bound(rem, self.lb_depth_limit) >= ub {
            return INFTY;
        }

        let mut val = self.dfs_good_solution(rem);

        // parallel
        let partitions: Vec<BTreeMap<usize,Vec<usize>>> = self.all.par_iter().map(|guess| {
            self.partition(rem, &guess)
        }).collect();

        let penalty: Vec<f32> = partitions.iter().map(|part| {
            // maximize "entropy"
            part.values().map(|s| {
                let x = s.len() as f32;
                x.log2() * x
            }).sum::<f32>()
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
                self.cache.memo.insert(rem_id, (val, *guess, part.clone()));
            }
        }

        self.cache.best.insert(rem_id, val);

        val
    }

    pub fn write(&self) {
        let mut guess_seq: Vec<Vec<usize>> = (0..self.n).map(|_| Vec::new()).collect();
        self.dfs_build_guess_seq(&mut guess_seq, &self.all);

        let mut f = fs::File::create(format!("tree_n={}.txt", self.n)).unwrap();
        for guess in &guess_seq {
            f.write_all(format!("{}\n", guess.iter().map(|g| g.to_string()).collect::<Vec<String>>().join(" ")).as_bytes()).unwrap();
        }
    }

    fn dfs_build_guess_seq(&self,  guess_seq: &mut Vec<Vec<usize>>, rem: &Vec<usize>) {
        if rem.len() == 1 {
            guess_seq[rem[0]].push(rem[0]);
            return;
        }

        let rem_id = self.cache.set_id.get(rem).unwrap();
        let (_, guess, part) = &self.cache.memo[&rem_id];

        for ans in rem {
            guess_seq[*ans].push(guess.clone());
        }
        for s in part.values() {
            self.dfs_build_guess_seq(guess_seq, s);
        }
    }
}


fn main() {
    let start = Instant::now();

    let mut solver = Solver::new();

    solver.build_best_solution();
    //solver.build_good_solution();

    println!("best.len(): {:?}", solver.cache.best.len());
    println!("memo.len(): {:?}", solver.cache.memo.len());
    println!("lb_memo.len(): {:?}", solver.cache.lb_memo.len());

    solver.write();

    println!(
        "elapsed time: {:?} [sec]",
        start.elapsed().as_nanos() as f32 / 1_000_000_000 as f32
    );
}
