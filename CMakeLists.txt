cmake_minimum_required(VERSION 3.16)
project(win-git-status LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 17)

include(conan.cmake)
conan_cmake_run(CONANFILE conanfile.py
        BASIC_SETUP CMAKE_TARGETS
        BUILD missing)
include(${CMAKE_BINARY_DIR}/conan_paths.cmake)
find_package(libgit2)
find_package(Catch2)
add_subdirectory(src/win-git-status)

# We do this, call CTEST_TARGETS_ADDED to minimize the number of extranious targets enable_testing() adds
set_property(GLOBAL PROPERTY CTEST_TARGETS_ADDED 1)
enable_testing()
add_subdirectory(tests)
