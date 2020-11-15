//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#include <filesystem>
#include <iostream>
#include "Repo.hpp"

int main(int argc, const char ** argv)
{
    auto dir = std::filesystem::current_path();
    try {
        auto repo = Repo(dir.string());
        std::cout << repo.status();
    }
    catch(std::exception e){
        std::cout << e.what() << "\n";
    }

    return 0;
}

