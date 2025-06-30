#pragma once

#include "kernel/register.h"

using namespace Yosys;

struct SvqlPass : public Pass
{
	SvqlPass();
	void help()    override;
	void execute(std::vector<std::string> args,  RTLIL::Design *design) override;
} SvqlPass;

