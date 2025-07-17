#include "SvqlPass.hpp"

#include <variant>
#include <algorithm>
#include <cstring>
#include <fstream>
#include <regex>
#include <set>

#include "kernel/log.h"
#include "kernel/sigtools.h"
#include "libs/subcircuit/subcircuit.h"

#include "SubCircuitReSolver.hpp"
#include "GraphConversion.hpp"
#include "svql_common.h"

using namespace svql;
using namespace Yosys;

std::vector<RTLIL::Wire *> svql::get_cell_wires(RTLIL::Cell *cell)
{
	std::vector<RTLIL::Wire *> wires;

	std::vector<std::pair<RTLIL::IdString, RTLIL::SigSpec>> sorted_connections;
	for (const auto &conn : cell->connections())
	{
		sorted_connections.emplace_back(conn.first, conn.second);
	}

	std::sort(sorted_connections.begin(), sorted_connections.end(),
			  [](const auto &a, const auto &b)
			  {
				  return strcmp(a.first.c_str(), b.first.c_str()) < 0;
			  });

	for (const auto &conn : sorted_connections)
	{
		for (const RTLIL::SigBit &bit : conn.second)
		{
			if (bit.is_wire() && bit.wire != nullptr)
			{
				wires.emplace_back(bit.wire);
			}
		}
	}

	return wires;
}

void svql::print_wire(RTLIL::Wire *wire)
{
	std::string output = "wire ";
	if (wire->width != 1)
		output += "width " + std::to_string(wire->width) + " ";
	if (wire->upto)
		output += "upto ";
	if (wire->start_offset != 0)
		output += "offset " + std::to_string(wire->start_offset) + " ";
	if (wire->port_input && !wire->port_output)
		output += "input " + std::to_string(wire->port_id) + " ";
	if (!wire->port_input && wire->port_output)
		output += "output " + std::to_string(wire->port_id) + " ";
	if (wire->port_input && wire->port_output)
		output += "inout " + std::to_string(wire->port_id) + " ";
	if (wire->is_signed)
		output += "signed ";
	output += wire->name.c_str();
	output += ":";
	// int to string
	output += std::to_string(wire->name.index_);
	log("%s\n", output.c_str());
}

SvqlPass::SvqlPass() : Pass("svql_driver", "find subcircuits and replace them with cells") {}

void SvqlPass::help()
{
	log("\n");
	log("    svql_driver -pat <pat_file> <pat_module_name> [options] [selection]\n");
	log("\n");
	log("This pass looks for subcircuits that are isomorphic to any of the modules\n");
	log("in the given map file.\n");
	log("map file can be a Verilog source file (*.v) or an RTLIL source file (*.il).\n");
	log("\n");
	log("    -pat <pat_file> <pat_module_name>\n");
	log("        use the modules in this file as reference. This option can be used\n");
	log("        multiple times.\n");
	log("\n");
	log("    -verbose\n");
	log("        print debug output while analyzing\n");
	log("\n");
	log("    -constports\n");
	log("        also find instances with constant drivers. this may be much\n");
	log("        slower than the normal operation.\n");
	log("\n");
	log("    -nodefaultswaps\n");
	log("        normally builtin port swapping rules for internal cells are used per\n");
	log("        default. This turns that off, so e.g. 'a^b' does not match 'b^a'\n");
	log("        when this option is used.\n");
	log("\n");
	log("    -compat <needle_type> <haystack_type>\n");
	log("        Per default, the cells in the map file (needle) must have the\n");
	log("        type as the cells in the active design (haystack). This option\n");
	log("        can be used to register additional pairs of types that should\n");
	log("        match. This option can be used multiple times.\n");
	log("\n");
	log("    -swap <needle_type> <port1>,<port2>[,...]\n");
	log("        Register a set of swappable ports for a needle cell type.\n");
	log("        This option can be used multiple times.\n");
	log("\n");
	log("    -perm <needle_type> <port1>,<port2>[,...] <portA>,<portB>[,...]\n");
	log("        Register a valid permutation of swappable ports for a needle\n");
	log("        cell type. This option can be used multiple times.\n");
	log("\n");
	log("    -cell_attr <attribute_name>\n");
	log("        Attributes on cells with the given name must match.\n");
	log("\n");
	log("    -wire_attr <attribute_name>\n");
	log("        Attributes on wires with the given name must match.\n");
	log("\n");
	log("    -ignore_parameters\n");
	log("        Do not use parameters when matching cells.\n");
	log("\n");
	log("    -ignore_param <cell_type> <parameter_name>\n");
	log("        Do not use this parameter when matching cells.\n");
	log("\n");
	log("This pass does not operate on modules with unprocessed processes in it.\n");
	log("(I.e. the 'proc' pass should be used first to convert processes to netlists.)\n");
	log("\n");
	log("This pass can also be used for mining for frequent subcircuits. In this mode\n");
	log("the following options are to be used instead of the -map option.\n");
	log("\n");
	log("The modules in the map file may have the attribute 'extract_order' set to an\n");
	log("integer value. Then this value is used to determine the order in which the pass\n");
	log("tries to map the modules to the design (ascending, default value is 0).\n");
	log("\n");
	log("See 'help techmap' for a pass that does the opposite thing.\n");
	log("\n");
}

