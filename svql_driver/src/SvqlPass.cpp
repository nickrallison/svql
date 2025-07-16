#include "SvqlPass.hpp"

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
#include "RegexMap.hpp"
#include "detail.hpp"
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
	log("    -re <re_file>.json\n");
	log("        use a regex to match filenames\n");
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

	size_t argidx;

	SvqlConfig config = configure(args, design, argidx);
	auto &solver = *config.solver;
	std::string pat_filename = config.pat_filename;
	std::string pat_module_name = config.pat_module_name;
	bool constports = config.constports;
	bool nodefaultswaps = config.nodefaultswaps;
	bool verbose = config.verbose;

	extra_args(args, argidx, design);

	if (pat_filename.empty())
		log_cmd_error("Missing option -pat <verilog_or_rtlil_file>.\n");

	RTLIL::Design *map = nullptr;
	map = new RTLIL::Design;

	if (pat_filename.compare(0, 1, "%") == 0)
	{
		if (!saved_designs.count(pat_filename.substr(1)))
		{
			delete map;
			log_cmd_error("Can't saved design `%s'.\n", pat_filename.c_str() + 1);
		}
		for (auto mod : saved_designs.at(pat_filename.substr(1))->modules())
			if (!map->has(mod->name))
				map->add(mod->clone());
	}
	else
	{
		std::ifstream f;
		rewrite_filename(pat_filename);
		f.open(pat_filename.c_str());
		if (f.fail())
		{
			delete map;
			log_cmd_error("Can't open map file `%s'.\n", pat_filename.c_str());
		}
		Frontend::frontend_call(map, &f, pat_filename, (pat_filename.size() > 3 && pat_filename.compare(pat_filename.size() - 3, std::string::npos, ".il") == 0 ? "rtlil" : "verilog"));
		f.close();

		if (pat_filename.size() <= 3 || pat_filename.compare(pat_filename.size() - 3, std::string::npos, ".il") != 0)
		{
			Pass::call(map, "proc");
			Pass::call(map, "opt_clean");
		}
	}

	// Setting Up Pattern
	RTLIL::Module *needle = map->module(pat_module_name);
	std::vector<RTLIL::Wire *> pat_in_ports = std::vector<RTLIL::Wire *>();
	std::vector<RTLIL::Wire *> pat_out_ports = std::vector<RTLIL::Wire *>();
	std::vector<RTLIL::Wire *> pat_inout_ports = std::vector<RTLIL::Wire *>();
	std::vector<Match> matches = std::vector<Match>();

	for (auto wire : needle->wires())
	{
		print_wire(wire);
		if (wire->port_input && !wire->port_output)
		{
			pat_in_ports.push_back(wire);
		}
		if (!wire->port_input && wire->port_output)
		{
			pat_out_ports.push_back(wire);
		}
		if (wire->port_input && wire->port_output)
		{
			pat_inout_ports.push_back(wire);
		}
	}

	// Setting up the graph solver
	std::map<std::string, RTLIL::Module *> needle_map, haystack_map;
	std::set<RTLIL::IdString> needle_ports;

	log_header(design, "Creating graphs for SubCircuit library.\n");

	// #### Needle Ports
	std::vector<RTLIL::IdString> ports = needle->ports;
	for (auto &port : ports)
	{
		needle_ports.insert(port);
	}

	for (auto it = needle->connections().begin(); it != needle->connections().end(); ++it)
	{
		log("%s %s", it->first, it->second);
	}

	// #### Create Needle Graph
	SubCircuit::Graph mod_graph;
	std::string graph_name = "needle_" + RTLIL::unescape_id(needle->name);
	log("Creating needle graph %s.\n", graph_name.c_str());
	if (module2graph(mod_graph, needle, constports))
	{
		solver.addGraph(graph_name, mod_graph);
		needle_map[graph_name] = needle;
		// needle_list.push_back(module);
	}

	for (auto module : design->modules())
	{
		SubCircuit::Graph mod_graph;
		std::string graph_name = "haystack_" + RTLIL::unescape_id(module->name);
		log("Creating haystack graph %s.\n", graph_name.c_str());
		if (module2graph(mod_graph, module, constports, design, -1, nullptr))
		{
			solver.addGraph(graph_name, mod_graph);
			haystack_map[graph_name] = module;
		}
	}

	std::vector<SubCircuit::Solver::Result> results;
	log_header(design, "Running solver from SubCircuit library.\n");

	for (auto &haystack_it : haystack_map)
	{
		log("Solving for %s in %s.\n", ("needle_" + RTLIL::unescape_id(needle->name)).c_str(), haystack_it.first.c_str());
		solver.solve(results, "needle_" + RTLIL::unescape_id(needle->name), haystack_it.first, false);
	}

	log("Found %d matches.\n", GetSize(results));

	if (results.size() > 0)
	{
		// log_header(design, "Found SubCircuits.\n");

		for (int i = 0; i < int(results.size()); i++)
		{
			auto &result = results[i];

			for (const auto &it : result.mappings)
			{
				auto *graphCell = static_cast<RTLIL::Cell *>(it.second.haystackUserData);
				auto *needleCell = static_cast<RTLIL::Cell *>(it.second.needleUserData);

				std::vector<RTLIL::Wire *> needle_cell_connections = get_cell_wires(needleCell);
				std::vector<RTLIL::Wire *> haystack_cell_connections = get_cell_wires(graphCell);

				// sort by

				// zip together

				std::vector<std::pair<RTLIL::Wire *, RTLIL::Wire *>> connections;
				for (size_t j = 0; j < std::min(needle_cell_connections.size(), haystack_cell_connections.size()); j++)
				{
					connections.emplace_back(needle_cell_connections[j], haystack_cell_connections[j]);
				}

				for (const auto &pair : connections)
				{
					log("Needle cell %s has wire %s with id %d, Haystack cell %s has wire %s with id %d\n",
						needleCell->name.c_str(), pair.first->name.c_str(), pair.first->name.index_,
						graphCell->name.c_str(), pair.second->name.c_str(), pair.second->name.index_);
					print_wire(pair.first);
					print_wire(pair.second);

					if (pair.first->name != pair.second->name)
					{
						log("Mismatch in wire names: %s != %s\n", pair.first->name.c_str(), pair.second->name.c_str());
					}

					// if needle ports contains the port, then add a log statement
					if (needle_ports.find(pair.first->name) != needle_ports.end())
					{
						log("Needle port %s found in haystack cell %s.\n", pair.first->name.c_str(), graphCell->name.c_str());
					}
					else
					{
						log("Needle port %s not found in haystack cell %s.\n", pair.first->name.c_str(), graphCell->name.c_str());
					}
				}
			}
			log("\nMatch #%d: (%s in %s)\n", i, result.needleGraphId.c_str(), result.haystackGraphId.c_str());
			for (const auto &it : result.mappings)
			{
				auto *graphCell = static_cast<RTLIL::Cell *>(it.second.haystackUserData);
				CSourceLoc *source_loc = svql_source_loc_parse(graphCell->get_src_attribute().c_str(), '|');
				char *source_loc_str = svql_source_loc_to_json(source_loc);
				log("```\n%s\n```", source_loc_str);
				svql_free_string(source_loc_str);
				svql_source_loc_free(source_loc);
			}
		}
	}

	delete map;
	log_pop();
}

SvqlConfig SvqlPass::configure(std::vector<std::string> args, RTLIL::Design *design, size_t &argidx)
{

	auto solver = std::make_unique<SubCircuitReSolver>();

	std::string pat_filename;
	std::string pat_module_name;
	bool constports = false;
	bool nodefaultswaps = false;
	bool verbose = false;

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