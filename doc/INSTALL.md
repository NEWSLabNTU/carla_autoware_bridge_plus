# Installation Guide for Linux


The instructions were tested on Ubuntu 20.04. Other Linuxes may work
but it might extra fix-ups.

## Prepare Toolchains

Install build tools. This step is done at most once.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --git https://github.com/jerry73204/cargo-ament-build.git
pip3 install git+https://github.com/jerry73204/colcon-ros-cargo.git@merge-colcon-cargo
```

## Install Dependencies

This package runs on ROS Galactic. It will be upgraded to ROS Humble
in the future as Autoware drops support for Galactic since
Dec, 2022. For the time being, Galactic versions of Ubuntu and
Autoware are used.

Install extra dependencies:


```bash
sudo apt install ros-galactic-moveit-msgs
pip install https://github.com/usdot-fhwa-stol/opendrive2lanelet/archive/develop.zip
```


To use Galactic version of Autoware,

```bash
git clone https://github.com/autowarefoundation/autoware.git
cd autoware

git checkout galactic
vcs import src < autoware.repos
vcs pull src < autoware.repos
```

## Repository Preparation

Prepare a ROS repository including this bridge, Autoware and Carla's ros-bridge.

```
repo/
└── src/
    ├── carla_autoware_bridge_plus/ (this repo)
    ├── ros-bridge/ (https://github.com/carla-simulator/ros-bridge master branch)
    └── autoware/ (https://github.com/autowarefoundation/autoware.git galactic branch)
```

The `ros-bridge` has git submodules. Remember to update them.

```bash
cd repo/src/ros-bridge
git submodule update --init --recursive
```

The `autoware` tracks underlying dependencies using `vcs`. Please
update the dependencies this way.

```bash
cd repo/src/autoware
git checkout galactic  # Use Galactic branch

mkdir -p src
vcs import src < autoware.repos
vcs pull src < autoware.repos
```

## Build the Repository

Build the whole repository inside the repo/ directory.

```bash
cd repo/

colcon build \
    --symlink-install \
    --cmake-args -DCMAKE_BUILD_TYPE=Release \
    --cargo-args --release
```
