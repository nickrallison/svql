#pragma once

#include "kernel/register.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

#include "svql_common.h"

using namespace Yosys;

struct SvqlPatPass : public Pass
{
	SvqlPatPass();
	void help() override;
	void execute(std::vector<std::string> args, RTLIL::Design *design) override;
} SvqlPatPass;

// struct CPattern *cpattern_new(const char *file_loc,
//                               const char *const *in_ports,
//                               uintptr_t in_ports_len,
//                               const char *const *out_ports,
//                               uintptr_t out_ports_len,
//                               const char *const *inout_ports,
//                               uintptr_t inout_ports_len);

// Additional JSON serialization functions from svql_common
// char *cpattern_to_json(const struct CPattern *pattern);
// struct CPattern *cpattern_from_json(const char *json_str);
// void cpattern_json_free(char *json_str);
