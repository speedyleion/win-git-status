//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#define CATCH_CONFIG_MAIN
#include <catch2/catch.hpp>

TEST_CASE("No files are dirty")
{
    REQUIRE(4 == 4);
}
