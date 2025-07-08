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

std::vector<CPattern *> get_patterns_from_file(const std::string &file_path)
{
	const char* file_loc = file_path.c_str();

	// std::vector<CPattern *> patterns;

	std::string cmd = "read_verilog " + file_path;
	Yosys::run_pass(cmd);
	// Assuming the file has been read and patterns are extracted


	// Read the file and extract patterns
	// This is a placeholder implementation
	// You need to replace it with your actual file reading and pattern extraction logic

	return patterns;
}