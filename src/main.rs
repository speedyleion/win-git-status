use std::env;
use std::path::Path;
fn main() {
    let args: Vec<String> = env::args().collect();
    let worktree = win_git_status::WorkTree::new(Path::new(&args[1])).unwrap();
    println!("{:?}", worktree);
}
