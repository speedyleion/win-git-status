# win-git-status

git-status optimized for windows

Using git for window's `status` command can be slow on windows machines,
particularly when submodules are involved.

For example on the small repository generated in unit tests: 

- `git status` takes ~150ms
- `win-git-status` takes ~90ms 

The repository has 2 local submodules.  

This was tested against git for windows version 2.29.

Though these times are fairly small, they start to really show up when one gets
to larger repositories.

> Note: The time for `win-git-status` may be a little misleading as 
> `win-git-status` is not fully functional at this point. So it may slow down 
> some when the full functionality is added.  

# Status

This is currently a proof of concept.  Where `win-git-status` closely mimics 
`git status` with no arguments.  However things like rebase state, cherry-pick 
state, and merge conflicts are not yet supported.

# Building

`win-git-status` is built using CMake.  It also requires 
[conan](https://conan.io/) to be installed and available.

    cmake -B build_dir
    cmake --build build_dir --config Release
    
> Note: For some reason the builds with Ninja (using Clion) seemed to have
> better run time performance than those using the default of msbuild

# Roadmap

1. Investigate if there is a faster backend for getting the status.  Currently
   `win-git-status` is using [libgit2](https://libgit2.org/) as the backend.
   `libgit2` does support threads, 
   [GIT_FEATURE_THREADS](https://libgit2.org/libgit2/#v0.21.4/group/libgit2/git_libgit2_features),
   but it's not clear if this is leveraged for the status operations. It 
   may be worth looking into. It might also be worth seeing if something like 
   [std::async](https://en.cppreference.com/w/cpp/thread/async) might
   improve speed.
2. Finish out the status command so that it can actually be a full replacement 
   for `git status`, with no arguments.
3. Add argument support so that it can fully replace `git status`.
   