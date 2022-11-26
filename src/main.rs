mod types;

use anyhow::{bail, Result};
use futures::{
    join, select,
    stream::{StreamExt, TryStreamExt},
    try_join, Stream, TryFutureExt,
};
use ndarray::Array2;
use num_traits::FromPrimitive;
use r2r::{
    ackermann_msgs::msg::AckermannDrive,
    autoware_auto_control_msgs::msg::{
        AckermannControlCommand, AckermannLateralCommand, LongitudinalCommand,
    },
    autoware_auto_perception_msgs::msg::{
        DetectedObject, DetectedObjectKinematics, DetectedObjects, ObjectClassification, Shape,
    },
    autoware_auto_vehicle_msgs::msg::SteeringReport,
    carla_msgs::msg::{
        CarlaEgoVehicleControl, CarlaEgoVehicleInfo, CarlaEgoVehicleInfoWheel,
        CarlaEgoVehicleStatus,
    },
    derived_object_msgs::msg::{Object, ObjectArray},
    geometry_msgs::msg::{
        AccelWithCovariance, AccelWithCovarianceStamped, PoseWithCovariance, TwistWithCovariance,
        Vector3,
    },
    log_warn,
    nav_msgs::msg::{Odometry, Path},
    shape_msgs::msg::SolidPrimitive,
    Context, Node, ParameterValue, Publisher, QosProfile,
};
use std::time::Duration;
use tokio::task::spawn_blocking;

use crate::types::{
    AutowareClassification, AutowareShapeType, DerivedObjClassification, SolidPrimitiveType,
};

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = Context::create()?;
    let mut node = Node::create(ctx, "carla_autoware_bridge", "/simulator")?;

    let role_name = {
        let params = node.params.lock().unwrap();
        match params.get("role_name") {
            Some(ParameterValue::String(role_name)) => role_name,
            Some(_) => bail!("invalid parameter type"),
            None => "ego_vehicle",
        }
        .to_string()
    };

    // carla subscriptions
    let carla_obj_sub = node.subscribe::<ObjectArray>(
        &format!("/carla/{}/objects", role_name),
        QosProfile::default(),
    )?;
    let carla_odom_sub = node.subscribe::<Odometry>(
        &format!("/carla/{}/odometry", role_name),
        QosProfile::default(),
    )?;
    let carla_vehicle_status_sub = node.subscribe::<CarlaEgoVehicleStatus>(
        &format!("/carla/{}/vehicle_status", role_name),
        QosProfile::default(),
    )?;
    let carla_vehicle_info_sub = node.subscribe::<CarlaEgoVehicleInfo>(
        &format!("/carla/{}/vehicle_info", role_name),
        QosProfile::default(),
    )?;
    let carla_waypoints_sub = node.subscribe::<Path>(
        &format!("/carla/{}/waypoints", role_name),
        QosProfile::default(),
    )?;

    // carla publishers
    let carla_ackermann_pub = node.create_publisher::<AckermannDrive>(
        &format!("/carla/{}/ackermann_cmd", role_name),
        QosProfile::default(),
    )?;

    // autoware subscriptions
    let autoware_control_cmd_sub = node.subscribe::<AckermannControlCommand>(
        "/control/command/control_cmd",
        QosProfile::default(),
    )?;
    // let autoware_gear_cmd_sub =
    //     node.subscribe::<GearCommand>("/control/command/gear_cmd", QosProfile::default())?;
    // let autoware_hazard_lights_cmd_sub = node.subscribe::<HazardLightsCommand>(
    //     "/control/command/hazard_lights_cmd",
    //     QosProfile::default(),
    // )?;
    // let autoware_turn_indicators_cmd_sub = node.subscribe::<TurnIndicatorsCommand>(
    //     "/control/command/turn_indicators_cmd",
    //     QosProfile::default(),
    // )?;
    // let autoware_initial_pose_sub =
    //     node.subscribe::<PoseWithCovarianceStamped>("/initialpose3d", QosProfile::default())?;
    // let autoware_engabe_sub = node.subscribe::<Engage>("", QosProfile::default())?;

    // autoware publishers
    let autoware_accel_pub = node.create_publisher::<AccelWithCovarianceStamped>(
        "/localization/acceleration",
        QosProfile::default(),
    )?;
    let autoware_odom_pub =
        node.create_publisher::<Odometry>("/localization/kinematic_state", QosProfile::default())?;
    let autoware_obj_pub = node.create_publisher::<DetectedObjects>(
        "/perception/object_recognition/detection/objects",
        QosProfile::default(),
    )?;
    let autoware_steer_status_pub = node.create_publisher::<SteeringReport>(
        "/vehicle/status/steering_status",
        QosProfile::default(),
    )?;

    // tasks
    let spin_task = spawn_blocking(move || loop {
        node.spin_once(Duration::from_millis(10));
    })
    .map_err(anyhow::Error::from);
    let forward_obj = forward_obj(carla_obj_sub, autoware_obj_pub);
    let forward_odom = forward_odom(carla_odom_sub, autoware_odom_pub);
    let forward_status = forward_status(
        carla_vehicle_info_sub,
        carla_vehicle_status_sub,
        autoware_accel_pub,
        autoware_steer_status_pub,
    );
    let forward_ackermann = forward_ackermann(autoware_control_cmd_sub, carla_ackermann_pub);

    try_join!(
        spin_task,
        forward_odom,
        forward_status,
        forward_ackermann,
        forward_obj,
    )?;

    Ok(())
}

