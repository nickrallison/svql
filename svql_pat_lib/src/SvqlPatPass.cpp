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

	if (module_name.empty()) {
		log_error("No module name specified. Use -module <module_name>\n");
	}

	if (module_name[0] != '\\') {
		module_name = "\\" + module_name; // Ensure module name starts with a backslash
	}

	// Find the module in the design
	RTLIL::Module *module = design->module(RTLIL::IdString(module_name));
	if (!module) {
		log_error("Module '%s' not found in design\n", module_name.c_str());
	}

	// Collect port names by type
	std::vector<std::string> input_ports;
	std::vector<std::string> output_ports;
	std::vector<std::string> inout_ports;

	// Iterate through module ports to categorize them
	for (auto &port_name : module->ports) {
		RTLIL::Wire *wire = module->wire(port_name);
		if (!wire) continue;

		std::string port_str = port_name.str();
		
		if (wire->port_input && wire->port_output) {
			// Inout port
			inout_ports.push_back(port_str);
		} else if (wire->port_input) {
			// Input port
			input_ports.push_back(port_str);
		} else if (wire->port_output) {
			// Output port
			output_ports.push_back(port_str);
		}
	}

	// Convert std::vector<std::string> to const char* arrays for C interface
	std::vector<const char*> input_ptrs;
	std::vector<const char*> output_ptrs;
	std::vector<const char*> inout_ptrs;

	for (const auto& port : input_ports) {
		input_ptrs.push_back(port.c_str());
	}
	for (const auto& port : output_ports) {
		output_ptrs.push_back(port.c_str());
	}
	for (const auto& port : inout_ports) {
		inout_ptrs.push_back(port.c_str());
	}

	// Create the CPattern
	CPattern *pattern = cpattern_new(
		pattern_file.c_str(),
		input_ptrs.empty() ? nullptr : input_ptrs.data(),
		input_ptrs.size(),
		output_ptrs.empty() ? nullptr : output_ptrs.data(),
		output_ptrs.size(),
		inout_ptrs.empty() ? nullptr : inout_ptrs.data(),
		inout_ptrs.size()
	);

	if (!pattern) {
		log_error("Failed to create pattern\n");
	}

	log("Created pattern for module '%s' with %zu input(s), %zu output(s), and %zu inout(s) ports\n",
		module_name.c_str(), input_ports.size(), output_ports.size(), inout_ports.size());

	// Serialize pattern to JSON and log it
	char *json_str = cpattern_to_json(pattern);
	if (json_str) {
		log("```\n%s\n```\n", json_str);
		cpattern_json_free(json_str);
	} else {
		log("Failed to serialize pattern to JSON\n");
	}

	// TODO: Further processing will be added later
	// For now, just clean up the pattern
	cpattern_free(pattern);

	log_pop();
}

// from svql_common.h
// struct CPattern *cpattern_new(const char *file_loc,
//                               const char *const *in_ports,
//                               uintptr_t in_ports_len,
//                               const char *const *out_ports,
//                               uintptr_t out_ports_len,
//                               const char *const *inout_ports,
//                               uintptr_t inout_ports_len);