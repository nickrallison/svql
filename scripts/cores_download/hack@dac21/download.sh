#!/bin/sh

# Capture the current directory before any cd operations
ORIGINAL_DIR=$(pwd)

# if path not exist
if [ ! -d "generated/hackatdac21" ]; then
    mkdir -p generated/hackatdac21
    git clone https://github.com/HACK-EVENT/hackatdac21.git --revision=bcae7aba7f9daee8ad2cfd47b997ac7ad6611034 generated/hackatdac21
    
    # Copy Modified Files
    cp scripts/cores_download/hack@dac21/preproc.py generated/hackatdac21/piton/tools/src/fusesoc/preproc.py
    cp scripts/cores_download/hack@dac21/_exu_bw_r_irf_common.core_ generated/hackatdac21/piton/design/chip/tile/sparc/exu/bw_r_irf/common/rtl/exu_bw_r_irf_common.core
    cp scripts/cores_download/hack@dac21/_manycore.core_ generated/hackatdac21/piton/verif/env/manycore/manycore.core            
    cp scripts/cores_download/hack@dac21/_chipset.core_ generated/hackatdac21/piton/design/chipset/rtl/chipset.core
fi

# cd generated/hackatdac21
export PITON_ROOT=$(pwd)/generated/hackatdac21
export DV_ROOT=$(pwd)/generated/hackatdac21/piton

source $PITON_ROOT/piton/piton_settings.bash
source $PITON_ROOT/piton/ariane_setup.sh
source $PITON_ROOT/piton/ariane_build_tools.sh

chmod +x $PITON_ROOT/piton/tools/bin/pyhp.py

if [ ! -f "fusesoc.conf" ]; then
    fusesoc library add hackatdac21 "$PITON_ROOT"
fi