std::variant<RTLIL::Design *, std::string> SvqlPass::setup(SvqlConfig &config, std::string &pat_filename, std::string &pat_module_name)
{
	RTLIL::Design *needle_design = new RTLIL::Design;

	if (pat_filename.compare(0, 1, "%") == 0)
	{
		if (!saved_designs.count(pat_filename.substr(1)))
		{
			delete needle_design;
			std::string error_msg = "Saved design `" + pat_filename.substr(1) + "` not found.";
			return error_msg;
		}
		for (auto mod : saved_designs.at(pat_filename.substr(1))->modules())
			if (!needle_design->has(mod->name))
				needle_design->add(mod->clone());
	}
	else
	{
		std::ifstream f;
		rewrite_filename(pat_filename);
		f.open(pat_filename.c_str());
		if (f.fail())
		{
			delete needle_design;
			std::string error_msg = "Can't open map file `" + pat_filename + "`.";
			return error_msg;
		}
		Frontend::frontend_call(needle_design, &f, pat_filename, (pat_filename.size() > 3 && pat_filename.compare(pat_filename.size() - 3, std::string::npos, ".il") == 0 ? "rtlil" : "verilog"));
		f.close();

		if (pat_filename.size() <= 3 || pat_filename.compare(pat_filename.size() - 3, std::string::npos, ".il") != 0)
		{
			Pass::call(needle_design, "proc");
			Pass::call(needle_design, "opt_clean");
		}
	}
	return needle_design;
}

void SvqlPass::execute(std::vector<std::string> args, RTLIL::Design *design)
{
	log_header(design, "Executing SVQL DRIVER pass (find matching subcircuits).\n");
	log_push();

	size_t argidx;
	SvqlConfig config = configure(args, argidx);
	std::string pat_filename = config.pat_filename;
	std::string pat_module_name = config.pat_module_name;

	// std::string pat_filename = std::string("svql_query/verilog/and.v");
	// std::string pat_module_name = std::string("and_gate");

	extra_args(args, argidx, design);

	if (pat_filename.empty())
		log_cmd_error("Missing option -pat <verilog_or_rtlil_file>.\n");

	// Setup the needle design
	std::variant<RTLIL::Design *, std::string> setup_result = SvqlPass::setup(config, pat_filename, pat_module_name);
	if (std::holds_alternative<std::string>(setup_result))
	{
		log_error("Error setting up needle design: %s\n", std::get<std::string>(setup_result).c_str());
		return;
	}

	RTLIL::Design *needle_design = std::get<RTLIL::Design *>(setup_result);

	// Run the query
	CMatchList *cmatch_list = run_query(config, needle_design, design);

	if (cmatch_list != nullptr)
	{
		// Process and display results
		if (cmatch_list->len > 0)
		{
			log("Found %zu matches.\n", cmatch_list->len);

			// Display matches with detailed information
			for (size_t i = 0; i < cmatch_list->len; i++)
			{
				if (cmatch_list->matches[i] != nullptr)
				{
					log("\nMatch #%zu:\n", i);

					// Serialize and display the match
					char *json_str = cmatch_serialize(cmatch_list->matches[i]);
					if (json_str != nullptr)
					{
						log("Match data: %s\n", json_str);
						free_json_string(json_str);
					}
				}
			}
		}
		else
		{
			log("No matches found.\n");
		}

		// Clean up
		cmatchlist_free(cmatch_list);
	}
	else
	{
		log("Query execution failed.\n");
	}

	// Clean up needle design
	if (needle_design != nullptr)
	{
		delete needle_design;
		needle_design = nullptr;
	}

	log_pop();
}

