use clap::{App, Arg};
use std::{env, process};
use termcolor::{ColorChoice, StandardStream};
use win_git_status::RepoStatus;
use win_git_status::StatusError;

fn run() -> Result<(), StatusError> {
    let matches = App::new("Win-git-status")
        .version("0.1.0")
        .about("Performs basic git status operations optimized for windows")
        .arg(
            Arg::with_name("short")
                .short("s")
                .long("short")
                .takes_value(false)
                .help("Give the output in the short-format."),
        )
        .get_matches();

    let path = env::current_dir()?;
    let status = RepoStatus::new(&path)?;
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    if matches.is_present("short") {
        status.write_short_message(&mut stdout)?;
    } else {
        status.write_long_message(&mut stdout)?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        process::exit(1);
    }
}
