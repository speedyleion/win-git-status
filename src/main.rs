use std::env;
use win_git_status::RepoStatus;
use win_git_status::StatusError;

fn main() -> Result<(), StatusError> {
    let path = env::current_dir()?;
    let status = RepoStatus::new(&path);
    match status {
        Err(error) => println!("{}", error.message),
        Ok(status) => println!("{}", status.message().unwrap()),
    };
    Ok(())
}