SvqlConfig SvqlPass::configure(std::vector<std::string> args, size_t &argidx)
{

	auto solver = std::make_unique<SubCircuitReSolver>();

	bool constports = false;
	bool nodefaultswaps = false;
	bool verbose = false;
	std::string pat_filename;
	std::string pat_module_name;

	for (argidx = 1; argidx < args.size(); argidx++)
	{

		if (args[argidx] == "-pat" && argidx + 2 < args.size())
		{
			pat_filename = args[++argidx];
			pat_module_name = RTLIL::escape_id(args[++argidx]);
			continue;
		}
		if (args[argidx] == "-verbose")
		{
			solver->setVerbose();
			continue;
		}
		if (args[argidx] == "-constports")
		{
			constports = true;
			continue;
		}
		if (args[argidx] == "-nodefaultswaps")
		{
			nodefaultswaps = true;
			continue;
		}
		if (args[argidx] == "-compat" && argidx + 2 < args.size())
		{
			std::string needle_type = RTLIL::escape_id(args[++argidx]);
			std::string haystack_type = RTLIL::escape_id(args[++argidx]);
			solver->addCompatibleTypes(needle_type, haystack_type);
			continue;
		}
		if (args[argidx] == "-swap" && argidx + 2 < args.size())
		{
			std::string type = RTLIL::escape_id(args[++argidx]);
			std::set<std::string> ports;
			std::string ports_str = args[++argidx], p;
			while (!(p = next_token(ports_str, ",\t\r\n ")).empty())
				ports.insert(RTLIL::escape_id(p));
			solver->addSwappablePorts(type, ports);
			continue;
		}
		if (args[argidx] == "-perm" && argidx + 3 < args.size())
		{
			std::string type = RTLIL::escape_id(args[++argidx]);
			std::vector<std::string> map_left, map_right;
			std::string left_str = args[++argidx];
			std::string right_str = args[++argidx], p;
			while (!(p = next_token(left_str, ",\t\r\n ")).empty())
				map_left.push_back(RTLIL::escape_id(p));
			while (!(p = next_token(right_str, ",\t\r\n ")).empty())
				map_right.push_back(RTLIL::escape_id(p));
			if (map_left.size() != map_right.size())
				log_cmd_error("Arguments to -perm are not a valid permutation!\n");
			std::map<std::string, std::string> map;
			for (size_t i = 0; i < map_left.size(); i++)
				map[map_left[i]] = map_right[i];
			std::sort(map_left.begin(), map_left.end());
			std::sort(map_right.begin(), map_right.end());
			if (map_left != map_right)
				log_cmd_error("Arguments to -perm are not a valid permutation!\n");
			solver->addSwappablePortsPermutation(type, map);
			continue;
		}
		if (args[argidx] == "-cell_attr" && argidx + 1 < args.size())
		{
			solver->cell_attr.insert(RTLIL::escape_id(args[++argidx]));
			continue;
		}
		if (args[argidx] == "-wire_attr" && argidx + 1 < args.size())
		{
			solver->wire_attr.insert(RTLIL::escape_id(args[++argidx]));
			continue;
		}
		if (args[argidx] == "-ignore_parameters")
		{
			solver->ignoreParameters = true;
			continue;
		}
		if (args[argidx] == "-ignore_param" && argidx + 2 < args.size())
		{
			solver->ignoredParams.insert(std::pair<RTLIL::IdString, RTLIL::IdString>(RTLIL::escape_id(args[argidx + 1]), RTLIL::escape_id(args[argidx + 2])));
			argidx += 2;
			continue;
		}
		break;
	}

	if (!nodefaultswaps)
	{
		solver->addSwappablePorts("$and", "\\A", "\\B");
		solver->addSwappablePorts("$or", "\\A", "\\B");
		solver->addSwappablePorts("$xor", "\\A", "\\B");
		solver->addSwappablePorts("$xnor", "\\A", "\\B");
		solver->addSwappablePorts("$eq", "\\A", "\\B");
		solver->addSwappablePorts("$ne", "\\A", "\\B");
		solver->addSwappablePorts("$eqx", "\\A", "\\B");
		solver->addSwappablePorts("$nex", "\\A", "\\B");
		solver->addSwappablePorts("$add", "\\A", "\\B");
		solver->addSwappablePorts("$mul", "\\A", "\\B");
		solver->addSwappablePorts("$logic_and", "\\A", "\\B");
		solver->addSwappablePorts("$logic_or", "\\A", "\\B");
		solver->addSwappablePorts("$_AND_", "\\A", "\\B");
		solver->addSwappablePorts("$_OR_", "\\A", "\\B");
		solver->addSwappablePorts("$_XOR_", "\\A", "\\B");
	}

	if (verbose)
	{
		solver->setVerbose();
	}

	SvqlConfig config;
	config.solver = std::move(solver);
	config.pat_filename = pat_filename;
	config.pat_module_name = pat_module_name;
	config.constports = constports;
	config.nodefaultswaps = nodefaultswaps;
	config.verbose = verbose;

	return config;
}

