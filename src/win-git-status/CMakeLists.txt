add_executable(win-git-status Main.cpp)

target_link_libraries(win-git-status PRIVATE libgit2::libgit2)