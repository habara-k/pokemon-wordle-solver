use std::io::Write;

use argh::FromArgs;

use wordle_pokemon::{pokemon::*,judge::*,tree::*};

#[derive(FromArgs)]
/// Build decision tree
struct Args {
    /// the number of pokemons
    #[argh(option, short='n')]
    num_pokemons: usize,

    /// the filepath of decision tree input
    #[argh(option, short='i')]
    input: String,
}

fn main() {
    let args: Args = argh::from_env();
    let n = args.num_pokemons;

    let pokemons = PokemonList::new(n);

    let root = DecisionTree::new(&args.input).build(&pokemons.all_ans, 0);

    let mut history = vec![];

    loop {
        let nxt = DecisionTree::next(&root, &history);
        println!("(残り{}匹) {}", nxt.rem_ans.len(), POKEMONS[nxt.guess]);

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
    }
}