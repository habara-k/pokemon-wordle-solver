use argh::FromArgs;
use std::fs;

use wordle_pokemon::{pokemon::*, tree::*};

#[derive(FromArgs)]
/// Build decision tree
struct Args {
    /// the filepath of decision tree input
    #[argh(option, short = 'i')]
    input: String,

    /// the filepath of decision tree json output
    #[argh(option, short = 'o')]
    output: String,
}

fn main() {
    let args: Args = argh::from_env();

    let tree = DecisionTree::new(&args.input);
    let pokemons = PokemonList::new(tree.judge_table.ans_until, tree.judge_table.guess_until);
    let root = tree.build(&pokemons.all_ans, 0);

    let mut f = fs::File::create(&args.output).unwrap();
    root.write(&mut f);

    return;
}