SvqlConfig SvqlPass::configure(CConfig &ccfg)
{
	auto solver = std::make_unique<SubCircuitReSolver>();

	// Set basic configuration options
	bool constports = ccfg.const_ports;
	bool nodefaultswaps = ccfg.nodefaultswaps;
	bool verbose = ccfg.verbose;

	// Configure solver verbose mode
	if (verbose)
	{
		solver->setVerbose();
	}

	// Configure ignore parameters
	if (ccfg.ignore_parameters)
	{
		solver->ignoreParameters = true;
	}

	// Add compatible types
	for (uintptr_t i = 0; i < ccfg.compat_pairs_len; i++)
	{
		std::string needle_type = RTLIL::escape_id(ccfg.compat_pairs_ptr[i].first);
		std::string haystack_type = RTLIL::escape_id(ccfg.compat_pairs_ptr[i].second);
		solver->addCompatibleTypes(needle_type, haystack_type);
	}

	// Add swappable ports
	for (uintptr_t i = 0; i < ccfg.swap_ports_len; i++)
	{
		std::string type = RTLIL::escape_id(ccfg.swap_ports_ptr[i].key);
		std::set<std::string> ports;
		for (uintptr_t j = 0; j < ccfg.swap_ports_ptr[i].values_len; j++)
		{
			ports.insert(RTLIL::escape_id(ccfg.swap_ports_ptr[i].values_ptr[j]));
		}
		solver->addSwappablePorts(type, ports);
	}

	// Add permutation ports
	for (uintptr_t i = 0; i < ccfg.perm_ports_len; i++)
	{
		std::string type = RTLIL::escape_id(ccfg.perm_ports_ptr[i].key);
		std::vector<std::string> map_left, map_right;

		for (uintptr_t j = 0; j < ccfg.perm_ports_ptr[i].first_values_len; j++)
		{
			map_left.push_back(RTLIL::escape_id(ccfg.perm_ports_ptr[i].first_values_ptr[j]));
		}

		for (uintptr_t j = 0; j < ccfg.perm_ports_ptr[i].second_values_len; j++)
		{
			map_right.push_back(RTLIL::escape_id(ccfg.perm_ports_ptr[i].second_values_ptr[j]));
		}

		if (map_left.size() != map_right.size())
		{
			log_cmd_error("Arguments to -perm are not a valid permutation!\n");
		}

		std::map<std::string, std::string> map;
		for (size_t j = 0; j < map_left.size(); j++)
		{
			map[map_left[j]] = map_right[j];
		}

		// Validate permutation
		std::sort(map_left.begin(), map_left.end());
		std::sort(map_right.begin(), map_right.end());
		if (map_left != map_right)
		{
			log_cmd_error("Arguments to -perm are not a valid permutation!\n");
		}

		solver->addSwappablePortsPermutation(type, map);
	}

	// Add cell attributes
	for (uintptr_t i = 0; i < ccfg.cell_attr_len; i++)
	{
		solver->cell_attr.insert(RTLIL::escape_id(ccfg.cell_attr_ptr[i]));
	}

	// Add wire attributes
	for (uintptr_t i = 0; i < ccfg.wire_attr_len; i++)
	{
		solver->wire_attr.insert(RTLIL::escape_id(ccfg.wire_attr_ptr[i]));
	}

	// Add ignored parameters
	for (uintptr_t i = 0; i < ccfg.ignore_param_len; i++)
	{
		solver->ignoredParams.insert(std::pair<RTLIL::IdString, RTLIL::IdString>(
			RTLIL::escape_id(ccfg.ignore_param_ptr[i].first),
			RTLIL::escape_id(ccfg.ignore_param_ptr[i].second)));
	}

	// Add default swappable ports if not disabled
	if (!nodefaultswaps)
	{
		solver->addSwappablePorts("$and", "\\A", "\\B");
		solver->addSwappablePorts("$or", "\\A", "\\B");
		solver->addSwappablePorts("$xor", "\\A", "\\B");
		solver->addSwappablePorts("$xnor", "\\A", "\\B");
		solver->addSwappablePorts("$eq", "\\A", "\\B");
		solver->addSwappablePorts("$ne", "\\A", "\\B");
		solver->addSwappablePorts("$eqx", "\\A", "\\B");
		solver->addSwappablePorts("$nex", "\\A", "\\B");
		solver->addSwappablePorts("$add", "\\A", "\\B");
		solver->addSwappablePorts("$mul", "\\A", "\\B");
		solver->addSwappablePorts("$logic_and", "\\A", "\\B");
		solver->addSwappablePorts("$logic_or", "\\A", "\\B");
		solver->addSwappablePorts("$_AND_", "\\A", "\\B");
		solver->addSwappablePorts("$_OR_", "\\A", "\\B");
		solver->addSwappablePorts("$_XOR_", "\\A", "\\B");
	}

	// Create and return the SvqlConfig
	SvqlConfig config;
	config.solver = std::move(solver);
	config.pat_filename = "";	 // Not used in this context
	config.pat_module_name = ""; // Not used in this context
	config.constports = constports;
	config.nodefaultswaps = nodefaultswaps;
	config.verbose = verbose;

	return config;
}

