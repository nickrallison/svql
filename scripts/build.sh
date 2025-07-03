#!/bin/sh

bazel build //yosys:yosys --verbose_failures --sandbox_debug
# bazel build //yosys:cxxopts_header --verbose_failures --sandbox_debug