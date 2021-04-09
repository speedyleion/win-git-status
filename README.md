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

[git2-rs](https://github.com/rust-lang/git2-rs) is being used for some of the backend logic.  
It provides much of the git plumbing functionality.  However [git2-rs](https://github.com/rust-lang/git2-rs)
does not support running multi-threaded and in particular in comparing the working tree multi-threaded
seems to really perform.  So the working tree comparison is more or less re-implemented.

## Status
I've been trying to blog a bit about the development process at
https://speedyleion.github.io/.  This would probably give a better idea of the design progress, 
since the functional progress is taking a bit.

>Note: Since I chose to also take this opportunity to learn Rust, it means that
> the status on this will most likely be slow as I spin up on all the nuances 
> of Rust.

Currently ``win-git-status.exe`` will produce a message similar to ``git status``
Note: The repo directory needs to be specified, and it can be anywhere.

For example one could do:

    win-git-status.exe .

This currently doesn't handle significant features like:
 - ignore files,
 - merge states
 - rebase states
 - cherry-pick states
 - bisect state
 - submodules, (these are partially implemented, but won't notice commit changes)
    
### Performance
For repos without submodules ``win-git-status`` currently does not perform as well as 
``git status``.

Running on this repo:
- 0.049s for ``git status``
- 0.102s for ``win-git-status.exe .``

For repos with submodules ``win-git-status.exe`` can be up to 6-7x faster at times.
For one proprietary repo:
- 1.8s for ``git status``
- 0.3s for ``win-git-status.exe``

For another proprietary repo:
- 5.9s for ``git status``
- 0.8s for ``win-git-status.exe``


