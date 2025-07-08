#include "SvqlPatPass.hpp"

#include <fstream>
#include <regex>

#include "kernel/log.h"
#include "kernel/sigtools.h"

using namespace Yosys;

SvqlPatPass::SvqlPatPass() : Pass("svql_pat", "takes a verilog file and prints a pattern of its interface for the svql pass") {}

void SvqlPatPass::help()
{
	log("\n");
	log("    svql_pat -pat <map_file> [options] [selection]\n");
	log("\n");
	log("This pass prints a pattern for use by the svql pass\n");
	log("\n");
	log("    -pat <pattern_file>\n");
	log("        prints the pattern for the given file\n");
	log("\n");
}

void SvqlPatPass::execute(std::vector<std::string> args, RTLIL::Design *design)
{
	log_header(design, "Executing SVQL PAT pass.\n");
	log_push();

	std::vector<std::string> pattern_files = std::vector<std::string>();

	size_t argidx;
	for (argidx = 1; argidx < args.size(); argidx++)
	{

		if (args[argidx] == "-pat" && argidx + 1 < args.size())
		{
			pattern_files.emplace_back(args[++argidx]);
			continue;
		}
		break;
	}

	extra_args(args, argidx, design);

	if (pattern_files.empty())
	{
		log_error("No pattern files specified.\n");
	}
	for (const auto &pattern_file : pattern_files)
	{
		std::cout << "Pattern file: " << pattern_file << std::endl;
	}

	log_pop();
}
