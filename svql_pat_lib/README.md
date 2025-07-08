# svql

## Prerequisites

- cmake
  - Ubuntu: `sudo apt-get install cmake`

## Depends

- nlohmann-json
  - Ubuntu: `sudo apt-get install nlohmann-json3-dev`
- Yosys Installed From Source so yosys-config is also installed
```sh
git clone https://github.com/YosysHQ/yosys.git
cd yosys
git submodule update --init --recursive
make -j$(nproc)
sudo make install
```