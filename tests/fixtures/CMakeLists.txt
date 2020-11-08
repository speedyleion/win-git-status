add_library(fixtures TempDirectory.cpp TempDirectory.hpp TempRepo.cpp TempRepo.hpp RepoBuilder.cpp RepoBuilder.hpp)
target_include_directories(fixtures PUBLIC .)
target_link_libraries(fixtures PUBLIC libgit2::libgit2 PRIVATE Catch2::Catch2)