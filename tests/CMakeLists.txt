add_library(test_main TestMain.cpp)
target_link_libraries(test_main PRIVATE Catch2::Catch2 fixtures libgit2::libgit2)

add_subdirectory(fixtures)

add_executable(test_no_args TestNoArgs.cpp)
target_link_libraries(test_no_args Catch2::Catch2 fixtures test_main win-git-status)
add_test(NAME test_no_args COMMAND test_no_args)