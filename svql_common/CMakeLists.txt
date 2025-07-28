cmake_minimum_required(VERSION 3.20)

project(svql_common NONE)

find_program(
        CARGO_EXECUTABLE
        NAMES cargo
        HINTS "$ENV{HOME}/.cargo/bin"
        REQUIRED
)

#### CMake Cache Variables ####
if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_CMD ${CARGO_EXECUTABLE} build)
    set(TARGET_DIR "debug")
else ()
    set(CARGO_CMD ${CARGO_EXECUTABLE} build --release)
    set(TARGET_DIR "release")
endif ()

if (DEFINED ENV{CARGO_BUILD_TARGET})
    set(TARGET_DIR "$ENV{CARGO_BUILD_TARGET}/${TARGET_DIR}")
endif()

if(ENABLE_LTO)
    set(RUST_FLAGS_LIST "-C linker=clang")
    string (REPLACE ";" " " RUST_FLAGS "${RUST_FLAGS_LIST}")
endif()


#### CMake Setup Custom Outputs ####
set(GEN_SRCDIR ${CMAKE_CURRENT_BINARY_DIR}/build/${PROJECT_NAME})
file(MAKE_DIRECTORY "${GEN_SRCDIR}")
set(SVQL_COMMON_CONFIG_CXX "${GEN_SRCDIR}/${PROJECT_NAME}_config.cpp")
set(SVQL_COMMON_CONFIG_H   "${GEN_SRCDIR}/${PROJECT_NAME}_config.h")

set(SVQL_COMMON_MATCHES_CXX "${GEN_SRCDIR}/${PROJECT_NAME}_matches.cpp")
set(SVQL_COMMON_MATCHES_H   "${GEN_SRCDIR}/${PROJECT_NAME}_matches.h")

set(SVQL_COMMON_PATTERN_CXX "${GEN_SRCDIR}/${PROJECT_NAME}_pattern.cpp")
set(SVQL_COMMON_PATTERN_H   "${GEN_SRCDIR}/${PROJECT_NAME}_pattern.h")

set(SVQL_COMMON_LIB "${CMAKE_BINARY_DIR}/cargo_target/${TARGET_DIR}/lib${PROJECT_NAME}.a")

add_custom_command(
    OUTPUT ${SVQL_COMMON_LIB} ${SVQL_COMMON_CONFIG_CXX} ${SVQL_COMMON_CONFIG_H} ${SVQL_COMMON_MATCHES_CXX} ${SVQL_COMMON_MATCHES_H} ${SVQL_COMMON_PATTERN_CXX} ${SVQL_COMMON_PATTERN_H}
    COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${CMAKE_BINARY_DIR}/cargo_target
            RUSTFLAGS=${RUST_FLAGS}
            ${CARGO_CMD}
    COMMAND cp ${CMAKE_BINARY_DIR}/cargo_target/cxxbridge/${PROJECT_NAME}/src/config.rs.cc ${SVQL_COMMON_CONFIG_CXX}
    COMMAND cp ${CMAKE_BINARY_DIR}/cargo_target/cxxbridge/${PROJECT_NAME}/src/config.rs.h  ${SVQL_COMMON_CONFIG_H}
    COMMAND cp ${CMAKE_BINARY_DIR}/cargo_target/cxxbridge/${PROJECT_NAME}/src/matches.rs.cc ${SVQL_COMMON_MATCHES_CXX}
    COMMAND cp ${CMAKE_BINARY_DIR}/cargo_target/cxxbridge/${PROJECT_NAME}/src/matches.rs.h  ${SVQL_COMMON_MATCHES_H}
    COMMAND cp ${CMAKE_BINARY_DIR}/cargo_target/cxxbridge/${PROJECT_NAME}/src/pattern.rs.cc ${SVQL_COMMON_PATTERN_CXX}
    COMMAND cp ${CMAKE_BINARY_DIR}/cargo_target/cxxbridge/${PROJECT_NAME}/src/pattern.rs.h  ${SVQL_COMMON_PATTERN_H}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust+generating CXX bridge"
    VERBATIM
)
add_custom_target(${PROJECT_NAME}_gen
    DEPENDS
        ${SVQL_COMMON_LIB}
        ${SVQL_COMMON_CONFIG_CXX}
        ${SVQL_COMMON_CONFIG_H}
        ${SVQL_COMMON_MATCHES_CXX}
        ${SVQL_COMMON_MATCHES_H}
        ${SVQL_COMMON_PATTERN_CXX}
        ${SVQL_COMMON_PATTERN_H}
)

#### C++ Library Setup ####
add_library(${PROJECT_NAME} STATIC
    ${SVQL_COMMON_CONFIG_CXX}
    ${SVQL_COMMON_MATCHES_CXX}
    ${SVQL_COMMON_PATTERN_CXX}
)
target_include_directories(${PROJECT_NAME} PUBLIC ${CMAKE_CURRENT_SOURCE_DIR} ${GEN_SRCDIR})
target_link_libraries(${PROJECT_NAME} pthread dl ${SVQL_COMMON_LIB})

add_dependencies(${PROJECT_NAME} ${PROJECT_NAME}_gen)

add_test(NAME ${PROJECT_NAME}_test 
    COMMAND ${CARGO_EXECUTABLE} test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})