CMatchList *SvqlPass::run_query(const SvqlConfig &config, RTLIL::Design *needle_design, RTLIL::Design *design)
{
	if (needle_design == nullptr)
	{
		log_error("Needle design is not set up. Call setup() before running queries.\n");
		return nullptr;
	}

	if (design == nullptr)
	{
		log_error("Design is not set. Call execute() with a valid design first.\n");
		return nullptr;
	}

	// Get the needle module from the design
	RTLIL::Module *needle = needle_design->module(config.pat_module_name);
	if (needle == nullptr)
	{
		log_error("Module %s not found in needle design.\n", config.pat_module_name.c_str());
		return nullptr;
	}

	// Setting up the graph solver
	std::map<std::string, RTLIL::Module *> needle_map, haystack_map;
	std::set<RTLIL::IdString> needle_ports;

	// Get needle ports
	std::vector<RTLIL::IdString> ports = needle->ports;
	for (auto &port : ports)
	{
		needle_ports.insert(port);
	}

	// Use the solver from the config (make a copy for this query)
	SubCircuitReSolver *solver = config.solver.get();

	// Create Needle Graph
	SubCircuit::Graph mod_graph;
	std::string graph_name = "needle_" + RTLIL::unescape_id(needle->name);
	log("Creating needle graph %s.\n", graph_name.c_str());
	if (module2graph(mod_graph, needle, config.constports))
	{
		solver->addGraph(graph_name, mod_graph);
		needle_map[graph_name] = needle;
	}

	// Create haystack graphs from the main design
	for (auto module : design->modules())
	{
		SubCircuit::Graph mod_graph;
		std::string graph_name = "haystack_" + RTLIL::unescape_id(module->name);
		log("Creating haystack graph %s.\n", graph_name.c_str());
		if (module2graph(mod_graph, module, config.constports, design, -1, nullptr))
		{
			solver->addGraph(graph_name, mod_graph);
			haystack_map[graph_name] = module;
		}
	}

	// Run the solver
	std::vector<SubCircuit::Solver::Result> results;
	log_header(design, "Running solver from SubCircuit library.\n");

	for (auto &haystack_it : haystack_map)
	{
		log("Solving for %s in %s.\n", ("needle_" + RTLIL::unescape_id(needle->name)).c_str(), haystack_it.first.c_str());
		solver->solve(results, "needle_" + RTLIL::unescape_id(needle->name), haystack_it.first, false);
	}

	// log("Found %d matches.\n", GetSize(results));

	// Create CMatchList to return
	CMatchList *cmatch_list = cmatchlist_new();

	if (results.size() > 0)
	{
		for (int i = 0; i < int(results.size()); i++)
		{
			auto &result = results[i];

			// Create a new CMatch
			CMatch *cmatch = cmatch_new();

			for (const auto &it : result.mappings)
			{
				auto *graphCell = static_cast<RTLIL::Cell *>(it.second.haystackUserData);
				auto *needleCell = static_cast<RTLIL::Cell *>(it.second.needleUserData);

				std::string needle_name = escape_needle_name(needleCell->name.str());
				std::string haystack_name = escape_needle_name(graphCell->name.str());
				int needle_id = needleCell->name.index_;
				int haystack_id = graphCell->name.index_;

				CCellData *needle_cell_data = ccelldata_new(needle_name.c_str(), needle_id);
				CCellData *haystack_cell_data = ccelldata_new(haystack_name.c_str(), haystack_id);

				// Add cell data to the CMatch
				cmatch_add_celldata(cmatch, needle_cell_data, haystack_cell_data);

				// Get cell connections
				std::vector<RTLIL::Wire *> needle_cell_connections = get_cell_wires(needleCell);
				std::vector<RTLIL::Wire *> haystack_cell_connections = get_cell_wires(graphCell);

				// Create port mappings
				std::vector<std::pair<RTLIL::Wire *, RTLIL::Wire *>> connections;
				for (size_t j = 0; j < std::min(needle_cell_connections.size(), haystack_cell_connections.size()); j++)
				{
					connections.emplace_back(needle_cell_connections[j], haystack_cell_connections[j]);
				}

				// Log port mappings
				for (const auto &pair : connections)
				{
					if (needle_ports.find(pair.first->name) != needle_ports.end())
					{
						// log("Needle port %s mapped to haystack wire %s.\n",
						// 	pair.first->name.c_str(), pair.second->name.c_str());
						cmatch_add_port(cmatch, pair.first->name.c_str(), pair.second->name.c_str());
					}
				}
			}

			// Add the match to the list
			cmatchlist_add_match(cmatch_list, cmatch);
		}
	}

	return cmatch_list;
}

