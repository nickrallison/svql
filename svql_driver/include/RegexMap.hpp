
#pragma once

#include <map>
#include <regex>
#include <string>

#include "kernel/yosys.h"
#include "kernel/yosys.h"
#include "kernel/rtlil.h"

using namespace Yosys;

namespace svql
{
    using RegexEntry = std::pair<std::regex, std::string>;
    using RegexMap   = std::map<std::string, std::map<RTLIL::IdString, RegexEntry>>;

    //  Throws std::runtime_error on file / JSON error
    RegexMap load_regex_map(const std::string &jsonFile);

} // namespace svql

