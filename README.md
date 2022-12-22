# CARLA-Autoware Bridge Plus

The bridge exposes Carla simulator parameters and object entities to
ROS 2 topics in Autoware message types. It supports the following
features:

- Automatic topic creation for newly added vehicles and other actors.
- Multi-vehicle and multi-actor message publication/subscription.
- Support both direct and Ackermann vehicle control.
- Direct connection to Carla Simulator without going through extra
  bridges.

## Installation

Please read the [installation guide](doc/INSTALL.md).

## Usage

The bridge is programmed as a ROS node. Please read the [usage
guide](doc/USAGE.md) to learn about the launch process. The [ROS API
reference](doc/API.md) lists ROS topics provided by this node.

## Bugs and Issues

- The bridge halts and any other Carla client reload the world.
- When any other Carla client applies control to a car, the bridge is
  no longer able to control that car.

## License

This project is licensed under MIT license. Please check [the license
file](LICENSE.txt).
