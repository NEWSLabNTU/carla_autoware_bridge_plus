# Usage Guide for Linux

The instructions were tested on Ubuntu 20.04. Other Linuxes may work
but it might extra fix-ups.

## The Launch Process

Always remember to source generated `setup.sh` whenever you start a
new shell.

```bash
source repo/install/setup.sh
```

There are two ways to bring up this bridge, either by `ros2 run` or
`cargo run`.

Using `ros2` is standard way from ROS and can be integrated into ROS 2
launch. Any modification to the code requires rebuilding the whole
repository.

For developers, it is recommended to use `cargo run` because it
automatically re-compiles if the code is modified. In this way, the
whole repository is only needed to be built once, and later on, repeat
`cargo run` whenever the code is changed.

## Run Using `ros2 run` (for Users)

To run with default settings,


```bash
ros2 run carla_autoware_bridge_plus carla_autoware_bridge_plus
```

To pass custom address and port parameters,

```bash
ros2 run carla_autoware_bridge_plus carla_autoware_bridge_plus \
    --ros-args \
    -p carla_host:=127.0.0.1 \
    -p carla_port:=3000
```

## Run Using `cargo run` (for Developers)

```bash
cd repo/src/carla_autoware_bridge_plus/
cargo run
```

To pass custom address and port parameters,

```bash
cargo run -- \
    --ros-args \
    -p carla_host:=127.0.0.1 \
    -p carla_port:=2000
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
