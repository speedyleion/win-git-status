use std::{env, process};
use termcolor::{ColorChoice, StandardStream};
use win_git_status::RepoStatus;
use win_git_status::StatusError;

fn run() -> Result<(), StatusError> {
    let path = env::current_dir()?;
    let status = RepoStatus::new(&path)?;
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    status.write_long_message(&mut stdout)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        process::exit(1);
    }
}
