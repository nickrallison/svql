#include "SvqlPass.hpp"

#include <optional>
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
#include "svql_common_config.h"
#include "svql_common_mat.h"

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
	log("    svql_driver -cmd [-pat <pat_file> <pat_module_name> [options] [selection]]\n");
	log("    or\n");
	log("    svql_driver -net [port]\n");
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
	log_header(design, "Executing SVQL DRIVER pass.\n");
	log_push();

	std::string error_msg;

	// 1. Parse args to config (Rust FFI)
	size_t argsidx = 1;

	std::string mode = args[1];
	if (mode == "-cmd")
	{
		log("Running in command mode.\n");
		execute_cmd(args, design);
	}
	else if (mode == "-net")
	{
		log("Running in network mode.\n");
		execute_net(args, design);
	}
	else
	{
		log_error("Invalid mode '%s'. Use '-cmd' or '-net'.\n", mode.c_str());
		return;
	}

	// std::optional<SvqlRuntimeConfig> cfg = parse_args_to_config(argsidx, args, error_msg);
	// if (!cfg)
	// {
	// 	log_error("Error parsing arguments: %s\n", error_msg.c_str());
	// 	return;
	// }
	// extra_args(args, argsidx, design);
	// SvqlRuntimeConfig &cfg_ref = *cfg;

	// // 2. Create solver
	// auto solver = create_solver(cfg_ref);

	// // 3. Setup needle design
	// RTLIL::Design *needle_design = setup_needle_design(cfg_ref, error_msg);
	// if (!needle_design)
	// {
	// 	log_error("Error setting up needle design: %s\n", error_msg.c_str());
	// 	return;
	// }

	// // 4. Run solver
	// std::optional<QueryMatchList> match_list = run_solver(solver.get(), cfg_ref, needle_design, design, error_msg);
	// if (!match_list)
	// {
	// 	log_error("Error running solver: %s\n", error_msg.c_str());
	// 	return;
	// }
	// QueryMatchList &match_list_ref = match_list.value();

	// // 5. Print results (as before)
	// rust::String json_str = matchlist_into_json_string(match_list_ref);
	// log("SVQL_MATCHES: %s\n", json_str.c_str());

	// // 6. Clean up
	// delete needle_design;
	log_pop();
}

void SvqlPass::execute_cmd(std::vector<std::string> args, RTLIL::Design *design)
{

	std::string error_msg;

	// 1. Parse args to config (Rust FFI)
	size_t argsidx = 1;
	args.erase(args.begin());
	std::optional<SvqlRuntimeConfig> cfg = parse_args_to_config(argsidx, args, error_msg);
	if (!cfg)
	{
		log_error("Error parsing arguments: %s\n", error_msg.c_str());
		return;
	}
	extra_args(args, argsidx, design);
	SvqlRuntimeConfig &cfg_ref = *cfg;

	// 2. Create solver
	auto solver = create_solver(cfg_ref);

	// 3. Setup needle design
	RTLIL::Design *needle_design = setup_needle_design(cfg_ref, error_msg);
	if (!needle_design)
	{
		log_error("Error setting up needle design: %s\n", error_msg.c_str());
		return;
	}

	// 4. Run solver
	std::optional<QueryMatchList> match_list = run_solver(solver.get(), cfg_ref, needle_design, design, error_msg);
	if (!match_list)
	{
		log_error("Error running solver: %s\n", error_msg.c_str());
		return;
	}
	QueryMatchList &match_list_ref = match_list.value();

	// 5. Print results (as before)
	rust::String json_str = matchlist_into_json_string(match_list_ref);
	log("SVQL_MATCHES: %s\n", json_str.c_str());

	// 6. Clean up
	delete needle_design;
}

void SvqlPass::execute_net(std::vector<std::string> args, RTLIL::Design *design)
{

	std::string error_msg;

	// 1. Parse args to config (Rust FFI)
	size_t argsidx = 1;
	std::optional<SvqlRuntimeConfig> cfg = parse_args_to_config(argsidx, args, error_msg);
	if (!cfg)
	{
		log_error("Error parsing arguments: %s\n", error_msg.c_str());
		return;
	}
	extra_args(args, argsidx, design);
	SvqlRuntimeConfig &cfg_ref = *cfg;

	// 2. Create solver
	auto solver = create_solver(cfg_ref);

	// 3. Setup needle design
	RTLIL::Design *needle_design = setup_needle_design(cfg_ref, error_msg);
	if (!needle_design)
	{
		log_error("Error setting up needle design: %s\n", error_msg.c_str());
		return;
	}

	// 4. Run solver
	std::optional<QueryMatchList> match_list = run_solver(solver.get(), cfg_ref, needle_design, design, error_msg);
	if (!match_list)
	{
		log_error("Error running solver: %s\n", error_msg.c_str());
		return;
	}
	QueryMatchList &match_list_ref = match_list.value();

	// 5. Print results (as before)
	rust::String json_str = matchlist_into_json_string(match_list_ref);
	log("SVQL_MATCHES: %s\n", json_str.c_str());

	// 6. Clean up
	delete needle_design;
}

