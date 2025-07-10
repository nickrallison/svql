#include "SvqlPatPass.hpp"

#include <fstream>
#include <regex>

#include "kernel/register.h"
#include "kernel/log.h"
#include "kernel/sigtools.h"

// #include "svql_common.h"

using namespace Yosys;

SvqlPatPass::SvqlPatPass() : Pass("svql_pat", "takes a verilog file and prints a pattern of its interface for the svql pass") {}

void SvqlPatPass::help()
{
	log("\n");
	log("    svql_pat -module <module name> -pattern_file <pattern_file> [options] [selection]\n");
	log("\n");
	log("This pass prints a pattern of the selected module name for use by the svql pass\n");
	log("\n");
	log("    -pat <pattern_file>\n");
	log("        prints the pattern for the given file\n");
	log("\n");
}

void SvqlPatPass::execute(std::vector<std::string> args, RTLIL::Design *design)
{
	log_header(design, "Executing SVQL PAT pass.\n");
	log_push();

	std::string pattern_file = "";
	std::string module_name = "";


	size_t argidx;
	for (argidx = 1; argidx < args.size(); argidx++)
	{

		if (args[argidx] == "-module" && argidx + 1 < args.size())
		{
			module_name = args[++argidx];
			continue;
		}

		if (args[argidx] == "-pattern_file" && argidx + 1 < args.size())
		{
			pattern_file = args[++argidx];
			continue;
		}

		break;
	}

	extra_args(args, argidx, design);

	// if (pattern_files.empty())
	// {
	// 	log_error("No pattern files specified.\n");
	// }
	// for (const auto &pattern_file : pattern_files)
	// {
	// 	std::cout << "Pattern file: " << pattern_file << std::endl;
	// }

	CPattern *pattern = cpattern_new(file_loc, nullptr, 0, nullptr, 0, nullptr, 0);
	

	log_pop();
}

// struct CPattern *cpattern_new(const char *file_loc,
//                               const char *const *in_ports,
//                               uintptr_t in_ports_len,
//                               const char *const *out_ports,
//                               uintptr_t out_ports_len,
//                               const char *const *inout_ports,
//                               uintptr_t inout_ports_len);

CPattern* get_patterns_from_file(const std::string &file_path)
{
	const char* file_loc = file_path.c_str();

	CPattern *pattern = cpattern_new(file_loc, nullptr, 0, nullptr, 0, nullptr, 0);



	return pattern;
}