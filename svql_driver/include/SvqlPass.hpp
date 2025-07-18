#pragma once

#include <variant>

#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "SubCircuitReSolver.hpp"

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

		// ####
		static CSvqlRuntimeConfig *parse_args_to_config(const std::vector<std::string> &args);
		static std::unique_ptr<SubCircuitReSolver> create_solver(const CSvqlRuntimeConfig *cfg);
		static RTLIL::Design *setup_needle_design(const CSvqlRuntimeConfig *cfg, std::string &error_msg);
		static CMatchList *run_solver(SubCircuitReSolver *solver, const CSvqlRuntimeConfig *cfg, RTLIL::Design *needle, RTLIL::Design *haystack);
	} SvqlPass;

	void print_wire(RTLIL::Wire *wire);
	std::string escape_needle_name(const std::string &name);
	std::vector<RTLIL::Wire *> get_cell_wires(RTLIL::Cell *cell);

} // namespace svql