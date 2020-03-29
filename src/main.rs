mod crab;
use std::env;

fn main() {
    crab::foo();
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
}


