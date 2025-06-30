#include "GraphConversion.hpp"

#include "kernel/rtlil.h"
#include "kernel/sigtools.h"
#include "kernel/log.h"

using namespace Yosys;

namespace svql {

    std::vector<RTLIL::Wire*> get_output_wires(RTLIL::Cell *cell) {
        std::vector<RTLIL::Wire*> result;
        for (const auto &conn : cell->connections()) {
            if (cell->output(conn.first)) {
                for (const RTLIL::SigBit &bit : conn.second) {
                    if (bit.is_wire())
                        result.push_back(bit.wire);
                }
            }
        }
        return result;
    }

    bool module2graph(SubCircuit::Graph &graph, RTLIL::Module *mod, bool constports, RTLIL::Design *sel,
				  int max_fanout, std::set<std::pair<RTLIL::IdString, RTLIL::IdString>> *split)
	{
		SigMap sigmap(mod);
		std::map<RTLIL::SigBit, bit_ref_t> sig_bit_ref;

		if (sel && !sel->selected(mod))
		{
			log("  Skipping module %s as it is not selected.\n", log_id(mod->name));
			return false;
		}

		if (mod->processes.size() > 0)
		{
			log("  Skipping module %s as it contains unprocessed processes.\n", log_id(mod->name));
			return false;
		}

		if (constports)
		{
			graph.createNode("$const$0", "$const$0", nullptr, true);
			graph.createNode("$const$1", "$const$1", nullptr, true);
			graph.createNode("$const$x", "$const$x", nullptr, true);
			graph.createNode("$const$z", "$const$z", nullptr, true);
			graph.createPort("$const$0", "\\Y", 1);
			graph.createPort("$const$1", "\\Y", 1);
			graph.createPort("$const$x", "\\Y", 1);
			graph.createPort("$const$z", "\\Y", 1);
			graph.markExtern("$const$0", "\\Y", 0);
			graph.markExtern("$const$1", "\\Y", 0);
			graph.markExtern("$const$x", "\\Y", 0);
			graph.markExtern("$const$z", "\\Y", 0);
		}

		std::map<std::pair<RTLIL::Wire *, int>, int> sig_use_count;
		if (max_fanout > 0)
			for (auto cell : mod->cells())
			{
				if (!sel || sel->selected(mod, cell))
					for (auto &conn : cell->connections())
					{
						RTLIL::SigSpec conn_sig = conn.second;
						sigmap.apply(conn_sig);
						for (auto &bit : conn_sig)
							if (bit.wire != nullptr)
								sig_use_count[std::pair<RTLIL::Wire *, int>(bit.wire, bit.offset)]++;
					}
			}

		// create graph nodes from cells
		for (auto cell : mod->cells())
		{
			if (sel && !sel->selected(mod, cell))
				continue;

			std::string type = cell->type.str();
			if (sel == nullptr && type.compare(0, 2, "\\$") == 0)
				type = type.substr(1);
			graph.createNode(cell->name.str(), type, (void *)cell);

			for (auto &conn : cell->connections())
			{
				graph.createPort(cell->name.str(), conn.first.str(), conn.second.size());

				if (split && split->count(std::pair<RTLIL::IdString, RTLIL::IdString>(cell->type, conn.first)) > 0)
					continue;

				RTLIL::SigSpec conn_sig = conn.second;
				sigmap.apply(conn_sig);

				for (int i = 0; i < conn_sig.size(); i++)
				{
					auto &bit = conn_sig[i];

					if (bit.wire == nullptr)
					{
						if (constports)
						{
							std::string node = "$const$x";
							if (bit == RTLIL::State::S0)
								node = "$const$0";
							if (bit == RTLIL::State::S1)
								node = "$const$1";
							if (bit == RTLIL::State::Sz)
								node = "$const$z";
							graph.createConnection(cell->name.str(), conn.first.str(), i, node, "\\Y", 0);
						}
						else
							graph.createConstant(cell->name.str(), conn.first.str(), i, int(bit.data));
						continue;
					}

					if (max_fanout > 0 && sig_use_count[std::pair<RTLIL::Wire *, int>(bit.wire, bit.offset)] > max_fanout)
						continue;

					if (sel && !sel->selected(mod, bit.wire))
						continue;

					if (sig_bit_ref.count(bit) == 0)
					{
						bit_ref_t &bit_ref = sig_bit_ref[bit];
						bit_ref.cell = cell->name.str();
						bit_ref.port = conn.first.str();
						bit_ref.bit = i;
					}

					bit_ref_t &bit_ref = sig_bit_ref[bit];
					graph.createConnection(bit_ref.cell, bit_ref.port, bit_ref.bit, cell->name.str(), conn.first.str(), i);
				}
			}
		}

		// mark external signals (used in non-selected cells)
		for (auto cell : mod->cells())
		{
			if (sel && !sel->selected(mod, cell))
				for (auto &conn : cell->connections())
				{
					RTLIL::SigSpec conn_sig = conn.second;
					sigmap.apply(conn_sig);

					for (auto &bit : conn_sig)
						if (sig_bit_ref.count(bit) != 0)
						{
							bit_ref_t &bit_ref = sig_bit_ref[bit];
							graph.markExtern(bit_ref.cell, bit_ref.port, bit_ref.bit);
						}
				}
		}

		// mark external signals (used in module ports)
		for (auto wire : mod->wires())
		{
			if (wire->port_id > 0)
			{
				RTLIL::SigSpec conn_sig(wire);
				sigmap.apply(conn_sig);

				for (auto &bit : conn_sig)
					if (sig_bit_ref.count(bit) != 0)
					{
						bit_ref_t &bit_ref = sig_bit_ref[bit];
						graph.markExtern(bit_ref.cell, bit_ref.port, bit_ref.bit);
					}
			}
		}

		// graph.print();
		return true;
	}
}
