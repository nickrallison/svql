#pragma once

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
		void setup(SvqlConfig &config);
		void terminate();
		void terminate(std::string error_message);

		CMatchList *run_query(std::string pat_filename, std::string pat_module_name);

		// ####
		static SvqlConfig configure(std::vector<std::string> args, RTLIL::Design *design, size_t &argidx);

		// ####
		RTLIL::Design *design = nullptr;
		RTLIL::Design *needle_design = nullptr;

	} SvqlPass;

	void print_wire(RTLIL::Wire *wire);
	std::string escape_needle_name(const std::string &name);
	std::vector<RTLIL::Wire *> get_cell_wires(RTLIL::Cell *cell);

} // namespace svql