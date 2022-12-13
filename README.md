# CARLA-Autoware Bridge Plus

The bridge exposes Carla simulator parameters and object entities to
ROS 2 topics in Autoware message types.

This repository is still in alpha stage. Great refactoring is
expected.

## Prepare


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

Install an extra dependency:


```bash
sudo apt install ros-galactic-moveit-msgs
```

## Build

Prepare a ROS repository including this bridge, Autoware and Carla's ros-bridge.

```
repo/
└── src/
    ├── carla_autoware_bridge_plus/ (this repo)
    ├── ros-bridge/ (https://github.com/carla-simulator/ros-bridge master branch)
    └── autoware/ (https://github.com/autowarefoundation/autoware.git galactic branch)
```


Build the whole project inside the repo/ directory.

```bash
cd repo/

colcon build \
    --symlink-install \
    --cmake-args -DCMAKE_BUILD_TYPE=Release \
    --cargo-args --release
```

## Launch the Bridge

The bridge can be brought up using either `ros2` or `cargo run`. Using
`ros2` is standard way to bring up a ROS node. For developers, it's
recommended to use `cargo run`.

Always remember to source generated `setup.sh`.

```bash
source repo/install/setup.sh
```

### Run Bridge using `ros2`


```bash
ros2 run carla_autoware_bridge_plus carla_autoware_bridge_plus
```

### Run Bridge using `cargo run`

```bash
cd repo/src/carla_autoware_bridge_plus/
cargo run
```

## Node Parameters

The node have the following parameters.

- `carla_host`

  Sets the Carla server address the bridge connects to. The default is
  "127.0.0.1".

- `carla_host`

  Sets the Carla server port. The default is 2000.

- `carla_timeout_millis`

  Sets the Carla client connection timeout in milliseconds. The
  default is 20000.

## License

This project is licensed under MIT license. Please check [the license
file](LICENSE.txt).
