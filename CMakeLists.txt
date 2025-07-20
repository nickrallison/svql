cmake_minimum_required(VERSION 3.13)
project(svql LANGUAGES CXX C)

# Set C++ standard and compatibility flags globally
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Enable LTO
option(ENABLE_LTO "Enable cross language linking time optimization" OFF)
if(ENABLE_LTO)
  include(CheckIPOSupported)
  check_ipo_supported(RESULT supported OUTPUT error)
  if(supported)
      message(STATUS "IPO / LTO enabled")
      set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
      add_link_options(-fuse-ld=lld)
  else()
      message(STATUS "IPO / LTO not supported: <${error}>")
  endif()
endif()

enable_testing()

add_subdirectory(svql_common)
add_subdirectory(yosys)
add_subdirectory(svql_driver)

# add_subdirectory(svql_pat)
# add_subdirectory(svql_pat_lib)