std::optional<SvqlRuntimeConfig> SvqlPass::parse_args_to_config(size_t &argsidx, const std::vector<std::string> &args, std::string &error_msg)
{

	SvqlRuntimeConfig cfg = SvqlRuntimeConfig();

	for (argsidx = 1; argsidx < args.size(); argsidx++)
	{
		if (args[argsidx] == "-pat" && argsidx + 2 < args.size())
		{
			cfg.pat_module_name = args[++argsidx];
			cfg.pat_filename = args[++argsidx];
			continue;
		}
		if (args[argsidx] == "-verbose")
		{
			cfg.verbose = true;
			continue;
		}
		if (args[argsidx] == "-constports")
		{
			cfg.const_ports = true;
			continue;
		}
		if (args[argsidx] == "-nodefaultswaps")
		{
			cfg.nodefaultswaps = true;
			continue;
		}
		if (args[argsidx] == "-compat" && argsidx + 2 < args.size())
		{
			std::string needle_type = RTLIL::escape_id(args[++argsidx]);
			std::string haystack_type = RTLIL::escape_id(args[++argsidx]);

			CompatPair compat_pair;
			compat_pair.needle = needle_type;
			compat_pair.haystack = haystack_type;

			cfg.compat_pairs.emplace_back(compat_pair);
			continue;
		}
		if (args[argsidx] == "-swap" && argsidx + 2 < args.size())
		{
			std::string type = RTLIL::escape_id(args[++argsidx]);
			std::set<std::string> ports;
			std::string ports_str = args[++argsidx], p;
			while (!(p = next_token(ports_str, ",\t\r\n ")).empty())
				ports.insert(RTLIL::escape_id(p));

			SwapPort swap_port;
			swap_port.type_name = type;
			for (const auto &port : ports)
			{
				swap_port.ports.emplace_back(port);
			}
			cfg.swap_ports.emplace_back(swap_port);
			continue;
		}
		if (args[argsidx] == "-perm" && argsidx + 3 < args.size())
		{
			std::string type = RTLIL::escape_id(args[++argsidx]);
			std::vector<std::string> map_left, map_right;
			std::string left_str = args[++argsidx];
			std::string right_str = args[++argsidx], p;
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

			PermPort perm_port;
			perm_port.type_name = type;
			for (const auto &port : map_left)
				perm_port.left.emplace_back(port);
			for (const auto &wire : map_right)
				perm_port.right.emplace_back(wire);

			cfg.perm_ports.emplace_back(perm_port);
			continue;
		}
		if (args[argsidx] == "-cell_attr" && argsidx + 1 < args.size())
		{
			cfg.cell_attr.emplace_back(RTLIL::escape_id(args[++argsidx]));
			continue;
		}
		if (args[argsidx] == "-wire_attr" && argsidx + 1 < args.size())
		{
			cfg.wire_attr.emplace_back(RTLIL::escape_id(args[++argsidx]));
			continue;
		}
		if (args[argsidx] == "-ignore_parameters")
		{
			cfg.ignore_params = true;
			continue;
		}
		if (args[argsidx] == "-ignore_param" && argsidx + 2 < args.size())
		{
			IgnoreParam ignore_param;
			ignore_param.param_name = RTLIL::escape_id(args[++argsidx]);
			ignore_param.param_value = RTLIL::escape_id(args[++argsidx]);
			cfg.ignored_parameters.emplace_back(ignore_param);
			continue;
		}

		error_msg = "Unknown argument: " + args[argsidx];
		return std::nullopt;
	}

	return cfg;
}

