cmake_minimum_required (VERSION 3.16)
project (svql_driver LANGUAGES CXX)

# ----------------------------------------------------------------------------
set (CMAKE_CXX_STANDARD 17)

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
                ${CMAKE_CURRENT_SOURCE_DIR}/include
                ${yosys_SOURCE_DIR})

# set_target_properties (svql_driver PROPERTIES
#         LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/out")

target_compile_options(svql_driver PRIVATE -D_YOSYS_)
target_link_libraries(svql_driver PUBLIC svql_common)


install(TARGETS svql_driver
        LIBRARY DESTINATION .
        ARCHIVE DESTINATION .
        RUNTIME DESTINATION .
)

install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/
        DESTINATION include
        FILES_MATCHING PATTERN "*.h")

add_test(
        NAME    svql_driver_test
        COMMAND $<TARGET_FILE:yosys_exe>
                -m $<TARGET_FILE:svql_driver>
                ${CMAKE_SOURCE_DIR}/examples/cwe1234/variant1.v
                -p "hierarchy -top locked_register_example"
                -p "proc"
                -p "svql -map ${CMAKE_SOURCE_DIR}/examples/cwe1234/locked_register_pat.v -verbose"
    )
    set_tests_properties(svql_driver_test PROPERTIES
                         DEPENDS svql_driver 
                         DEPENDS yosys_exe)