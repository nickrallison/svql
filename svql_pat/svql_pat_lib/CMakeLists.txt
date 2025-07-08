cmake_minimum_required (VERSION 3.16)
project (svql_pat_lib LANGUAGES CXX)

# ----------------------------------------------------------------------------
set (CMAKE_CXX_STANDARD 17)

# ----------------------------------------------------------------------------
set (YOSYS_BIN     "${yosys_BINARY_DIR}/yosys" CACHE FILEPATH "yosys executable")
set (YOSYS_CONFIG  "${yosys_BINARY_DIR}/yosys-config" CACHE FILEPATH "yosys-config executable")

# Query yosys-config
execute_process (COMMAND ${YOSYS_CONFIG} --cxxflags   OUTPUT_VARIABLE YOSYS_CXXFLAGS   OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process (COMMAND ${YOSYS_CONFIG} --ldflags    OUTPUT_VARIABLE YOSYS_LDFLAGS    OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process (COMMAND ${YOSYS_CONFIG} --ldlibs     OUTPUT_VARIABLE YOSYS_LDLIBS     OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process (COMMAND ${YOSYS_CONFIG} --datdir     OUTPUT_VARIABLE YOSYS_DATDIR     OUTPUT_STRIP_TRAILING_WHITESPACE)

# Convert the flag strings to proper CMake lists
separate_arguments (YOSYS_CXXFLAGS_LIST  UNIX_COMMAND "${YOSYS_CXXFLAGS}")
separate_arguments (YOSYS_LDFLAGS_LIST   UNIX_COMMAND "${YOSYS_LDFLAGS}")
separate_arguments (YOSYS_LDLIBS_LIST    UNIX_COMMAND "${YOSYS_LDLIBS}")

# --- Source files ----------------------------------------------------------
set (SVQL_SRC
        src/SvqlPatPass.cpp
)

add_library (${PROJECT_NAME} SHARED ${SVQL_SRC})

target_include_directories (${PROJECT_NAME}
        PUBLIC
                ${CMAKE_CURRENT_SOURCE_DIR}/include
                ${yosys_SOURCE_DIR}/
        )

target_compile_options    (${PROJECT_NAME} PRIVATE ${YOSYS_CXXFLAGS_LIST} -D_YOSYS_)
target_link_options       (${PROJECT_NAME} PRIVATE ${YOSYS_LDFLAGS_LIST})
target_link_libraries     (${PROJECT_NAME} PRIVATE ${YOSYS_LDLIBS_LIST} 
        yosys
)

install(TARGETS ${PROJECT_NAME}
        LIBRARY DESTINATION .
        ARCHIVE DESTINATION .
        RUNTIME DESTINATION .
)

install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/
        DESTINATION include
        FILES_MATCHING PATTERN "*.h")

# add_test(
#         NAME    ${PROJECT_NAME}_test
#         COMMAND ${yosys_SOURCE_DIR}/yosys
#                 -m $<TARGET_FILE:${PROJECT_NAME}>
#                 ${CMAKE_SOURCE_DIR}/examples/cwe1234/variant1.v
#                 -p "hierarchy -top locked_register_example"
#                 -p "proc"
#                 -p "svql -map ${CMAKE_SOURCE_DIR}/examples/cwe1234/locked_register_pat.v -verbose"
#     )
#     set_tests_properties(${PROJECT_NAME}_test PROPERTIES
#                          DEPENDS ${PROJECT_NAME}
#                          DEPENDS yosys)