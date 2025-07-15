#pragma once

#include <memory>
#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "SubCircuitReSolver.hpp"

using namespace Yosys;

namespace svql
{

    struct SvqlConfig
    {
        std::unique_ptr<SubCircuitReSolver> solver;

        std::vector<std::string> pat_filenames;
        std::vector<std::string> regex_filenames;
        std::map<std::string, std::map<RTLIL::IdString, std::regex>> pat_regexes;
        bool constports;
        bool nodefaultswaps;
        bool verbose;
    };

};
