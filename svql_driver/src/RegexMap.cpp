#include "RegexMap.hpp"

#include <fstream>

// #include <nlohmann/json.hpp>

#include "kernel/log.h"

// using nlohmann::json;

using namespace Yosys;

namespace svql
{
    // RegexMap load_regex_map(const std::string &jsonFile)
    // {
    //     std::ifstream f(jsonFile);
    //     if (!f.is_open())
    //         throw std::runtime_error("Cannot open regex file " + jsonFile);
    //
    //     json j;  f >> j;
    //
    //     RegexMap out;
    //     for (auto &mod : j.items()) {
    //         const std::string &modName = mod.key();
    //         for (auto &sig : mod.value().items()) {
    //             RTLIL::IdString sigId = RTLIL::escape_id(sig.key());
    //             std::string pattern   = sig.value().get<std::string>();
    //             out[modName][sigId]   = { std::regex(pattern), pattern };
    //         }
    //     }
    //     return out;
    // }

    RegexMap load_regex_map(const std::string &jsonFile)
    {
        RegexMap out;
        return out;
    }

}
