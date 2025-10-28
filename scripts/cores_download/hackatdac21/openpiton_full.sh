#!/bin/sh

# yosys -p "verific -sv build/openpiton__chip_0.1/pickle-icarus/openpiton__chip_0.1.v" -p "hierarchy -top chip" -p proc -p memory -p opt_clean -p flatten -p "write_json examples/fixtures/larger_designs/json/openpiton_chip_full.json"
# yosys -p "verific -sv examples/fixtures/larger_designs/json/openpiton_tile.v" -p "hierarchy -top tile" -p proc -p memory -p opt_clean -p flatten -p "write_json examples/fixtures/larger_designs/json/openpiton_tile_full.json"
yosys -p "verific -sv examples/fixtures/larger_designs/json/openpiton_tile.v" -p "hierarchy -top tile" -p proc -p memory -p opt_clean -p flatten -p "write_json examples/fixtures/larger_designs/json/openpiton_tile_full.json"