use memu::Conf;
use clap::Parser;

fn main() {
    let conf = Conf::parse();
    memu::run(conf).unwrap_or_else(|e| println!("Error: {}", e));
}
