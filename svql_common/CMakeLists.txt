cmake_minimum_required(VERSION 3.20)

project(svql_common NONE)

include(ExternalProject)

find_program(CARGO_EXECUTABLE     cargo     REQUIRED)
find_program(CBINDGEN_EXECUTABLE  cbindgen  REQUIRED)

set(SVQL_COMMON_TARGET_DIR  ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}_target)
set(SVQL_COMMON_HEADER_DIR  ${CMAKE_CURRENT_BINARY_DIR}/include)
file(MAKE_DIRECTORY          ${SVQL_COMMON_HEADER_DIR})

ExternalProject_Add(
    ${PROJECT_NAME}_build
    SOURCE_DIR      ${CMAKE_CURRENT_SOURCE_DIR}
    CONFIGURE_COMMAND ""
    BUILD_COMMAND
        ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${SVQL_COMMON_TARGET_DIR}
            ${CARGO_EXECUTABLE} build --release --lib
        COMMAND
            ${CBINDGEN_EXECUTABLE}
                --lang C++
                --output ${SVQL_COMMON_HEADER_DIR}/svql_common.h
                ${CMAKE_CURRENT_SOURCE_DIR}
    BUILD_BYPRODUCTS
        ${SVQL_COMMON_TARGET_DIR}/release/lib${PROJECT_NAME}.a
        ${SVQL_COMMON_HEADER_DIR}/svql_common.h
    INSTALL_COMMAND ""
)

add_library(${PROJECT_NAME} STATIC IMPORTED GLOBAL)
set_target_properties(
    ${PROJECT_NAME}
    PROPERTIES
        IMPORTED_LOCATION             ${SVQL_COMMON_TARGET_DIR}/release/lib${PROJECT_NAME}.a
        INTERFACE_INCLUDE_DIRECTORIES ${SVQL_COMMON_HEADER_DIR}
)

add_dependencies(${PROJECT_NAME} ${PROJECT_NAME}_build)