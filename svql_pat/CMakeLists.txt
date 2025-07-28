cmake_minimum_required(VERSION 3.20)
project(svql_pat NONE)

include(ExternalProject)

find_program(CARGO_EXECUTABLE     cargo     REQUIRED)

set(SVQL_COMMON_TARGET_DIR  ${CMAKE_BINARY_DIR}/cargo_target)

ExternalProject_Add(
    ${PROJECT_NAME}_build
    SOURCE_DIR      ${CMAKE_CURRENT_SOURCE_DIR}
    CONFIGURE_COMMAND ""
    BUILD_COMMAND
        ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${SVQL_COMMON_TARGET_DIR}
            ${CARGO_EXECUTABLE} build --release --lib
    BUILD_BYPRODUCTS
        ${SVQL_COMMON_TARGET_DIR}/release/lib${PROJECT_NAME}.a
    INSTALL_COMMAND ""
)