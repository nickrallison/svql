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

        std::string pat_filename;
        std::string pat_module_name;
        bool constports;
        bool nodefaultswaps;
        bool verbose;
    };

};
