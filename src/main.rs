use lib::chip;

fn main() {
    let mut a = chip::Chip::new();
    a.load_program("ibm.ch8");
    a.interpret();
}
