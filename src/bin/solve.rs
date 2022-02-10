use argh::FromArgs;
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
//use pprof::protos::Message;

use wordle_pokemon::{judge::*, pokemon::*};

type SetId = usize;
type Score = i32;

const INFTY: Score = Score::MAX / 2;

#[derive(Default)]
struct Cache {
    memo: HashMap<SetId, (Score, Guess, Partition)>,
    best: HashMap<SetId, Score>,
    lb_memo: HashMap<SetId, (usize, Score)>,

    set_id: HashMap<Vec<Answer>, SetId>,
    cnt: usize,
}
impl Cache {
    pub fn get_set_id(&mut self, st: &Vec<Answer>) -> SetId {
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
    ans_until: usize,
    guess_until: usize,

    pokemons: PokemonList,
    judge_table: JudgeTable,

    cache: Arc<Mutex<Cache>>,
}

impl Solver {
    const LB_DEPTH_LIMIT: usize = 1;
    pub fn new(ans_until: usize, guess_until: usize) -> Self {
        let pokemons = PokemonList::new(ans_until, guess_until);
        let judge_table = JudgeTable::new(ans_until, guess_until);

        Self {
            ans_until,
            guess_until,
            pokemons,
            judge_table,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn build_good_solution(&self) {
        println!(
            "期待回数(貪欲): {} = {}/{}",
            self.dfs_good_solution(&self.pokemons.all_ans) as f32
                / self.pokemons.all_ans.len() as f32,
            self.dfs_good_solution(&self.pokemons.all_ans),
            self.pokemons.all_ans.len()
        );
    }

    fn dfs_good_solution(&self, rem_ans: &Vec<Answer>) -> Score {
        assert!(rem_ans.len() > 0);
        if rem_ans.len() == 1 {
            return 1;
        }
        if rem_ans.len() == 2 {
            return 1 + 2;
        }

        let rem_id = self.cache.lock().unwrap().get_set_id(rem_ans);

        if let Some((val, ..)) = self.cache.lock().unwrap().memo.get(&rem_id) {
            return *val;
        }

        let all_guess = if rem_ans.len() == 3 {
            // 残りの候補から宣言した場合, 最悪 1 + 2 + 3 = 6
            // 他の候補を宣言した場合, 最良で 2 * 3 = 6

            // 従って残りの3つの候補から宣言する場合だけ考えれば良い.
            rem_ans
        } else {
            // おそらく5文字の宣言が最適
            &self.pokemons.all_ans
        };

        let good_guess = all_guess
            .par_iter()
            .min_by_key(|guess| {
                OrderedFloat(
                    self.judge_table
                        .partition(rem_ans, &guess)
                        .values()
                        .map(|s| {
                            // minimize average size, maximize entropy
                            let x = s.len() as f32;
                            (0.1 * x + x.log2()) * x
                        })
                        .sum::<f32>(),
                )
            })
            .unwrap();

        // TODO: avoid same calculation
        let part = self.judge_table.partition(rem_ans, &good_guess);

        let val: Score = rem_ans.len() as Score
            + part
                .par_iter()
                .map(|(_, s)| self.dfs_good_solution(s))
                .sum::<Score>();

        self.cache
            .lock()
            .unwrap()
            .memo
            .insert(rem_id, (val, *good_guess, part.clone()));

        val
    }

    fn lower_bound(&self, rem_ans: &Vec<Answer>, depth: usize) -> Score {
        assert!(rem_ans.len() > 0);
        if depth == 0 || rem_ans.len() <= 2 {
            return 2 * rem_ans.len() as Score - 1;
        }

        let rem_id = self.cache.lock().unwrap().get_set_id(rem_ans);

        if let Some((d, lb)) = self.cache.lock().unwrap().lb_memo.get(&rem_id) {
            if *d >= depth {
                return *lb;
            }
        }

        let all_guess = if rem_ans.len() == 3 {
            // 残りの候補から宣言した場合, 最悪 1 + 2 + 3 = 6
            // 他の候補を宣言した場合, 最良で 2 * 3 = 6

            // 従って残りの3つの候補から宣言する場合だけ考えれば良い.
            rem_ans
        } else {
            &self.pokemons.all_guess
        };

        let ret: Score = rem_ans.len() as Score
            + all_guess
                .par_iter()
                .map(|guess| {
                    self.judge_table
                        .partition(rem_ans, &guess)
                        .values()
                        .map(|s| self.lower_bound(s, depth - 1))
                        .sum::<Score>()
                })
                .min()
                .unwrap();

        // assert!(ret >= 2 * rem_ans.len() as Score - 1);

        self.cache
            .lock()
            .unwrap()
            .lb_memo
            .insert(rem_id, (depth, ret));
        ret
    }

    pub fn build_best_solution(&self) {
        println!(
            "期待回数(最適): {} = {}/{}",
            self.dfs_best_solution(&self.pokemons.all_ans, INFTY) as f32
                / self.pokemons.all_ans.len() as f32,
            self.dfs_best_solution(&self.pokemons.all_ans, INFTY),
            self.pokemons.all_ans.len()
        );
    }

    fn dfs_best_solution(&self, rem_ans: &Vec<Answer>, ub: Score) -> Score {
        assert!(rem_ans.len() > 0);
        if rem_ans.len() == 1 {
            return 1;
        }
        if rem_ans.len() == 2 {
            return 1 + 2;
        }

        let rem_id = self.cache.lock().unwrap().get_set_id(rem_ans);

        if let Some(val) = self.cache.lock().unwrap().best.get(&rem_id) {
            return *val;
        }

        if self.lower_bound(rem_ans, Self::LB_DEPTH_LIMIT) >= ub {
            return INFTY;
        }

        let mut val = self.dfs_good_solution(rem_ans);

        let all_guess = if rem_ans.len() == 3 {
            // 残りの候補から宣言した場合, 最悪 1 + 2 + 3 = 6
            // 他の候補を宣言した場合, 最良で 2 * 3 = 6

            // 従って残りの3つの候補から宣言する場合だけ考えれば良い.
            rem_ans
        } else {
            &self.pokemons.all_guess
        };

        let partitions: Vec<Partition> = all_guess
            .par_iter()
            .map(|guess| self.judge_table.partition(rem_ans, &guess))
            .collect();

        let penalty: Vec<f32> = partitions
            .par_iter()
            .map(|part| {
                // maximize "entropy"
                part.values()
                    .map(|s| {
                        let x = s.len() as f32;
                        x.log2() * x
                    })
                    .sum::<f32>()
            })
            .collect();

        let mut order: Vec<usize> = (0..all_guess.len()).collect();
        order.sort_by_key(|i| OrderedFloat(penalty[*i]));

        for &i in order.iter() {
            let guess = &all_guess[i];
            let part = &partitions[i];

            let lb: Score = rem_ans.len() as Score
                + part
                    .values()
                    .map(|s| self.lower_bound(s, Self::LB_DEPTH_LIMIT))
                    .sum::<Score>();

            // ここを並列化すると遅くなる.
            // // parallel
            // let lb: Score = rem_ans.len() as Score + part.par_iter().map(|(_, s)| {
            //     self.lower_bound(s, self.lb_depth_limit)
            // }).sum::<Score>();

            if lb >= val {
                continue;
            }

            let mut tmp = rem_ans.len() as Score;
            for s in part.values() {
                tmp += self.dfs_best_solution(s, val - tmp);
                if tmp >= val {
                    break;
                }
            }

            if tmp < val {
                val = tmp;
                self.cache
                    .lock()
                    .unwrap()
                    .memo
                    .insert(rem_id, (val, *guess, part.clone()));
            }
        }

        self.cache.lock().unwrap().best.insert(rem_id, val);

        val
    }

    pub fn write(&self, filepath: &str) {
        let mut guess_seq: Vec<Vec<Guess>> = (0..self.ans_until).map(|_| Vec::new()).collect();
        self.dfs_build_guess_seq(&mut guess_seq, &self.pokemons.all_ans);

        let mut f = fs::File::create(filepath).unwrap();
        for guess in &guess_seq {
            f.write_all(
                format!(
                    "{}\n",
                    guess
                        .iter()
                        .map(|g| g.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
                .as_bytes(),
            )
            .unwrap();
        }
    }

    fn dfs_build_guess_seq(&self, guess_seq: &mut Vec<Vec<Guess>>, rem_ans: &Vec<Answer>) {
        if rem_ans.len() == 1 {
            guess_seq[rem_ans[0]].push(rem_ans[0]);
            return;
        }
        if rem_ans.len() == 2 {
            guess_seq[rem_ans[0]].push(rem_ans[0]);
            guess_seq[rem_ans[1]].push(rem_ans[0]);
            guess_seq[rem_ans[1]].push(rem_ans[1]);
            return;
        }

        let (guess, part) = {
            let cache = self.cache.lock().unwrap();
            let rem_id = *cache.set_id.get(rem_ans).unwrap();
            let (_, guess, part) = cache.memo[&rem_id].clone();
            (guess, part)
        };

        for ans in rem_ans {
            guess_seq[*ans].push(guess.clone());
        }
        for s in part.values() {
            self.dfs_build_guess_seq(guess_seq, s);
        }
    }
}

#[derive(FromArgs)]
/// Minimize expectation of the number of guess
struct Args {
    /// the number of answer pokemons
    #[argh(option)]
    ans_until: usize,

    /// the number of guess pokemons
    #[argh(option)]
    guess_until: usize,

    /// the number of threads
    #[argh(option, short = 't', default = "default_num_threads()")]
    num_threads: usize,

    /// the filepath of decision tree output
    #[argh(option, short = 'o')]
    output: String,
}
fn default_num_threads() -> usize {
    1
}

fn main() {
    let args: Args = argh::from_env();

    let solver = Solver::new(args.ans_until, args.guess_until);

    //let guard = pprof::ProfilerGuard::new(100).unwrap();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.num_threads)
        .build()
        .unwrap();

    let start = Instant::now();
    pool.install(|| solver.build_best_solution());
    println!(
        "elapsed time: {:?} [sec]",
        start.elapsed().as_nanos() as f32 / 1_000_000_000 as f32
    );

    //match guard.report().build() {
    //    Ok(report) => {
    //        let mut file = fs::File::create("profile.pb").unwrap();
    //        let profile = report.pprof().unwrap();

    //        let mut content = Vec::new();
    //        profile.encode(&mut content).unwrap();
    //        file.write_all(&content).unwrap();
    //    }
    //    Err(_) => {}
    //};

    println!("best.len(): {:?}", solver.cache.lock().unwrap().best.len());
    println!("memo.len(): {:?}", solver.cache.lock().unwrap().memo.len());
    println!(
        "lb_memo.len(): {:?}",
        solver.cache.lock().unwrap().lb_memo.len()
    );

    solver.write(&args.output);
}
