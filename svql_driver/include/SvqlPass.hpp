#pragma once

#include <variant>

#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "SvqlConfig.hpp"

#include "svql_common.h"

using namespace Yosys;

namespace svql
{

	struct SvqlPass : public Pass
	{
		SvqlPass();
		void help() override;
		void execute(std::vector<std::string> args, RTLIL::Design *design) override;
		void terminate();
		void terminate(std::string error_message);

		CMatchList *run_query(const SvqlConfig &config, RTLIL::Design *needle_design, RTLIL::Design *design);

		// ####
		static std::variant<RTLIL::Design *, std::string> setup(SvqlConfig &config, std::string &pat_filename, std::string &pat_module_name);
		static SvqlConfig configure(std::vector<std::string> args, size_t &argidx);
		static SvqlConfig configure(CConfig &ccfg);

	} SvqlPass;

	void print_wire(RTLIL::Wire *wire);
	std::string escape_needle_name(const std::string &name);
	std::vector<RTLIL::Wire *> get_cell_wires(RTLIL::Cell *cell);

} // namespace svql