async fn forward_obj(
    sub: impl Stream<Item = ObjectArray> + Unpin,
    pub_: Publisher<DetectedObjects>,
) -> Result<()> {
    sub.map(anyhow::Ok)
        .try_fold(pub_, |pub_, in_msg| async move {
            let ObjectArray { header, objects } = in_msg;
            let objects = objects
                .into_iter()
                .map(|in_obj| {
                    use AutowareClassification as TC;
                    use AutowareShapeType as TT;
                    use DerivedObjClassification as FC;
                    use SolidPrimitiveType as FT;

                    let Object {
                        // header,
                        // id,
                        // detection_level,
                        object_classified,
                        pose,
                        twist,
                        // accel,
                        polygon,
                        shape: from_shape,
                        classification: from_class,
                        classification_certainty,
                        // classification_age,
                        ..
                    } = in_obj;

                    let to_class = loop {
                        if !object_classified {
                            break TC::UNKNOWN;
                        }
                        let Some(from_class) = FC::from_u8(from_class) else {
                            log_warn!(
                                env!("CARGO_BIN_NAME"),
                                "Unsupported classification {from_class}"
                            );
                            break TC::UNKNOWN;
                        };
                        let to_class = match from_class {
                            FC::UNKNOWN
                            | FC::UNKNOWN_BIG
                            | FC::UNKNOWN_MEDIUM
                            | FC::UNKNOWN_SMALL
                            | FC::OTHER_VEHICLE
                            | FC::BARRIER
                            | FC::SIGN => TC::UNKNOWN,
                            FC::PEDESTRIAN => TC::PEDESTRIAN,
                            FC::BIKE => TC::BICYCLE,
                            FC::CAR => TC::CAR,
                            FC::TRUCK => TC::TRUCK,
                            FC::MOTORCYCLE => TC::MOTORCYCLE,
                        };
                        break to_class;
                    };

                    let to_shape = loop {
                        let SolidPrimitive {
                            type_: from_type,
                            dimensions: from_dims,
                        } = from_shape;
                        let Some(from_type) = FT::from_u8(from_type) else {
                            break None
                        };
                        let (to_type, to_dims) = match from_type {
                            FT::BOX => {
                                let ty = TT::BOUNDING_BOX;
                                let dims = match &*from_dims {
                                    &[x, y, z] => Vector3 { x, y, z },
                                    _ => break None,
                                };
                                (ty, dims)
                            }
                            FT::CYLINDER => {
                                let ty = TT::CYLINDER;
                                let dims = match &*from_dims {
                                    &[x, y, z] => Vector3 { x, y, z },
                                    _ => break None,
                                };
                                (ty, dims)
                            }
                            FT::SPHERE | FT::CONE => break None,
                        };

                        break Some(Shape {
                            type_: to_type as u8,
                            footprint: polygon,
                            dimensions: to_dims,
                        });
                    }
                    .unwrap_or_default();

                    DetectedObject {
                        existence_probability: 1.0,
                        classification: vec![ObjectClassification {
                            label: to_class as u8,
                            probability: classification_certainty as f32 / 255.0,
                        }],
                        kinematics: DetectedObjectKinematics {
                            pose_with_covariance: PoseWithCovariance {
                                pose,
                                covariance: Array2::from_diag_elem(6, 1.0).into_raw_vec(),
                            },
                            has_position_covariance: true,
                            orientation_availability: 0, // UNAVAILABLE
                            twist_with_covariance: TwistWithCovariance {
                                twist,
                                covariance: Array2::from_diag_elem(6, 1.0).into_raw_vec(),
                            },
                            has_twist: true,
                            has_twist_covariance: true,
                        },
                        shape: to_shape,
                    }
                })
                .collect();
            let out_msg = DetectedObjects { header, objects };
            pub_.publish(&out_msg)?;
            Ok(pub_)
        })
        .await?;
    Ok(())
}

