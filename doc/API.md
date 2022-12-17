# ROS API Reference

This page enumerates available ROS topics provided by the bridge.

## Actors

Actors represent movable and interactive objects in Carla
simulator. It includes vehicles, sensors, traffic lights and traffic
signs. This bridge exposes their topics under individual
namespaces. The namespaces are named as follows.


| Actor   | Namespace                     | Example                |
|---------|-------------------------------|------------------------|
| vehicle | `/carla/vehicle/<ROLE_NAME>`  | `/carla/vehicle/hero`  |
| others  | `/carla/sensor/id_<ACTOR_ID>` | `/carla/vehicle/id_10` |

For simplicity, the namespace is denoted as `<P>` prefix. For example,
`<P>/odometry` denotes `/carla/vehicle/<ROLE_NAME>/odometry` for
vehicles.

### Common Actor Topics

The topics are defined for any actor kinds.


| Kind | Name               | Interface                                    | Description                                |
|------|--------------------|----------------------------------------------|--------------------------------------------|
| pub  | `<P>/odometry`     | `nav_msgs/msg/Odometry`                        | Object pose, velocity and angular velocity |
| pub  | `<P>/acceleration` | `geometry_msgs/msg/AccelWithCovarianceStamped` | Object acceleration with a timestamp       |

### Vehicle Topics

| Kind | Name               | Interface                                                                                                                               | Description                                                     |
|------|--------------------|-----------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------|
| pub  | `<P>/vehicle_info` | [`carla_msgs/msg/CarlaEgoVehicleInfo`](https://carla.readthedocs.io/projects/ros-bridge/en/latest/ros_msgs/#carlaegovehicleinfomsg)       | Vehicle information including max steering angle, etc.          |
| sub  | `<P>/control_cmd`  | [`carla_msgs/msg/CarlaEgoVehicleControl`](https://carla.readthedocs.io/projects/ros-bridge/en/latest/ros_msgs/#carlaegovehiclecontrolmsg) | Vehicle driving parameters, including brake, throttle and steer |

### Sensor Topics

Carla provides various kinds of sensors. The sensor type is published
in the topic.

| Kind | Name       | Interface             | Description                                                                                                         |
|------|------------|-----------------------|---------------------------------------------------------------------------------------------------------------------|
| pub  | `<P>/type` | `std_msgs/msg/String` | The sensor type string. The list of types can be found [here](https://carla.readthedocs.io/en/latest/ref_sensors/). |

The published data topic name and interface depend on the sensor type.

| Sensor Type                      | Name                      | Interface                     | Description                                                                                                                                            |
|----------------------------------|---------------------------|-------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| `sensor.camera.rgb`              | `<P>/image`               | `sensor_msgs/msg/Image`       | Camera image pixel data                                                                                                                                |
| `sensor.lidar.ray_cast`          | `<P>/pointcloud`          | `sensor_msgs/msg/PointCloud2` | Point array of positions. Fields are defined [here](https://carla.readthedocs.io/en/latest/python_api/#carlalidarmeasurement).                         |
| `sensor.lidar.ray_cast_semantic` | `<P>/semantic_pointcloud` | `sensor_msgs/msg/PointCloud2` | Point array of positions and object tags. Fields are defined [here](https://carla.readthedocs.io/en/latest/python_api/#carlasemanticlidarmeasurement). |
| `sensor.other.imu`               | `<P>/imu`                 | `sensor_msgs/msg/Imu`         | Linear and angular acceleration measurements                                                                                                           |
| `sensor.other.collision`         | `<P>/event`               | `builtin_interfaces/msg/Time` | Time of the last collision event                                                                                                                       |


### Traffic Sign

| Kind | Name                 | Interface                             | Description                                             |
|------|----------------------|---------------------------------------|---------------------------------------------------------|
| pub  | `<P>/trigger_volume` | `moveit_msgs/msg/OrientedBoundingBox` | The trigger box indicating where a vehicle should stop. |

### Traffic Light

| Kind | Name                 | Interface                                                | Description                                             |
|------|----------------------|----------------------------------------------------------|---------------------------------------------------------|
| pub  | `<P>/trigger_volume` | `moveit_msgs/msg/OrientedBoundingBox`                    | The trigger box indicating where a vehicle should stop. |
| pub  | `<P>/status`         | `autoware_auto_perception_msgs/msg/TrafficSignalStamped` | The color status of the traffic lights                  |

## Map

| Kind | Name         | Interface                                      | Description                                  |
|------|--------------|------------------------------------------------|----------------------------------------------|
| srv  | `/carla/map` | `autoware_auto_mapping_msgs/srv/HADMapService` | Provides vector map data in Lanelet2 format. |

