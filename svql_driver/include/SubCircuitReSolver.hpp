#pragma once

#include "kernel/rtlil.h"
#include "kernel/yosys.h"
#include "libs/subcircuit/subcircuit.h"

using namespace Yosys;

namespace svql {
/*  A very thin wrapper around SubCircuit::Solver that adds regex based net-name
 * comparison */
class SubCircuitReSolver : public SubCircuit::Solver {
 public:
  SubCircuitReSolver();

  void setVerbose(bool enable = true);

  // Attribute / parameter knobs ------------------------------------
  bool ignoreParameters = false;
  std::set<std::pair<RTLIL::IdString, RTLIL::IdString>> ignoredParams;
  std::set<RTLIL::IdString> cell_attr;
  std::set<RTLIL::IdString> wire_attr;

 private:
  //  SubCircuit::Solver hooks
  bool compareAttributes(
      const std::set<RTLIL::IdString> &attr,
      const dict<RTLIL::IdString, RTLIL::Const> &needleAttr,
      const dict<RTLIL::IdString, RTLIL::Const> &haystackAttr) const;
  bool userCompareNodes(
      const std::string &, const std::string &, void *needleUser,
      const std::string &, const std::string &, void *haystackUser,
      const std::map<std::string, std::string> &portMap) override;

  // helpers ---------------------------------------------------------
  RTLIL::Const unifiedParam(RTLIL::IdString cell_type, RTLIL::IdString param,
                            RTLIL::Const value);

  bool myVerbose = false;
};

}  // namespace svql
