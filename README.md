# win-git-status

git-status optimized for windows

The intent is to try and re-implement git status so that it can run better on windows. In 
particular for repositories that have submodules.

For example using a small repo (12 files) with 2 submodules and the 
[libgit2](https://libgit2.org/libgit2/ex/HEAD/status.html) status example:

- `git status` takes ~150ms
- `libgit2` takes ~90ms 

This was tested against git for windows version 2.29.

Though these times are fairly small, they start to really show up when one gets
to larger repositories.

> Note: The time for `libgit` may be a little misleading as 
> `libgit2` does not support full git status functionality

There are 2 Rust packages out there that provide git functionaltiy:

- [gitoxide](https://github.com/Byron/gitoxide)
- [git2-rs](https://github.com/rust-lang/git2-rs)

These packages are not being utilized in the implementation for two main reasons:

- Neither one seems to have it on their roadmap to support async.  Without profiling
  it's hard to say if async will help, but I've got a hunch that for windows it will.
  I think it will most likely be time consuming to get a status implementation working
  with one of these two as the backand and then try and rework to test out async performance.
- They are not meant to replace the git cli, thus they are missing some needed features.  
  i.e. git2-rs doesn't support getting all the information needed for a rebase status when
  in the middle of a rebase.

[git2-rs](https://github.com/rust-lang/git2-rs) is being used in the test verification.  This was
chosen with previous exposer to the `libgit2` api.

Since I chose to also take this opportunity to learn Rust, it means that the status on this will most
likely be slow as I spin up on all the nuances of Rust.
