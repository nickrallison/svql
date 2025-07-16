#pragma once

#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "SvqlConfig.hpp"

using namespace Yosys;

namespace svql
{

	struct SvqlPass : public Pass
	{
		SvqlPass();
		void help() override;
		void execute(std::vector<std::string> args, RTLIL::Design *design) override;
		SvqlConfig configure(std::vector<std::string> args, RTLIL::Design *design, size_t &argidx);
	} SvqlPass;

	std::string escape_needle_name(const std::string &name);
	std::vector<RTLIL::Wire *> get_cell_wires(RTLIL::Cell *cell);

} // namespace svql