git submodule update --init --recursive
cp scripts/yosys_BUILD_bazel yosys/BUILD.bazel
# Remove vendored cxxopts bazel file, should be found by glob
rm -f yosys/libs/cxxopts/BUILD.bazel yosys/libs/cxxopts/WORKSPACE
