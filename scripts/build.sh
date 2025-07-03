#!/bin/sh

bazel build --jobs 32 --cpu=multi-threaded //yosys:yosys --show_progress --worker_verbose --verbose_failures --sandbox_debug
# bazel build --jobs 32 --cpu=multi-threaded //yosys:cxxopts_header --show_progress --worker_verbose --verbose_failures --sandbox_debug