use std::io::Write;

use argh::FromArgs;

use wordle_pokemon::{judge::*, pokemon::*, tree::*};

#[derive(FromArgs)]
/// Build decision tree
struct Args {
    /// the filepath of decision tree input
    #[argh(option, short = 'i')]
    input: String,
}

fn main() {
    let args: Args = argh::from_env();

    let tree = DecisionTree::new(&args.input);
    let pokemons = PokemonList::new(tree.judge_table.ans_until, tree.judge_table.guess_until);
    let mut node = tree.build(&pokemons.all_ans, 0);

    while let Node::NonTerminal { guess, rem_ans, .. } = &*node {
        println!("(残り{}匹) {}", rem_ans.len(), POKEMONS[*guess]);

        print!("-> ");
        std::io::stdout().flush().unwrap();
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).unwrap();
        let s = s.trim().to_string();

        node = node.next(
            &(s.chars()
                .enumerate()
                .map(|(i, c)| c.to_digit(10).unwrap() << 2 * i)
                .sum::<u32>() as Judge),
        );
    }
}
