use memu::Conf;
use structopt::StructOpt;

fn main() {
    let conf = Conf::from_args();
    memu::run(conf).unwrap_or_else(|e| println!("Error: {}", e));
}