std::unique_ptr<SubCircuitReSolver> SvqlPass::create_solver(const SvqlRuntimeConfig &cfg)
{
	auto solver = std::make_unique<SubCircuitReSolver>();

	if (cfg.verbose)
		solver->setVerbose();
	if (cfg.ignore_params)
		solver->ignoreParameters = true;

	for (size_t i = 0; i < cfg.compat_pairs.size(); ++i)
	{
		const auto &pair = cfg.compat_pairs[i];
		solver->addCompatibleTypes(pair.needle.operator std::string(), pair.haystack.operator std::string());
	}

	for (size_t i = 0; i < cfg.swap_ports.size(); ++i)
	{
		const auto &swap = cfg.swap_ports[i];
		std::set<std::string> ports;
		for (size_t j = 0; j < swap.ports.size(); ++j)
		{
			ports.insert(swap.ports[j].operator std::string());
		}
		solver->addSwappablePorts(swap.type_name.operator std::string(), ports);
	}

	for (size_t i = 0; i < cfg.perm_ports.size(); ++i)
	{
		const auto &perm = cfg.perm_ports[i];
		std::vector<std::string> left, right;
		for (size_t j = 0; j < perm.left.size(); ++j)
		{
			left.push_back(perm.left[j].operator std::string());
		}
		for (size_t j = 0; j < perm.right.size(); ++j)
		{
			right.push_back(perm.right[j].operator std::string());
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
		solver->addSwappablePortsPermutation(perm.type_name.operator std::string(), map);
	}

	for (size_t i = 0; i < cfg.cell_attr.size(); ++i)
	{
		solver->cell_attr.insert(cfg.cell_attr[i].operator std::string());
	}

	for (size_t i = 0; i < cfg.wire_attr.size(); ++i)
	{
		solver->wire_attr.insert(cfg.wire_attr[i].operator std::string());
	}

	for (size_t i = 0; i < cfg.ignored_parameters.size(); ++i)
	{
		const auto &ip = cfg.ignored_parameters[i];
		solver->ignoredParams.insert(std::make_pair(ip.param_name.operator std::string(), ip.param_value.operator std::string()));
	}

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

RTLIL::Design *SvqlPass::setup_needle_design(const SvqlRuntimeConfig &cfg, std::string &error_msg)
{
	RTLIL::Design *needle_design = new RTLIL::Design;
	std::string pat_filename = cfg.pat_filename.operator std::string();
	std::string pat_module_name = cfg.pat_module_name.operator std::string();

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

std::optional<QueryMatchList> SvqlPass::run_solver(SubCircuitReSolver *solver, const SvqlRuntimeConfig &cfg, RTLIL::Design *needle_design, RTLIL::Design *design, std::string &error_msg)
{
	if (needle_design == nullptr)
	{
		error_msg = "Needle design is not set up. Call setup() before running queries.";
		return std::nullopt;
	}

	if (design == nullptr)
	{
		error_msg = "Design is not set. Call execute() with a valid design first.";
		return std::nullopt;
	}

	std::string pat_module_name = cfg.pat_module_name.operator std::string();
	RTLIL::Module *needle = needle_design->module(pat_module_name);
	if (needle == nullptr)
	{
		error_msg = "Module " + pat_module_name + " not found in needle design.";
		return std::nullopt;
	}

	std::map<std::string, RTLIL::Module *> needle_map, haystack_map;
	std::set<RTLIL::IdString> needle_ports;

	std::vector<RTLIL::IdString> ports = needle->ports;
	for (auto &port : ports)
	{
		needle_ports.insert(port);
	}

	SubCircuit::Graph mod_graph;
	std::string graph_name = "needle_" + RTLIL::unescape_id(needle->name);
	log("Creating needle graph %s.\n", graph_name.c_str());
	if (module2graph(mod_graph, needle, cfg.const_ports))
	{
		solver->addGraph(graph_name, mod_graph);
		needle_map[graph_name] = needle;
	}

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

	std::vector<SubCircuit::Solver::Result> results;
	log_header(design, "Running solver from SubCircuit library.\n");

	for (auto &haystack_it : haystack_map)
	{
		log("Solving for %s in %s.\n", ("needle_" + RTLIL::unescape_id(needle->name)).c_str(), haystack_it.first.c_str());
		solver->solve(results, "needle_" + RTLIL::unescape_id(needle->name), haystack_it.first, false);
	}

	QueryMatchList matchlist = QueryMatchList();

	if (results.size() > 0)
	{
		for (int i = 0; i < int(results.size()); i++)
		{
			auto &result = results[i];

			QueryMatch match = QueryMatch();

			for (const auto &it : result.mappings)
			{
				auto *graphCell = static_cast<RTLIL::Cell *>(it.second.haystackUserData);
				auto *needleCell = static_cast<RTLIL::Cell *>(it.second.needleUserData);

				std::string needle_name = escape_needle_name(needleCell->name.str());
				std::string haystack_name = escape_needle_name(graphCell->name.str());
				int needle_id = needleCell->name.index_;
				int haystack_id = graphCell->name.index_;

				CellData needle_cell_data = CellData();
				needle_cell_data.cell_name = needle_name;
				needle_cell_data.cell_index = needle_id;

				CellData haystack_cell_data = CellData();
				haystack_cell_data.cell_name = haystack_name;
				haystack_cell_data.cell_index = haystack_id;

				CellPair cell_pair = CellPair();
				cell_pair.needle = needle_cell_data;
				cell_pair.haystack = haystack_cell_data;

				match.cell_map.emplace_back(cell_pair);

				std::vector<RTLIL::Wire *> needle_cell_connections = get_cell_wires(needleCell);
				std::vector<RTLIL::Wire *> haystack_cell_connections = get_cell_wires(graphCell);

				std::vector<std::pair<RTLIL::Wire *, RTLIL::Wire *>> connections;
				for (size_t j = 0; j < std::min(needle_cell_connections.size(), haystack_cell_connections.size()); j++)
				{
					connections.emplace_back(needle_cell_connections[j], haystack_cell_connections[j]);
				}

				for (const auto &pair : connections)
				{
					if (needle_ports.find(pair.first->name) != needle_ports.end())
					{

						StringPair port_pair = StringPair();
						port_pair.needle = pair.first->name.str();
						port_pair.haystack = pair.second->name.str();
						match.port_map.emplace_back(port_pair);
					}
				}
			}

			matchlist.matches.emplace_back(match);
		}
	}

	return matchlist;
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