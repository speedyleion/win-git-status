use std::env;
use std::path::Path;
fn main() {
    let args: Vec<String> = env::args().collect();
    let index = win_git_status::Index::new(Path::new(&args[1])).unwrap();
    println!("{:?}", index);
}
