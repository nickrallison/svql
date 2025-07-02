cmake_minimum_required (VERSION 3.16)
project (svql_driver LANGUAGES CXX)

# ----------------------------------------------------------------------------
set (CMAKE_CXX_STANDARD 23)

# ----------------------------------------------------------------------------
# The 'svql_common' library is now made available by the Bazel-generated
# CMake toolchain (via rules_foreign_cc). We no longer need to manually
# define an IMPORTED library from cache variables. We can just link to it
# directly later in this file.

# ----------------------------------------------------------------------------
set (YOSYS_BIN     "${CMAKE_CURRENT_SOURCE_DIR}/libs/yosys/yosys" CACHE FILEPATH "yosys executable")
set (YOSYS_CONFIG  "${CMAKE_CURRENT_SOURCE_DIR}/libs/yosys/yosys-config" CACHE FILEPATH "yosys-config executable")
set (YOSYS_SOURCE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/libs/yosys" CACHE PATH "Root of the Yosys source tree")

# Query yosys-config
execute_process (COMMAND ${YOSYS_CONFIG} --cxxflags   OUTPUT_VARIABLE YOSYS_CXXFLAGS   OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process (COMMAND ${YOSYS_CONFIG} --ldflags    OUTPUT_VARIABLE YOSYS_LDFLAGS    OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process (COMMAND ${YOSYS_CONFIG} --ldlibs     OUTPUT_VARIABLE YOSYS_LDLIBS     OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process (COMMAND ${YOSYS_CONFIG} --datdir     OUTPUT_VARIABLE YOSYS_DATDIR     OUTPUT_STRIP_TRAILING_WHITESPACE)

# Convert the flag strings to proper CMake lists
separate_arguments (YOSYS_CXXFLAGS_LIST  UNIX_COMMAND "${YOSYS_CXXFLAGS}")
separate_arguments (YOSYS_LDFLAGS_LIST   UNIX_COMMAND "${YOSYS_LDFLAGS}")
separate_arguments (YOSYS_LDLIBS_LIST    UNIX_COMMAND "${YOSYS_LDLIBS}")

#
# --- Source files ----------------------------------------------------------
#
set (SVQL_SRC
        src/GraphConversion.cpp
        src/RegexMap.cpp
        src/SubCircuitReSolver.cpp
        src/SvqlPass.cpp
)

add_library (svql SHARED ${SVQL_SRC})

target_include_directories (svql
        PUBLIC
        "${YOSYS_SOURCE_ROOT}"
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>)

set_target_properties (svql PROPERTIES
        PREFIX ""
        LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/out")

target_compile_options    (svql PRIVATE ${YOSYS_CXXFLAGS_LIST} -D_YOSYS_)
target_link_options       (svql PRIVATE ${YOSYS_LDFLAGS_LIST})
#target_link_libraries     (svql PRIVATE ${YOSYS_LDLIBS_LIST} json)
target_link_libraries     (svql PRIVATE ${YOSYS_LDLIBS_LIST} svql_common)

target_include_directories (svql PRIVATE ${YOSYS_SOURCE_ROOT})

install(TARGETS svql
        LIBRARY DESTINATION .
        ARCHIVE DESTINATION .
        RUNTIME DESTINATION .
)

# install our public headers
install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/
        DESTINATION include
        FILES_MATCHING PATTERN "*.h")

add_custom_target (test
        COMMAND ${YOSYS_BIN}
        -m $<TARGET_FILE:svql>
        ${CMAKE_CURRENT_SOURCE_DIR}/examples/cwe1234/variant1.v
        -p "hierarchy -top locked_register_example"
        -p "proc"
        -p "svql -map ./examples/cwe1234/locked_register_pat.v -verbose"
        DEPENDS svql
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        COMMENT "Running example flow with freshly built svql plug-in")