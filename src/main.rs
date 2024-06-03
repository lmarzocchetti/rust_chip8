use lib::chip;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        std::process::exit(0);
    }

    println!("{}", args[1]);

    let mut a = chip::Chip::new();
    a.load_program(&args[1]);
    a.interpret();
}