async fn forward_ackermann(
    sub: impl Stream<Item = AckermannControlCommand> + Unpin,
    pub_: Publisher<AckermannDrive>,
) -> Result<()> {
    sub.map(anyhow::Ok)
        .try_fold(pub_, |pub_, in_msg| async move {
            let AckermannControlCommand {
                lateral:
                    AckermannLateralCommand {
                        steering_tire_angle,
                        steering_tire_rotation_rate,
                        ..
                    },
                longitudinal:
                    LongitudinalCommand {
                        speed,
                        acceleration,
                        jerk,
                        ..
                    },
                ..
            } = in_msg;
            let out_msg = AckermannDrive {
                steering_angle: steering_tire_angle,
                steering_angle_velocity: steering_tire_rotation_rate,
                speed,
                acceleration,
                jerk,
            };

            pub_.publish(&out_msg)?;
            Ok(pub_)
        })
        .await?;
    Ok(())
}

async fn forward_odom(
    sub: impl Stream<Item = Odometry> + Unpin,
    pub_: Publisher<Odometry>,
) -> Result<()> {
    sub.map(anyhow::Ok)
        .try_fold(pub_, |pub_, msg| async move {
            pub_.publish(&msg)?;
            Ok(pub_)
        })
        .await?;
    Ok(())
}

async fn forward_status(
    info_sub: impl Stream<Item = CarlaEgoVehicleInfo> + Unpin,
    status_sub: impl Stream<Item = CarlaEgoVehicleStatus> + Unpin,
    accel_pub: Publisher<AccelWithCovarianceStamped>,
    steer_pub: Publisher<SteeringReport>,
) -> Result<()> {
    let mut info_sub = info_sub.fuse();
    let mut status_sub = status_sub.fuse();
    let (info, status) = join!(info_sub.next(), status_sub.next());
    let Some(mut info) = info else { return Ok(()); };
    let Some(mut status) = status else { return Ok(()); };

    loop {
        select! {
            new_info = info_sub.next() => {
                let Some(new_info) = new_info else { break };
                info = new_info;
            }
            new_status = status_sub.next() => {
                let Some(new_status) = new_status else { break };
                status = new_status;
            }
        }

        let CarlaEgoVehicleStatus {
            ref header,
            ref acceleration,
            control: CarlaEgoVehicleControl { steer, .. },
            ..
        } = status;
        let CarlaEgoVehicleInfo { ref wheels, .. } = info;

        if let Some(wheel) = wheels.get(0) {
            let CarlaEgoVehicleInfoWheel {
                max_steer_angle, ..
            } = wheel;
            let steering_tire_angle = max_steer_angle * steer;

            let steer_msg = SteeringReport {
                stamp: header.stamp.clone(),
                steering_tire_angle,
            };

            steer_pub.publish(&steer_msg)?;
        }

        let accel_msg = AccelWithCovarianceStamped {
            header: header.clone(),
            accel: AccelWithCovariance {
                accel: acceleration.clone(),
                covariance: Array2::from_diag_elem(6, 1.0).into_raw_vec(),
            },
        };
        accel_pub.publish(&accel_msg)?;
    }

    Ok(())
}
