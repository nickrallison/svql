#pragma once
#include <vector>

#include "libs/subcircuit/subcircuit.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

using namespace Yosys;

namespace svql
{
    // Convenience helper used by the solver
    std::vector<RTLIL::Wire *> get_output_wires(RTLIL::Cell *cell);

    // Wraps the long module-to-graph conversion routine
    bool module2graph(SubCircuit::Graph &graph, RTLIL::Module *mod,
                      bool constPorts,
                      RTLIL::Design *sel = nullptr,
                      int maxFanout = -1,
                      std::set<std::pair<RTLIL::IdString,
                                         RTLIL::IdString>> *split = nullptr);

    struct bit_ref_t
    {
        std::string cell, port;
        int bit;
    };

} // namespace svql