std::string svql::escape_needle_name(const std::string &name)
{
	if (name.compare(0, 7, "needle_") == 0)
	{
		return name.substr(7);
	}

	if (name.compare(0, 8, "haystack_") == 0)
	{
		return name.substr(8);
	}
	return name;
}

SvqlConfig svql::into_svql_runtime_config(const CSvqlRuntimeConfig &ccfg)
{
	SvqlConfig config;

	config.pat_module_name = ccfg.pat_module_name ? std::string(ccfg.pat_module_name) : "";
	config.pat_filename = ccfg.pat_filename ? std::string(ccfg.pat_filename) : "";
	config.verbose = ccfg.verbose;
	config.constports = ccfg.const_ports;
	config.nodefaultswaps = ccfg.nodefaultswaps;
	config.ignore_parameters = ccfg.ignore_parameters;

	// Convert compat_pairs
	for (uintptr_t i = 0; i < ccfg.compat_pairs_len; i++)
	{
		config.compat_pairs.emplace_back(
			ccfg.compat_pairs_ptr[i].first ? std::string(ccfg.compat_pairs_ptr[i].first) : "",
			ccfg.compat_pairs_ptr[i].second ? std::string(ccfg.compat_pairs_ptr[i].second) : "");
	}

	// Convert swap_ports
	for (uintptr_t i = 0; i < ccfg.swap_ports_len; i++)
	{
		std::set<std::string> ports;
		for (uintptr_t j = 0; j < ccfg.swap_ports_ptr[i].values_len; j++)
		{
			ports.insert(ccfg.swap_ports_ptr[i].values_ptr[j] ? std::string(ccfg.swap_ports_ptr[i].values_ptr[j]) : "");
		}
		config.swap_ports.emplace_back(
			ccfg.swap_ports_ptr[i].key ? std::string(ccfg.swap_ports_ptr[i].key) : "",
			ports);
	}

	// Convert perm_ports
	for (uintptr_t i = 0; i < ccfg.perm_ports_len; i++)
	{
		std::vector<std::string> perm_ports_vec;
		for (uintptr_t j = 0; j < ccfg.perm_ports_ptr[i].first_values_len; j++)
		{
			perm_ports_vec.push_back(ccfg.perm_ports_ptr[i].first_values_ptr[j] ? std::string(ccfg.perm_ports_ptr[i].first_values_ptr[j]) : "");
		}
		for (uintptr_t j = 0; j < ccfg.perm_ports_ptr[i].second_values_len; j++)
		{
			perm_ports_vec.push_back(ccfg.perm_ports_ptr[i].second_values_ptr[j] ? std::string(ccfg.perm_ports_ptr[i].second_values_ptr[j]) : "");
		}
		config.perm_ports.emplace_back(
			ccfg.perm_ports_ptr[i].key ? std::string(ccfg.perm_ports_ptr[i].key) : "",
			perm_ports_vec);
	}

	// Convert cell_attr
	for (uintptr_t i = 0; i < ccfg.cell_attr_len; i++)
	{
		config.cell_attr.push_back(ccfg.cell_attr_ptr[i] ? std::string(ccfg.cell_attr_ptr[i]) : "");
	}

	// Convert wire_attr
	for (uintptr_t i = 0; i < ccfg.wire_attr_len; i++)
	{
		config.wire_attr.push_back(ccfg.wire_attr_ptr[i] ? std::string(ccfg.wire_attr_ptr[i]) : "");
	}

	// Convert ignore_param
	for (uintptr_t i = 0; i < ccfg.ignore_param_len; i++)
	{
		config.ignore_param.emplace_back(
			ccfg.ignore_param_ptr[i].first ? std::string(ccfg.ignore_param_ptr[i].first) : "",
			ccfg.ignore_param_ptr[i].second ? std::string(ccfg.ignore_param_ptr[i].second) : "");
	}

	return config;
}

