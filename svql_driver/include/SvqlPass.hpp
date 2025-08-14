#pragma once

#include <optional>

#include "SubCircuitSolver.hpp"
#include "kernel/register.h"
#include "kernel/rtlil.h"
#include "kernel/yosys.h"
#include "svql_common_config.h"
#include "svql_common_matches.h"

using namespace Yosys;

namespace svql {

struct SvqlPass : public Pass {
    SvqlPass();
    void help() override;
    void execute(std::vector<std::string> args, RTLIL::Design *design) override;
    void terminate();
    void terminate(std::string error_message);

    void execute_cmd(std::vector<std::string> args, RTLIL::Design *design);
    void execute_net(std::vector<std::string> args, RTLIL::Design *design);

    // ####
    static std::optional<SvqlRuntimeConfig> parse_args_to_config(
        size_t &argsidx, const std::vector<std::string> &args,
        std::string &error_msg);
    static std::optional<uint16_t> parse_args_net(
        size_t &argsidx, const std::vector<std::string> &args,
        std::string &error_msg);
    static std::unique_ptr<SubCircuitSolver> create_solver(
        const SvqlRuntimeConfig &cfg);
    static RTLIL::Design *setup_needle_design(const SvqlRuntimeConfig &cfg,
                                              std::string &error_msg);
    static std::optional<QueryMatchList> run_solver(
        SubCircuitSolver *solver, const SvqlRuntimeConfig &cfg,
        RTLIL::Design *needle, RTLIL::Design *haystack, std::string &error_msg);
} SvqlPass;

void print_wire(RTLIL::Wire *wire);
std::string escape_needle_name(const std::string &name);
std::vector<RTLIL::Wire *> get_cell_wires(RTLIL::Cell *cell);

}  // namespace svql