/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
/*
Some discussion on directory walking metrics.

Using the llvm-project form https://github.com/llvm/llvm-project.git at commit,
0f9f0a4046e11c2b4c130640f343e3b2b5db08c1
Some metrics:

- 0.340s git status
- 0.700s using the walkdir crate from rust,
  https://github.com/BurntSushi/walkdir.git this utilized the walkdir-list
  command with ``-c`` to only show the count.
- 0.910s using fd with the ``-I`` flag to not look at git ignore.
- 1.120s using fd.  This dumped result to a log file
- 1.228s using libgit2 status example
- 4.000s Utilizing tokio and walking a directory asynchronously by blindly copying
  https://stackoverflow.com/a/58825638/4866781

So how do we get to the speed of git status if walkdir takes almost twice as long as git status?
git status needs to also look at the file sha's.

- 0.614s ``fd > /dev/null``.  It looks like writing and or updating the output file is having
  significant performance issues.  Not 100% why it's faster than walkdir.
- 0.400s ``fd -I  > /dev/null``.
- 0.847s ``fd -j 1 -I > /dev/null``.  It looks like fd uses threads, by default it seems to favor 12
  when using one it's noticeably slower.
- 0.511s ``fd -j 2 -I > /dev/null``
- 0.396s ``fd -j 3 -I > /dev/null``
- 0.356s ``fd -j 4 -I > /dev/null``
- 0.362s ``fd -j 5 -I > /dev/null``
- 0.390s ``fd -j 6 -I > /dev/null``  From 6-12 it can get down to 0.390s but 12 will often
  hit 0.400s
 */