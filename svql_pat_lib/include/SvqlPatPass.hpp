#pragma once

#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

using namespace Yosys;

struct SvqlPatPass : public Pass
{
	SvqlPatPass();
	void help() override;
	void execute(std::vector<std::string> args, RTLIL::Design *design) override;
} SvqlPatPass;