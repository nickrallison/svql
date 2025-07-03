git submodule update --init --recursive

cp scripts/yosys_BUILD_bazel yosys/BUILD.bazel
cp scripts/yosys_abc_CMakeLists yosys/abc/CMakeLists.txt

# rm yosys/libs/cxxopts/WORKSPACE
# Remove vendored cxxopts bazel file, should be found by glob
rm -f yosys/libs/cxxopts/BUILD.bazel yosys/libs/cxxopts/WORKSPACE
find yosys/ -mindepth 2 -name "BUILD.bazel" -delete
find yosys/ -mindepth 2 -name "BUILD" -delete