#!/bin/sh

# yosys -p "verific -sv build/openpiton__chip_0.1/pickle-icarus/openpiton__chip_0.1.v" -p "hierarchy -top chip" -p proc -p memory -p opt_clean -p flatten -p "write_json examples/fixtures/larger_designs/json/openpiton_chip_full.json"
# yosys -p "verific -sv examples/fixtures/larger_designs/json/openpiton_tile.v" -p "hierarchy -top tile" -p proc -p memory -p opt_clean -p flatten -p "write_json examples/fixtures/larger_designs/json/openpiton_tile_full.json"
# yosys -p "verific -sv examples/fixtures/larger_designs/json/openpiton_tile.v" -p "hierarchy -top tile" -p proc -p memory -p opt_clean -p flatten -p "write_json examples/fixtures/larger_designs/json/openpiton_tile_full.json"
cp examples/fixtures/larger_designs/json/openpiton_tile.json.gz examples/fixtures/larger_designs/json/openpiton_tile_bak.json.gz
rm examples/fixtures/larger_designs/json/openpiton_tile.json
gzip -d examples/fixtures/larger_designs/json/openpiton_tile.json.gz
mv examples/fixtures/larger_designs/json/openpiton_tile_bak.json.gz examples/fixtures/larger_designs/json/openpiton_tile.json.gz

sed -E 's/dcd_fuse_repair_en//g' examples/fixtures/larger_designs/json/openpiton_tile.json
sed -E '/[[:space:]]*"fuse_dcd_repair_en": \[ "0", "0" \],/d' examples/fixtures/larger_designs/json/openpiton_tile.json

yosys -p "read_json examples/fixtures/larger_designs/json/openpiton_tile.json" -p "hierarchy -top tile" -p proc -p memory -p opt_clean -p "flatten" -p "write_json examples/fixtures/larger_designs/json/openpiton_tile_flat.json"