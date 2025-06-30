#pragma once

using namespace Yosys;

namespace svql
{
    inline bool compareSortNeedleList(RTLIL::Module *left, RTLIL::Module *right)
    {
        int left_idx = 0, right_idx = 0;
        if (left->attributes.count(ID::extract_order) > 0)
            left_idx = left->attributes.at(ID::extract_order).as_int();
        if (right->attributes.count(ID::extract_order) > 0)
            right_idx = right->attributes.at(ID::extract_order).as_int();
        if (left_idx != right_idx)
            return left_idx < right_idx;
        return left->name < right->name;
    }
}
