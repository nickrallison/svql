git submodule update --init --recursive

cp scripts/yosys_BUILD_bazel yosys/BUILD.bazel
cp scripts/yosys_libs_cxxopts_MODULE yosys/libs/cxxopts/MODULE.bazel
cp scripts/yosys_libs_cxxopts_BUILD yosys/libs/cxxopts/BUILD.bazel

rm yosys/libs/cxxopts/WORKSPACE
# Remove vendored cxxopts bazel file, should be found by glob
# rm -f yosys/libs/cxxopts/BUILD.bazel yosys/libs/cxxopts/WORKSPACE