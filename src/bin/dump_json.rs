use std::fs;
use argh::FromArgs;

use wordle_pokemon::{pokemon::*,tree::*};

#[derive(FromArgs)]
/// Build decision tree
struct Args {
    /// the number of pokemons
    #[argh(option, short='n')]
    num_pokemons: usize,

    /// the filepath of decision tree input
    #[argh(option, short='i')]
    input: String,

    /// the filepath of decision tree json output
    #[argh(option, short='o')]
    output: String,
}

fn main() {
    let args: Args = argh::from_env();
    let n = args.num_pokemons;

    let pokemons = PokemonList::new(n);

    let root = DecisionTree::new(&args.input).build(&pokemons.all_ans, 0);


    let mut f = fs::File::create(&args.output).unwrap();
    root.write(&mut f);

    return;
}