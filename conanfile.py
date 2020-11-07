#           Copyright Nick G. 2020.
#  Distributed under the Boost Software License, Version 1.0.
#     (See accompanying file LICENSE or copy at
#           https://www.boost.org/LICENSE_1_0.txt)

from conans import ConanFile, CMake

class WinGitStatus(ConanFile):
    settings = "os", "compiler", "build_type", "arch"
    requires = "libgit2/1.0.1", "catch2/2.13.3"
    generators = "cmake_find_package", "cmake_paths", "cmake", "gcc", "txt"
    default_options = {"libgit2:shared": False}

    def imports(self):
        self.copy("*.lib", dst="bin", src="bin") # From bin to bin

    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()
