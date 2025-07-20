#pragma once

#include <optional>

#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "SubCircuitReSolver.hpp"

#include "svql_common_config.h"
#include "svql_common_mat.h"

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
		static std::optional<SvqlRuntimeConfig> parse_args_to_config(size_t &argsidx, const std::vector<std::string> &args, std::string &error_msg);
		static std::unique_ptr<SubCircuitReSolver> create_solver(const SvqlRuntimeConfig &cfg);
		static RTLIL::Design *setup_needle_design(const SvqlRuntimeConfig &cfg, std::string &error_msg);
		static std::optional<QueryMatchList> run_solver(SubCircuitReSolver *solver, const SvqlRuntimeConfig &cfg, RTLIL::Design *needle, RTLIL::Design *haystack, std::string &error_msg);
	} SvqlPass;

	void print_wire(RTLIL::Wire *wire);
	std::string escape_needle_name(const std::string &name);
	std::vector<RTLIL::Wire *> get_cell_wires(RTLIL::Cell *cell);

} // namespace svql