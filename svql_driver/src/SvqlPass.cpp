#include "SvqlPass.hpp"

#include <fstream>
#include <regex>

#include "kernel/log.h"
#include "kernel/sigtools.h"
#include "subcircuit.h"

#include "SubCircuitReSolver.hpp"
#include "GraphConversion.hpp"
#include "RegexMap.hpp"
#include "detail.hpp"
// #include "SourceLoc.h"

using namespace svql;
using namespace Yosys;

SvqlPass::SvqlPass() : Pass("svql", "find subcircuits and replace them with cells") {}

void SvqlPass::help()
{
	log("\n");
	log("    svql -map <map_file> [options] [selection]\n");
	log("\n");
	log("This pass looks for subcircuits that are isomorphic to any of the modules\n");
	log("in the given map file.\n");
	log("map file can be a Verilog source file (*.v) or an RTLIL source file (*.il).\n");
	log("\n");
	log("    -map <map_file>\n");
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
	log_header(design, "Executing SVQL pass (find matching subcircuits).\n");
	log_push();

	SubCircuitReSolver solver;

	std::vector<std::string> map_filenames;
	std::vector<std::string> regex_filenames;
	std::map<std::string, std::map<RTLIL::IdString, std::regex>> map_regexes;
	bool constports = false;
	bool nodefaultswaps = false;
	bool verbose = false;

	size_t argidx;
	for (argidx = 1; argidx < args.size(); argidx++)
	{

		if (args[argidx] == "-map" && argidx + 1 < args.size())
		{
			map_filenames.push_back(args[++argidx]);
			continue;
		}

		if (args[argidx] == "-re" && argidx + 1 < args.size())
		{
			regex_filenames.push_back(args[++argidx]);
			continue;
		}
		if (args[argidx] == "-verbose")
		{
			solver.setVerbose();
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
			solver.addCompatibleTypes(needle_type, haystack_type);
			continue;
		}
		if (args[argidx] == "-swap" && argidx + 2 < args.size())
		{
			std::string type = RTLIL::escape_id(args[++argidx]);
			std::set<std::string> ports;
			std::string ports_str = args[++argidx], p;
			while (!(p = next_token(ports_str, ",\t\r\n ")).empty())
				ports.insert(RTLIL::escape_id(p));
			solver.addSwappablePorts(type, ports);
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
			solver.addSwappablePortsPermutation(type, map);
			continue;
		}
		if (args[argidx] == "-cell_attr" && argidx + 1 < args.size())
		{
			solver.cell_attr.insert(RTLIL::escape_id(args[++argidx]));
			continue;
		}
		if (args[argidx] == "-wire_attr" && argidx + 1 < args.size())
		{
			solver.wire_attr.insert(RTLIL::escape_id(args[++argidx]));
			continue;
		}
		if (args[argidx] == "-ignore_parameters")
		{
			solver.ignoreParameters = true;
			continue;
		}
		if (args[argidx] == "-ignore_param" && argidx + 2 < args.size())
		{
			solver.ignoredParams.insert(std::pair<RTLIL::IdString, RTLIL::IdString>(RTLIL::escape_id(args[argidx + 1]), RTLIL::escape_id(args[argidx + 2])));
			argidx += 2;
			continue;
		}
		break;
	}

	extra_args(args, argidx, design);

	if (!nodefaultswaps)
	{
		solver.addSwappablePorts("$and", "\\A", "\\B");
		solver.addSwappablePorts("$or", "\\A", "\\B");
		solver.addSwappablePorts("$xor", "\\A", "\\B");
		solver.addSwappablePorts("$xnor", "\\A", "\\B");
		solver.addSwappablePorts("$eq", "\\A", "\\B");
		solver.addSwappablePorts("$ne", "\\A", "\\B");
		solver.addSwappablePorts("$eqx", "\\A", "\\B");
		solver.addSwappablePorts("$nex", "\\A", "\\B");
		solver.addSwappablePorts("$add", "\\A", "\\B");
		solver.addSwappablePorts("$mul", "\\A", "\\B");
		solver.addSwappablePorts("$logic_and", "\\A", "\\B");
		solver.addSwappablePorts("$logic_or", "\\A", "\\B");
		solver.addSwappablePorts("$_AND_", "\\A", "\\B");
		solver.addSwappablePorts("$_OR_", "\\A", "\\B");
		solver.addSwappablePorts("$_XOR_", "\\A", "\\B");
	}

	if (map_filenames.empty())
		log_cmd_error("Missing option -map <verilog_or_rtlil_file>.\n");

	for (auto &filename : regex_filenames)
	{
		std::map<std::string, std::map<RTLIL::IdString, std::pair<std::regex, std::string>>> regex_map = load_regex_map(filename);
		solver.joinRegexMap(regex_map);
	}

	if (verbose)
	{
		solver.setVerbose();
	}

	RTLIL::Design *map = nullptr;
	map = new RTLIL::Design;
	for (auto &filename : map_filenames)
	{
		if (filename.compare(0, 1, "%") == 0)
		{
			if (!saved_designs.count(filename.substr(1)))
			{
				delete map;
				log_cmd_error("Can't saved design `%s'.\n", filename.c_str() + 1);
			}
			for (auto mod : saved_designs.at(filename.substr(1))->modules())
				if (!map->has(mod->name))
					map->add(mod->clone());
		}
		else
		{
			std::ifstream f;
			rewrite_filename(filename);
			f.open(filename.c_str());
			if (f.fail())
			{
				delete map;
				log_cmd_error("Can't open map file `%s'.\n", filename.c_str());
			}
			Frontend::frontend_call(map, &f, filename, (filename.size() > 3 && filename.compare(filename.size() - 3, std::string::npos, ".il") == 0 ? "rtlil" : "verilog"));
			f.close();

			if (filename.size() <= 3 || filename.compare(filename.size() - 3, std::string::npos, ".il") != 0)
			{
				Pass::call(map, "proc");
				Pass::call(map, "opt_clean");
			}
		}
	}

	std::map<std::string, RTLIL::Module *> needle_map, haystack_map;
	std::vector<RTLIL::Module *> needle_list;

	log_header(design, "Creating graphs for SubCircuit library.\n");

	for (auto module : map->modules())
	{
		SubCircuit::Graph mod_graph;
		std::string graph_name = "needle_" + RTLIL::unescape_id(module->name);
		log("Creating needle graph %s.\n", graph_name.c_str());
		if (module2graph(mod_graph, module, constports))
		{
			solver.addGraph(graph_name, mod_graph);
			needle_map[graph_name] = module;
			needle_list.push_back(module);
		}
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

	std::sort(needle_list.begin(), needle_list.end(), compareSortNeedleList);

	for (auto needle : needle_list)
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
			log("\nMatch #%d: (%s in %s)\n", i, result.needleGraphId.c_str(), result.haystackGraphId.c_str());
			for (const auto &it : result.mappings)
			{
				auto *c = static_cast<RTLIL::Cell *>(it.second.haystackUserData);
				std::vector<RTLIL::Wire *> wires = get_output_wires(c);
				throw std::runtime_error("CSourceLoc not implemented yet");
				// SourceLoc source_loc = SourceLoc::parse(c->get_src_attribute());

				// log("%s: %s",
				// 	c->type.str().c_str(),
				// 	source_loc.toStringPretty().c_str());
				// log("\n");
			}
		}
	}

	delete map;
	log_pop();
}
