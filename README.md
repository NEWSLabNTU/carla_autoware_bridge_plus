# CARLA-Autoware Bridge Plus

A ROS 2 bridge that forwards topics from CARLA simualtor's
[ros_bridge](https://github.com/carla-simulator/ros-bridge) to
Autoware and vice versa.

This repository is still in alpha stage. Great refactoring is
expected.


## Usage

First, prepare a repository like this.

```
repo/
└── src/
    ├── carla_autoware_bridge_plus/ (this repo)
    ├── ros-bridge/ (https://github.com/carla-simulator/ros-bridge master branch)
    ├── autoware/ (https://github.com/autowarefoundation/autoware.git galactic branch)
    └── astuff_sensor_msgs/ (https://github.com/astuff/astuff_sensor_msgs master branch)
```

Install build tools. This step is done at most once.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --git https://github.com/jerry73204/cargo-ament-build.git
pip3 install git+https://github.com/jerry73204/colcon-ros-cargo.git@merge-colcon-cargo
```
            
Finally, run colcon to build the whole project inside the repo/ directory.

```bash
colcon build --symlink-install --cmake-args -DCMAKE_BUILD_TYPE=Release --cargo-args --release
```

After the repository is built, run the command to launch the bridge.

```bash
ros2 run carla_autoware_bridge_plus carla_autoware_bridge_plus
```

## License

This project is licensed under MIT license. Please check [the license
file](LICENSE.txt).
