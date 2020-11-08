add_library(win-git-status Repo.cpp Repo.hpp)

target_link_libraries(win-git-status PRIVATE libgit2::libgit2)

target_include_directories(win-git-status PUBLIC .)

add_executable(main Main.cpp)