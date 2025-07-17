#pragma once

#include <memory>
#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "SubCircuitReSolver.hpp"

#include "svql_common.h"

using namespace Yosys;

namespace svql
{

    struct SvqlConfig
    {
        std::unique_ptr<SubCircuitReSolver> solver;

        std::string pat_filename;
        std::string pat_module_name;

        bool verbose = false;
        bool constports = false;
        bool nodefaultswaps = false;

        std::vector<std::pair<std::string, std::string>> compat_pairs;
        std::vector<std::pair<std::string, std::set<std::string>>> swap_ports;
        std::vector<std::pair<std::string, std::vector<std::string>>> perm_ports;
        std::vector<std::string> cell_attr;
        std::vector<std::string> wire_attr;
        bool ignore_parameters = false;
        std::vector<std::pair<std::string, std::string>> ignore_param;
    };

    SvqlConfig into_svql_runtime_config(const CSvqlRuntimeConfig &ccfg);
    SvqlRuntimeConfig into_c_svql_runtime_config(const SvqlConfig &config);

};
