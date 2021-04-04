use std::env;
use std::path::Path;
use win_git_status::RepoStatus;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let status = RepoStatus::new(path).unwrap();
    println!("{:?}", status);
}
