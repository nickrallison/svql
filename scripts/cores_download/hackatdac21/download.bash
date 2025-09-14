#!/bin/bash

# RUN:
# source scripts/cores_download/hackatdac21/download.bash

# DEPENDENCIES
# pyenv
# pyenv-virtualenv
# 

# INSTALL
# pyenv install 2.7.18
# pyenv install 3.13.7

export ORIGINAL_DIR=$(pwd)
export PITON_ROOT="$(pwd)/generated/hackatdac21"


# Clone Repo
if [ ! -d "$PITON_ROOT" ]; then
    mkdir -p generated
    git clone https://github.com/HACK-EVENT/hackatdac21.git generated/hackatdac21
    cd $PITON_ROOT
    git checkout bcae7aba7f9daee8ad2cfd47b997ac7ad6611034
    cd $ORIGINAL_DIR
    cp $ORIGINAL_DIR/scripts/cores_download/hackatdac21/exu_bw_r_irf_common._core $PITON_ROOT/piton/design/chip/tile/sparc/exu/bw_r_irf/common/rtl/exu_bw_r_irf_common.core
    cp $ORIGINAL_DIR/scripts/cores_download/hackatdac21/preproc.py                $PITON_ROOT/piton/tools/src/fusesoc/preproc.py
    cp $ORIGINAL_DIR/scripts/cores_download/hackatdac21/preprocessor.core         $PITON_ROOT/piton/tools/src/fusesoc/preprocessor.core
fi

if [ ! -d ./venv ]; then
  $HOME/.pyenv/versions/3.13.7/bin/python -m venv ./venv
fi

source venv/bin/activate
source $PITON_ROOT/piton/piton_settings.bash
source $PITON_ROOT/piton/ariane_setup.sh

venv/bin/pip install fusesoc
venv/bin/pip install pyyaml

rm -f fusesoc.conf
fusesoc library add openpiton $PITON_ROOT

chmod +x $PITON_ROOT/piton/tools/bin/pyhp.py

source venv/bin/activate

export PATH="$PATH:$PITON_ROOT/piton/tools/bin"
export PATH="$HOME/.pyenv/versions/2.7.18/bin:$PATH"

# Genetate Base Netlist
fusesoc run --target=pickle chip