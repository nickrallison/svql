cmake_minimum_required(VERSION 3.20)

project(svql_common NONE)

include(ExternalProject)

find_program(CARGO_EXECUTABLE     cargo     REQUIRED)
find_program(CBINDGEN_EXECUTABLE  cbindgen  REQUIRED)

set(SVQL_COMMON_TARGET_DIR  ${CMAKE_CURRENT_BINARY_DIR}/svql_common_target)
set(SVQL_COMMON_HEADER_DIR  ${CMAKE_CURRENT_BINARY_DIR}/include)
file(MAKE_DIRECTORY          ${SVQL_COMMON_HEADER_DIR})

ExternalProject_Add(
    svql_common_build
    SOURCE_DIR      ${CMAKE_CURRENT_SOURCE_DIR}  # assumes CMakeLists.txt is next to Cargo.toml
    CONFIGURE_COMMAND ""
    BUILD_COMMAND
        ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${SVQL_COMMON_TARGET_DIR}
            ${CARGO_EXECUTABLE} build --release --lib
        COMMAND
            ${CBINDGEN_EXECUTABLE}
                --lang C
                --output ${SVQL_COMMON_HEADER_DIR}/source.h
                ${CMAKE_CURRENT_SOURCE_DIR}
    BUILD_BYPRODUCTS
        ${SVQL_COMMON_TARGET_DIR}/release/libsvql_common.a
        ${SVQL_COMMON_HEADER_DIR}/source.h
    INSTALL_COMMAND ""
)

add_library(svql_common STATIC IMPORTED GLOBAL)
set_target_properties(
    svql_common
    PROPERTIES
        IMPORTED_LOCATION             ${SVQL_COMMON_TARGET_DIR}/release/libsvql_common.a
        INTERFACE_INCLUDE_DIRECTORIES ${SVQL_COMMON_HEADER_DIR}
)

add_dependencies(svql_common svql_common_build)