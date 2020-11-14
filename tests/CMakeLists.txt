add_library(test_main TestMain.cpp)
target_link_libraries(test_main PRIVATE Catch2::Catch2 fixtures libgit2::libgit2)

add_subdirectory(fixtures)

add_executable(test_no_args TestNoArgs.cpp)
target_link_libraries(test_no_args Catch2::Catch2 fixtures test_main repo)
add_test(NAME test_no_args COMMAND test_no_args)

add_executable(test_status_unstaged TestStatusUnstaged.cpp)
target_link_libraries(test_status_unstaged Catch2::Catch2 fixtures test_main repo)
add_test(NAME test_status_unstaged COMMAND test_status_unstaged)

add_executable(test_repo TestRepo.cpp)
target_link_libraries(test_repo Catch2::Catch2 fixtures test_main repo)
add_test(NAME test_repo COMMAND test_repo)

add_executable(test_status_branch TestStatusBranch.cpp)
target_link_libraries(test_status_branch Catch2::Catch2 fixtures test_main repo)
add_test(NAME test_status_branch COMMAND test_status_branch)