CSvqlRuntimeConfig svql::into_c_svql_runtime_config(const SvqlConfig &config)
{
	CSvqlRuntimeConfig runtime_config;

	runtime_config.pat_module_name = config.pat_module_name.c_str();
	runtime_config.pat_filename = config.pat_filename.c_str();
	runtime_config.verbose = config.verbose;
	runtime_config.const_ports = config.constports;
	runtime_config.nodefaultswaps = config.nodefaultswaps;
	runtime_config.ignore_parameters = config.ignore_parameters;

	// Convert compat_pairs
	runtime_config.compat_pairs = config.compat_pairs;

	// Convert swap_ports
	for (const auto &swap_port : config.swap_ports)
	{
		std::vector<std::string> ports_vec(swap_port.second.begin(), swap_port.second.end());
		runtime_config.swap_ports.emplace_back(swap_port.first, ports_vec);
	}

	// Convert perm_ports - this is tricky as we need to reconstruct the pair structure
	for (const auto &perm_port : config.perm_ports)
	{
		size_t half_size = perm_port.second.size() / 2;
		std::vector<std::string> first_half(perm_port.second.begin(), perm_port.second.begin() + half_size);
		std::vector<std::string> second_half(perm_port.second.begin() + half_size, perm_port.second.end());
		runtime_config.perm_ports.emplace_back(perm_port.first, std::make_pair(first_half, second_half));
	}

	// Convert cell_attr
	runtime_config.cell_attr = config.cell_attr;

	// Convert wire_attr
	runtime_config.wire_attr = config.wire_attr;

	// Convert ignore_param
	runtime_config.ignore_param = config.ignore_param;

	return runtime_config;
}