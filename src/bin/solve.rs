use std::collections::BTreeMap;
use std::time::Instant;
use std::fs;
use std::io::Write;
use std::sync::{Mutex,Arc};
use ord_subset::{OrdSubsetIterExt,OrdSubsetSliceExt};
use rayon::prelude::*;
use argh::FromArgs;

use wordle_pokemon::{pokemon::*, judge::*};

#[derive(FromArgs)]
/// Minimize expectation of the number of guess
struct Args {
    /// the number of pokemons
    #[argh(option, short='n')]
    num_pokemons: usize,
    /// the limit depth of lower_bound dfs
    #[argh(option, default="default_lb_depth_limit()")]
    lb_depth_limit: usize,
    /// the number of threads
    #[argh(option, short='t', default="default_num_threads()")]
    num_threads: usize,
}
fn default_lb_depth_limit() -> usize { 1 }
fn default_num_threads() -> usize { 1 }

type SetId = usize;
type Score = i32;

const INFTY: Score = Score::MAX / 2;

#[derive(Default)]
struct Cache {
    memo: BTreeMap<SetId, (Score, Guess, BTreeMap<Judge, Vec<Pokemon>>)>,
    best: BTreeMap<SetId, Score>,
    lb_memo: BTreeMap<SetId, (usize, Score)>,

    set_id: BTreeMap<Vec<Pokemon>,SetId>,
    cnt: usize,
}
impl Cache {
    pub fn get_set_id(&mut self, st: &Vec<Pokemon>) -> SetId {
        if let Some(id) = self.set_id.get(st) {
            return *id;
        }
        let id = self.cnt;
        self.cnt += 1;
        self.set_id.insert(st.clone(), id);

        return id;
    }
}


#[derive(Default)]
struct Solver {
    n: usize,
    all: Vec<Pokemon>,
    lb_depth_limit: usize,

    judge_table: JudgeTable,
    //cache: Cache,
    cache: Arc<Mutex<Cache>>,
}

impl Solver {
    pub fn new() -> Self {
        let args: Args = argh::from_env();

        let n = args.num_pokemons;
        let all = (0..n).collect::<Vec<Pokemon>>();
        let lb_depth_limit = args.lb_depth_limit;

        let judge_table = JudgeTable::new(n);

        Self { n, all, lb_depth_limit, judge_table, ..Default::default() }
    }

    pub fn build_good_solution(&self) {
        let all = (0..self.n).collect();
        println!("期待回数(貪欲): {} = {}/{}",
            self.dfs_good_solution(&all) as f32 / self.n as f32,
            self.dfs_good_solution(&all), self.n
        );
    }

    fn dfs_good_solution(&self, rem: &Vec<Pokemon>) -> Score {
        assert!(rem.len() > 0);
        if rem.len() == 1 {
            return 1;
        }

        let rem_id = self.cache.lock().unwrap().get_set_id(rem);

        if let Some((val, ..)) = self.cache.lock().unwrap().memo.get(&rem_id) {
            return *val;
        }

        // parallel
        let partitions: Vec<Partition> = self.all.par_iter().map(|guess| {
            self.judge_table.partition(rem, &guess)
        }).collect();

        let good_guess: Guess = *self.all.iter().ord_subset_min_by_key(|&guess| -> f32 {
            // minimize average size, maximize entropy
            // n=511: 3.68688
            partitions[*guess].values().map(|s| {
                let x = s.len() as f32;
                (0.1*x + x.log2()) * x
            }).sum()
        }).unwrap();

        // parallel
        let val: Score = rem.len() as Score + partitions[good_guess].par_iter().map(
            |(_, s)| self.dfs_good_solution(s)).sum::<Score>();

        self.cache.lock().unwrap().memo.insert(rem_id, (val, good_guess, partitions[good_guess].clone()));

        val
    }

