add_library(repo Repo.cpp Repo.hpp Status.cpp Status.hpp)

target_link_libraries(repo PUBLIC libgit2::libgit2)

target_include_directories(repo PUBLIC .)

add_executable(win-git-status Main.cpp)

target_link_libraries(win-git-status PRIVATE repo)
