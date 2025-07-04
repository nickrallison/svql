cmake_minimum_required(VERSION 3.13)
project(yosys LANGUAGES CXX C)

enable_testing()

add_subdirectory(svql_common)
add_subdirectory(yosys)
add_subdirectory(svql_driver)