    fn lower_bound(&self, rem: &Vec<Pokemon>, depth: usize) -> Score {
        assert!(rem.len() > 0);
        if depth == 0 || rem.len() <= 2 {
            return 2 * rem.len() as Score - 1;
        }

        let rem_id = self.cache.lock().unwrap().get_set_id(rem);

        if let Some((d, lb)) = self.cache.lock().unwrap().lb_memo.get(&rem_id) {
            if *d >= depth {
                return *lb
            }
        }

        // parallel
        let partitions: Vec<Partition> = self.all.par_iter().map(|guess| {
            self.judge_table.partition(rem, &guess)
        }).collect();

        // parallel
        let ret: Score = rem.len() as Score + partitions.par_iter().map(|part| {
            part.values().map(|s| {
                self.lower_bound(s, depth-1)
            }).sum::<Score>()
        }).min().unwrap();

        // assert!(ret >= 2 * rem.len() as Score - 1);

        self.cache.lock().unwrap().lb_memo.insert(rem_id, (depth, ret));
        ret
    }

    pub fn build_best_solution(&self) {
        let all = (0..self.n).collect();
        println!("期待回数(最適): {} = {}/{}",
            self.dfs_best_solution(&all, INFTY) as f32 / self.n as f32,
            self.dfs_best_solution(&all, INFTY), self.n
        );
    }

    fn dfs_best_solution(&self, rem: &Vec<Pokemon>, ub: Score) -> Score {
        assert!(rem.len() > 0);
        if rem.len() == 1 {
            return 1;
        }

        let rem_id = self.cache.lock().unwrap().get_set_id(rem);

        if let Some(val) = self.cache.lock().unwrap().best.get(&rem_id) {
            return *val;
        }

        if self.lower_bound(rem, self.lb_depth_limit) >= ub {
            return INFTY;
        }

        let mut val = self.dfs_good_solution(rem);

        // parallel
        let partitions: Vec<Partition> = self.all.par_iter().map(|guess| {
            self.judge_table.partition(rem, &guess)
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

            let lb: Score = rem.len() as Score + part.values().map(|s| {
                self.lower_bound(s, self.lb_depth_limit)
            }).sum::<Score>();

            if lb >= val {
                continue
            }

            let mut tmp = rem.len() as Score;
            for s in part.values() {
                tmp += self.dfs_best_solution(s, val - tmp);
                if tmp >= val {
                    break;
                }
            }

            if tmp < val {
                val = tmp;
                self.cache.lock().unwrap().memo.insert(rem_id, (val, *guess, part.clone()));
            }
        }

        self.cache.lock().unwrap().best.insert(rem_id, val);

        val
    }

    pub fn write(&self) {
        let mut guess_seq: Vec<Vec<Guess>> = (0..self.n).map(|_| Vec::new()).collect();
        self.dfs_build_guess_seq(&mut guess_seq, &self.all);

        let mut f = fs::File::create(format!("tree_n={}.txt", self.n)).unwrap();
        for guess in &guess_seq {
            f.write_all(format!("{}\n", guess.iter().map(|g| g.to_string()).collect::<Vec<String>>().join(" ")).as_bytes()).unwrap();
        }
    }

    fn dfs_build_guess_seq(&self,  guess_seq: &mut Vec<Vec<Guess>>, rem: &Vec<Pokemon>) {
        if rem.len() == 1 {
            guess_seq[rem[0]].push(rem[0]);
            return;
        }

        let (guess, part) = {
            let cache = self.cache.lock().unwrap();
            let rem_id = *cache.set_id.get(rem).unwrap();
            let (_, guess, part) = cache.memo[&rem_id].clone();
            (guess, part)
        };

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

    let solver = Solver::new();

    //solver.build_best_solution();
    //solver.build_good_solution();

    let args: Args = argh::from_env();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(args.num_threads).build().unwrap();
    pool.install(|| solver.build_best_solution());

    //let args: Args = argh::from_env();
    //rayon::ThreadPoolBuilder::new().num_threads(args.num_threads).build_global().unwrap();
    //solver.build_best_solution();

    println!("best.len(): {:?}", solver.cache.lock().unwrap().best.len());
    println!("memo.len(): {:?}", solver.cache.lock().unwrap().memo.len());
    println!("lb_memo.len(): {:?}", solver.cache.lock().unwrap().lb_memo.len());

    solver.write();

    println!(
        "elapsed time: {:?} [sec]",
        start.elapsed().as_nanos() as f32 / 1_000_000_000 as f32
    );
}
