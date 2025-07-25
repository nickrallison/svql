#include "SvqlPatPass.hpp"

#include <fstream>
#include <regex>

#include "kernel/register.h"
#include "kernel/log.h"
#include "kernel/sigtools.h"

#include "svql_common_pattern.h"

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

	if (module_name.empty())
	{
		log("SVQL_PAT_ERROR: No module name specified. Use -module <module_name>\n");
		log_error("No module name specified. Use -module <module_name>\n");
	}

	if (module_name[0] != '\\')
	{
		module_name = "\\" + module_name; // Ensure module name starts with a backslash
	}

	// Find the module in the design
	RTLIL::Module *module = design->module(RTLIL::IdString(module_name));
	if (!module)
	{
		log("SVQL_PAT_ERROR: Module '%s' not found in design\n", module_name.c_str());
		log_error("Module '%s' not found in design\n", module_name.c_str());
	}

	// Collect port names by type
	Pattern pattern = Pattern();
	pattern.file_loc = pattern_file;

	// Iterate through module ports to categorize them
	for (auto &port_name : module->ports)
	{
		RTLIL::Wire *wire = module->wire(port_name);
		if (!wire)
			continue;

		std::string port_str = port_name.str();

		if (wire->port_input)
		{
			// Input port
			pattern.in_ports.emplace_back(port_str);
		}
		else if (wire->port_output)
		{
			// Output port
			pattern.out_ports.emplace_back(port_str);
		}
		else if (wire->port_input && wire->port_output)
          		{
          			// Inout port
          			pattern.inout_ports.emplace_back(port_str);
          		}
	}

	rust::String json_str = pattern_into_json_string(pattern);
	log("SVQL_PAT_JSON_BEGIN\n%s\nSVQL_PAT_JSON_END\n", json_str.c_str());
	log_pop();
}