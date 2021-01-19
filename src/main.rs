use std::env;
mod index;
use std::path::Path;
fn main() {
    let args: Vec<String> = env::args().collect();
    let index = index::Index::new(Path::new(&args[1])).unwrap();
    println!("{:?}", index);
}
