# CARLA-Autoware Bridge Plus

A ROS 2 bridge that forwards topics from CARLA simualtor's
[ros_bridge](https://github.com/carla-simulator/ros-bridge) to
Autoware and vice versa.

This repository is still in alpha stage. Great refactoring is
expected.

## Prepare


Install build tools. This step is done at most once.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --git https://github.com/jerry73204/cargo-ament-build.git
pip3 install git+https://github.com/jerry73204/colcon-ros-cargo.git@merge-colcon-cargo
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

### Run Bridge using `ros2`


```bash
ros2 run carla_autoware_bridge_plus carla_autoware_bridge_plus
```

### Run Bridge using `cargo run`

```bash
cd repo/src/carla_autoware_bridge_plus/
cargo run
```

## License

This project is licensed under MIT license. Please check [the license
file](LICENSE.txt).
