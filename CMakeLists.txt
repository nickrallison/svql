cmake_minimum_required(VERSION 3.13)
project(svql LANGUAGES CXX C)

# Set C++ standard and compatibility flags globally
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add compatibility flags to avoid GLIBCXX version issues
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -D_GLIBCXX_USE_CXX11_ABI=1")

enable_testing()

add_subdirectory(svql_common)
add_subdirectory(yosys)
add_subdirectory(svql_driver)

add_subdirectory(svql_pat)
add_subdirectory(svql_pat_lib)