#include "SvqlPatPass.hpp"

#include <fstream>
#include <regex>

#include "kernel/register.h"
#include "kernel/log.h"
#include "kernel/sigtools.h"

#include "svql_common.h"

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
	std::vector<std::string> input_ports;
	std::vector<std::string> output_ports;
	std::vector<std::string> inout_ports;

	// Iterate through module ports to categorize them
	for (auto &port_name : module->ports)
	{
		RTLIL::Wire *wire = module->wire(port_name);
		if (!wire)
			continue;

		std::string port_str = port_name.str();

		if (wire->port_input && wire->port_output)
		{
			// Inout port
			inout_ports.push_back(port_str);
		}
		else if (wire->port_input)
		{
			// Input port
			input_ports.push_back(port_str);
		}
		else if (wire->port_output)
		{
			// Output port
			output_ports.push_back(port_str);
		}
	}

	List<CrateCString> input_ptrs;
	List<CrateCString> output_ptrs;
	List<CrateCString> inout_ptrs;

	for (const auto &port : input_ports)
	{
		CrateCString *port_cstr = crate_cstring_new(port.c_str());
		if (port_cstr == nullptr)
		{
			log("SVQL_PAT_ERROR: Failed to create CrateCString for input port '%s'\n", port.c_str());
			log_error("Failed to create CrateCString for input port '%s'\n", port.c_str());
		}
		else
		{
			string_list_append(&input_ptrs, *port_cstr);
			crate_cstring_destroy(port_cstr);
		}
	}
	for (const auto &port : output_ports)
	{
		CrateCString *port_cstr = crate_cstring_new(port.c_str());
		if (port_cstr == nullptr)
		{
			log("SVQL_PAT_ERROR: Failed to create CrateCString for output port '%s'\n", port.c_str());
			log_error("Failed to create CrateCString for output port '%s'\n", port.c_str());
		}
		else
		{
			string_list_append(&output_ptrs, *port_cstr);
			crate_cstring_destroy(port_cstr);
		}
	}
	for (const auto &port : inout_ports)
	{
		CrateCString *port_cstr = crate_cstring_new(port.c_str());
		if (port_cstr == nullptr)
		{
			log("SVQL_PAT_ERROR: Failed to create CrateCString for inout port '%s'\n", port.c_str());
			log_error("Failed to create CrateCString for inout port '%s'\n", port.c_str());
		}
		else
		{
			string_list_append(&inout_ptrs, *port_cstr);
			crate_cstring_destroy(port_cstr);
		}
	}

	CPattern *pattern = pattern_new();

	CrateCString *module_name_cstr = crate_cstring_new(module_name.c_str());

	pattern->file_loc = *module_name_cstr; // Use the module name as the file location
	crate_cstring_destroy(module_name_cstr);

	pattern->in_ports = input_ptrs;
	pattern->out_ports = output_ptrs;
	pattern->inout_ports = inout_ptrs;

	log("Created pattern for module '%s' with %zu input(s), %zu output(s), and %zu inout(s) ports\n",
		module_name.c_str(), input_ports.size(), output_ports.size(), inout_ports.size());

	CrateCString json_str = pattern_to_json(pattern);
	log("SVQL_PAT_JSON_BEGIN\n%s\nSVQL_PAT_JSON_END\n", json_str);
	crate_cstring_destroy(&json_str);
	// if (json_str)
	// {
	// 	log("SVQL_PAT_JSON_BEGIN\n%s\nSVQL_PAT_JSON_END\n", json_str);
	// 	crate_cstring_destroy(json_str);
	// }
	// else
	// {
	// 	log("SVQL_PAT_ERROR: Failed to serialize pattern to JSON\n");
	// }
	pattern_destroy(pattern);

	log_pop();
}