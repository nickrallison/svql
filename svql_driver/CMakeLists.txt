cmake_minimum_required (VERSION 3.16)
project (svql_driver LANGUAGES CXX)

# ----------------------------------------------------------------------------
set (CMAKE_CXX_STANDARD 23)

# --- Source files ----------------------------------------------------------
set (SVQL_SRC
        src/GraphConversion.cpp
        src/RegexMap.cpp
        src/SubCircuitReSolver.cpp
        src/SvqlPass.cpp
)

add_library (svql_driver SHARED ${SVQL_SRC})

target_include_directories (svql_driver
        PUBLIC  
                ${CMAKE_CURRENT_SOURCE_DIR}/include)

set_target_properties (svql_driver PROPERTIES
        PREFIX ""
        LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/out")

# The necessary flags and include paths from Yosys are propagated automatically
# by Bazel through the generated CMake toolchain.
target_compile_options(svql_driver PRIVATE -D_YOSYS_)

# Simply link against the imported targets 'yosys' and 'svql_common'.
target_link_libraries(svql_driver PRIVATE yosys svql_common)

install(TARGETS svql_driver
        LIBRARY DESTINATION .
        ARCHIVE DESTINATION .
        RUNTIME DESTINATION .
)

install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/
        DESTINATION include
        FILES_MATCHING PATTERN "*.h")

add_custom_target (test
        COMMAND ${YOSYS_BIN}
        -m $<TARGET_FILE:svql_driver>
        ${CMAKE_CURRENT_SOURCE_DIR}/examples/cwe1234/variant1.v
        -p "hierarchy -top locked_register_example"
        -p "proc"
        -p "svql -map ./examples/cwe1234/locked_register_pat.v -verbose"
        DEPENDS svql_driver
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        COMMENT "Running example flow with freshly built svql plug-in")