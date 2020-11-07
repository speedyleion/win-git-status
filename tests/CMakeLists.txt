add_executable(test_no_args TestNoArgs.cpp)
target_link_libraries(test_no_args Catch2::Catch2)
add_test(NAME test_no_args COMMAND test_no_args)