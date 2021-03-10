use std::env;
use std::path::Path;
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let index_file = path.join(".git/index");
    let index = win_git_status::Index::new(&*index_file).unwrap();
    let worktree = win_git_status::WorkTree::diff_against_index(path, index, true);
    println!("{:?}", worktree);
}
