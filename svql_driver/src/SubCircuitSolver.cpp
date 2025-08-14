#include "SubCircuitSolver.hpp"
#include "GraphConversion.hpp"

#include "kernel/rtlil.h"
#include "kernel/log.h"

using namespace Yosys;

namespace svql
{
	SubCircuitSolver::SubCircuitSolver() = default;

	void SubCircuitSolver::setVerbose(bool v)
	{
		myVerbose = v;
		SubCircuit::Solver::setVerbose();
	}

	/* ---------------- userCompareNodes  (big, unchanged) ------------- */

	bool SubCircuitSolver::compareAttributes(const std::set<RTLIL::IdString> &attr, const dict<RTLIL::IdString, RTLIL::Const> &needleAttr, const dict<RTLIL::IdString, RTLIL::Const> &haystackAttr) const
	{
		for (auto &it : attr)
		{
			size_t nc = needleAttr.count(it), hc = haystackAttr.count(it);
			if (nc != hc || (nc > 0 && needleAttr.at(it) != haystackAttr.at(it)))
				return false;
		}
		return true;
	}

	RTLIL::Const SubCircuitSolver::unifiedParam(RTLIL::IdString cell_type, RTLIL::IdString param, RTLIL::Const value)
	{
		if (!cell_type.begins_with("$") || cell_type.begins_with("$_"))
			return value;

#define param_bool(_n) \
	if (param == _n)   \
		return value.as_bool();
		param_bool(ID::ARST_POLARITY);
		param_bool(ID::A_SIGNED);
		param_bool(ID::B_SIGNED);
		param_bool(ID::CLK_ENABLE);
		param_bool(ID::CLK_POLARITY);
		param_bool(ID::CLR_POLARITY);
		param_bool(ID::EN_POLARITY);
		param_bool(ID::SET_POLARITY);
		param_bool(ID::TRANSPARENT);
#undef param_bool

#define param_int(_n) \
	if (param == _n)  \
		return value.as_int();
		param_int(ID::ABITS)
			param_int(ID::A_WIDTH)
				param_int(ID::B_WIDTH)
					param_int(ID::CTRL_IN_WIDTH)
						param_int(ID::CTRL_OUT_WIDTH)
							param_int(ID::OFFSET)
								param_int(ID::PORTID)
									param_int(ID::PRIORITY)
										param_int(ID::RD_PORTS)
											param_int(ID::SIZE)
												param_int(ID::STATE_BITS)
													param_int(ID::STATE_NUM)
														param_int(ID::STATE_NUM_LOG2)
															param_int(ID::STATE_RST)
																param_int(ID::S_WIDTH)
																	param_int(ID::TRANS_NUM)
																		param_int(ID::WIDTH)
																			param_int(ID::WR_PORTS)
																				param_int(ID::Y_WIDTH)
#undef param_int

																					return value;
	}

	bool SubCircuitSolver::userCompareNodes(const std::string &, const std::string &, void *needleUserData,
											  const std::string &, const std::string &, void *haystackUserData, const std::map<std::string, std::string> &portMapping)
	{
		RTLIL::Cell *needleCell = (RTLIL::Cell *)needleUserData;
		RTLIL::Cell *haystackCell = (RTLIL::Cell *)haystackUserData;

		if (!needleCell || !haystackCell)
		{
			log_assert(!needleCell && !haystackCell);
			return true;
		}

		if (!ignoreParameters)
		{
			std::map<RTLIL::IdString, RTLIL::Const> needle_param, haystack_param;
			for (auto &it : needleCell->parameters)
				if (!ignoredParams.count(std::pair<RTLIL::IdString, RTLIL::IdString>(needleCell->type, it.first)))
					needle_param[it.first] = unifiedParam(needleCell->type, it.first, it.second);
			for (auto &it : haystackCell->parameters)
				if (!ignoredParams.count(std::pair<RTLIL::IdString, RTLIL::IdString>(haystackCell->type, it.first)))
					haystack_param[it.first] = unifiedParam(haystackCell->type, it.first, it.second);
			if (needle_param != haystack_param)
				return false;
		}

		if (cell_attr.size() > 0 && !compareAttributes(cell_attr, needleCell->attributes, haystackCell->attributes))
			return false;

		if (wire_attr.size() > 0)
		{
			RTLIL::Wire *lastNeedleWire = nullptr;
			RTLIL::Wire *lastHaystackWire = nullptr;
			dict<RTLIL::IdString, RTLIL::Const> emptyAttr;

			for (auto &conn : needleCell->connections())
			{
				RTLIL::SigSpec needleSig = conn.second;
				RTLIL::SigSpec haystackSig = haystackCell->getPort(portMapping.at(conn.first.str()));

				for (int i = 0; i < min(needleSig.size(), haystackSig.size()); i++)
				{
					RTLIL::Wire *needleWire = needleSig[i].wire, *haystackWire = haystackSig[i].wire;
					if (needleWire != lastNeedleWire || haystackWire != lastHaystackWire)
						if (!compareAttributes(wire_attr, needleWire ? needleWire->attributes : emptyAttr, haystackWire ? haystackWire->attributes : emptyAttr))
							return false;
					lastNeedleWire = needleWire, lastHaystackWire = haystackWire;
				}
			}
		}

		return true;
	}

}
