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

void SvqlPass::execute(std::vector<std::string> args, RTLIL::Design *design)
{
	log_header(design, "Executing SVQL DRIVER pass (find matching subcircuits).\n");
	log_push();

	std::string error_msg;

	// 1. Parse args to config (Rust FFI)
	CSvqlRuntimeConfig cfg = parse_args_to_config(args);

	// 2. Create solver
	auto solver = create_solver(cfg);

	// 3. Setup needle design
	RTLIL::Design *needle_design = setup_needle_design(cfg, error_msg);
	if (!needle_design)
	{
		log_error("Error setting up needle design: %s\n", error_msg.c_str());
		return;
	}

	// 4. Run solver
	CMatchList *cmatch_list = run_solver(solver.get(), cfg, needle_design, design);

	// 5. Print results (as before)
	if (cmatch_list != nullptr)
	{
		CrateCString json_str = match_list_to_json(cmatch_list);
		log("SVQL_MATCHES: %s\n", json_str.string);
		// Dont need to free string since its dropped at the end of the function
		// crate_cstring_destroy(&json_str);
		match_list_destroy(cmatch_list); // Rust FFI cleanup
	}

	// 6. Clean up
	delete needle_design;
	log_pop();
}

CSvqlRuntimeConfig SvqlPass::parse_args_to_config(const std::vector<std::string> &args)
{
	std::vector<const char *> argv;
	for (const auto &s : args)
		argv.push_back(s.c_str());
	// Call the Rust FFI function
	return svql_runtime_config_from_args((int)argv.size(), argv.data());
}

std::unique_ptr<SubCircuitReSolver> SvqlPass::create_solver(const CSvqlRuntimeConfig &cfg)
{
	auto solver = std::make_unique<SubCircuitReSolver>();

	// Use the config to set up the solver (as in your old configure(CConfig&))
	if (cfg.verbose)
		solver->setVerbose();
	if (cfg.ignore_parameters)
		solver->ignoreParameters = true;

	// compat_pairs
	for (size_t i = 0; i < cfg.compat_pairs.items.len; ++i)
	{
		const auto &pair = cfg.compat_pairs.items.ptr[i];
		solver->addCompatibleTypes(pair.item1.string, pair.item2.string);
	}

	// swap_ports
	for (size_t i = 0; i < cfg.swap_ports.items.len; ++i)
	{
		const auto &swap = cfg.swap_ports.items.ptr[i];
		std::set<std::string> ports;
		for (size_t j = 0; j < swap.ports.items.len; ++j)
		{
			ports.insert(swap.ports.items.ptr[j].string);
		}
		solver->addSwappablePorts(swap.name.string, ports);
	}

	// perm_ports
	for (size_t i = 0; i < cfg.perm_ports.items.len; ++i)
	{
		const auto &perm = cfg.perm_ports.items.ptr[i];
		std::vector<std::string> left, right;
		for (size_t j = 0; j < perm.ports.items.len; ++j)
		{
			left.push_back(perm.ports.items.ptr[j].string);
		}
		for (size_t j = 0; j < perm.wires.items.len; ++j)
		{
			right.push_back(perm.wires.items.ptr[j].string);
		}
		if (left.size() != right.size())
		{
			log_cmd_error("Arguments to -perm are not a valid permutation!\n");
		}
		std::map<std::string, std::string> map;
		for (size_t j = 0; j < left.size(); ++j)
		{
			map[left[j]] = right[j];
		}
		std::vector<std::string> left_sorted = left, right_sorted = right;
		std::sort(left_sorted.begin(), left_sorted.end());
		std::sort(right_sorted.begin(), right_sorted.end());
		if (left_sorted != right_sorted)
		{
			log_cmd_error("Arguments to -perm are not a valid permutation!\n");
		}
		solver->addSwappablePortsPermutation(perm.name.string, map);
	}

	// cell_attr
	for (size_t i = 0; i < cfg.cell_attr.items.len; ++i)
	{
		solver->cell_attr.insert(cfg.cell_attr.items.ptr[i].string);
	}

	// wire_attr
	for (size_t i = 0; i < cfg.wire_attr.items.len; ++i)
	{
		solver->wire_attr.insert(cfg.wire_attr.items.ptr[i].string);
	}

	// ignore_param
	for (size_t i = 0; i < cfg.ignore_param.items.len; ++i)
	{
		const auto &ip = cfg.ignore_param.items.ptr[i];
		solver->ignoredParams.insert(std::make_pair(ip.name.string, ip.value.string));
	}

	// Default swappable ports
	if (!cfg.nodefaultswaps)
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

	return solver;
}

RTLIL::Design *SvqlPass::setup_needle_design(const CSvqlRuntimeConfig &cfg, std::string &error_msg)
{
	RTLIL::Design *needle_design = new RTLIL::Design;
	std::string pat_filename = cfg.pat_filename.string;
	std::string pat_module_name = cfg.pat_module_name.string;

	if (pat_filename.empty())
	{
		error_msg = "Missing pattern filename.";
		delete needle_design;
		return nullptr;
	}

	if (pat_filename.compare(0, 1, "%") == 0)
	{

		if (!saved_designs.count(pat_filename.substr(1)))
		{
			error_msg = "Saved design `" + pat_filename.substr(1) + "` not found.";
			delete needle_design;
			return nullptr;
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
			error_msg = "Can't open map file `" + pat_filename + "`.";
			delete needle_design;
			return nullptr;
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

CMatchList *SvqlPass::run_solver(SubCircuitReSolver *solver, const CSvqlRuntimeConfig &cfg, RTLIL::Design *needle_design, RTLIL::Design *design)
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
	std::string pat_module_name = cfg.pat_module_name.string;
	RTLIL::Module *needle = needle_design->module(pat_module_name);
	if (needle == nullptr)
	{
		log_error("Module %s not found in needle design.\n", pat_module_name.c_str());
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

	// Create Needle Graph
	SubCircuit::Graph mod_graph;
	std::string graph_name = "needle_" + RTLIL::unescape_id(needle->name);
	log("Creating needle graph %s.\n", graph_name.c_str());
	if (module2graph(mod_graph, needle, cfg.const_ports))
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
		if (module2graph(mod_graph, module, cfg.const_ports, design, -1, nullptr))
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
	CMatchList *cmatch_list = match_list_new();

	if (results.size() > 0)
	{
		for (int i = 0; i < int(results.size()); i++)
		{
			auto &result = results[i];

			// Create a new CMatch
			CMatch *cmatch = match_new();

			for (const auto &it : result.mappings)
			{
				auto *graphCell = static_cast<RTLIL::Cell *>(it.second.haystackUserData);
				auto *needleCell = static_cast<RTLIL::Cell *>(it.second.needleUserData);

				std::string needle_name = escape_needle_name(needleCell->name.str());
				std::string haystack_name = escape_needle_name(graphCell->name.str());
				int needle_id = needleCell->name.index_;
				int haystack_id = graphCell->name.index_;

				CCellData *needle_cell_data = ccelldata_new(crate_cstring_new(needle_name.c_str()), needle_id);
				CCellData *haystack_cell_data = ccelldata_new(crate_cstring_new(haystack_name.c_str()), haystack_id);

				// Add cell data to the CMatch
				match_add_celldata(cmatch, *needle_cell_data, *haystack_cell_data);

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
						match_add_portdata(cmatch, crate_cstring_new(pair.first->name.c_str()), crate_cstring_new(pair.second->name.c_str()));
					}
				}
			}

			append_match_to_matchlist(cmatch_list, *cmatch);
			// match_destroy(cmatch);
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