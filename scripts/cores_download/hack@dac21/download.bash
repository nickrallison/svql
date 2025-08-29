#!/bin/bash

# Capture the current directory before any cd operations
ORIGINAL_DIR=$(pwd)

# if path not exist
if [ ! -d "generated/hackatdac21" ]; then
    mkdir -p generated
    git clone https://github.com/HACK-EVENT/hackatdac21.git generated/hackatdac21
    cd generated/hackatdac21
    git checkout bcae7aba7f9daee8ad2cfd47b997ac7ad6611034
    cd $ORIGINAL_DIR

    # Copy Modified Files
    cp scripts/cores_download/hack@dac21/preproc.py generated/hackatdac21/piton/tools/src/fusesoc/preproc.py
    cp scripts/cores_download/hack@dac21/_exu_bw_r_irf_common.core_ generated/hackatdac21/piton/design/chip/tile/sparc/exu/bw_r_irf/common/rtl/exu_bw_r_irf_common.core
    cp scripts/cores_download/hack@dac21/_manycore.core_ generated/hackatdac21/piton/verif/env/manycore/manycore.core            
    cp scripts/cores_download/hack@dac21/_chipset.core_ generated/hackatdac21/piton/design/chipset/rtl/chipset.core
fi

# cd generated/hackatdac21
export PITON_ROOT=$(pwd)/generated/hackatdac21
export DV_ROOT=$(pwd)/generated/hackatdac21/piton

# source $PITON_ROOT/piton/piton_settings.bash
# cd $PITON_ROOT/ && source $ORIGINAL_DIR/scripts/cores_download/hack@dac21/ariane_setup.bash
# cd $ORIGINAL_DIR
# source $PITON_ROOT/piton/ariane_build_tools.sh

chmod +x $PITON_ROOT/piton/tools/bin/pyhp.py

rm -rf fusesoc.conf
fusesoc library add hackatdac21 "$PITON_ROOT"

fusesoc run --target=pickle openpiton::system:0.1
cp $ORIGINAL_DIR/build/openpiton__system_0.1/pickle-icarus/openpiton__system_0.1 examples/fixtures/larger_designs/verilog/openpiton_system.v