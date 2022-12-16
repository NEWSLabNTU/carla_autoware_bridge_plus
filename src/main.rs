mod actor_node;
mod bridge;
mod map_srv;
mod params;
mod qos;
mod types;
mod utils;
// mod control_physics;

use anyhow::Result;
use bridge::Bridge;
use carla::{client::Client, prelude::*, rpc::ActorId};
use futures::{future::BoxFuture, join, select, stream::FuturesUnordered, FutureExt, StreamExt};
use params::Params;
use r2r::{log_info, std_msgs::msg::Empty, Clock, ClockType, Context, Node};
use std::{
    collections::{HashMap, HashSet},
    future::IntoFuture,
    time::Duration,
};
use tokio::task::spawn_blocking;

#[tokio::main]
async fn main() -> Result<()> {
    // Create channels
    let (future_tx, future_rx) = flume::bounded(4);

    // Construct publishers, subscribers and services, etc.
    let ctx = Context::create()?;
    let mut node = Node::create(ctx, "carla_autoware_bridge", "/carla")?;
    let Params {
        carla_host,
        carla_port,
        carla_timeout_millis,
    } = Params::load(&node)?;

    // Create Carla client
    let mut client = Client::connect(&carla_host, carla_port, None);
    client.set_timeout(Duration::from_millis(carla_timeout_millis));

    let map_srv = map_srv::new(&mut node, client.world())?.map(|result| result.unwrap());

    let looper = async {
        spawn_blocking(move || looper(node, client, future_tx))
            .await
            .unwrap()
            .unwrap();
    };
    let waiter = waiter(future_rx);

    // Wait for all tasks for finish
    join!(looper, waiter, map_srv);

    Ok(())
}

async fn waiter(future_rx: flume::Receiver<BoxFuture<'static, ()>>) {
    let mut futures = FuturesUnordered::new();

    loop {
        if futures.is_empty() {
            let Ok(fut) = future_rx.recv_async().await else {
                    break;
                };
            futures.push(fut);
        } else {
            select! {
                _ = futures.next() => {}
                fut = future_rx.recv_async() => {
                    let Ok(fut) = fut else {
                        break;
                    };
                    futures.push(fut);
                }
            }
        }
    }
}

// async fn forward_obj(sub: Subscriber<ObjectArray>, pub_: Publisher<DetectedObjects>) -> Result<()> {
//     sub.map(anyhow::Ok)
//         .try_fold(pub_, |pub_, in_msg| async move {
//             let ObjectArray { header, objects } = in_msg;
//             let objects = objects
//                 .into_iter()
//                 .map(|in_obj| {
//                     use AutowareClassification as TC;
//                     use AutowareShapeType as TT;
//                     use DerivedObjClassification as FC;
//                     use SolidPrimitiveType as FT;

//                     let Object {
//                         // header,
//                         // id,
//                         // detection_level,
//                         object_classified,
//                         pose,
//                         twist,
//                         // accel,
//                         polygon,
//                         shape: from_shape,
//                         classification: from_class,
//                         classification_certainty,
//                         // classification_age,
//                         ..
//                     } = in_obj;

//                     let to_class = loop {
//                         if !object_classified {
//                             break TC::UNKNOWN;
//                         }
//                         let Some(from_class) = FC::from_u8(from_class) else {
//                             log_warn!(
//                                 env!("CARGO_BIN_NAME"),
//                                 "Unsupported classification {from_class}"
//                             );
//                             break TC::UNKNOWN;
//                         };
//                         let to_class = match from_class {
//                             FC::UNKNOWN
//                             | FC::UNKNOWN_BIG
//                             | FC::UNKNOWN_MEDIUM
//                             | FC::UNKNOWN_SMALL
//                             | FC::OTHER_VEHICLE
//                             | FC::BARRIER
//                             | FC::SIGN => TC::UNKNOWN,
//                             FC::PEDESTRIAN => TC::PEDESTRIAN,
//                             FC::BIKE => TC::BICYCLE,
//                             FC::CAR => TC::CAR,
//                             FC::TRUCK => TC::TRUCK,
//                             FC::MOTORCYCLE => TC::MOTORCYCLE,
//                         };
//                         break to_class;
//                     };

//                     let to_shape = loop {
//                         let SolidPrimitive {
//                             type_: from_type,
//                             dimensions: from_dims,
//                         } = from_shape;
//                         let Some(from_type) = FT::from_u8(from_type) else {
//                             break None
//                         };
//                         let (to_type, to_dims) = match from_type {
//                             FT::BOX => {
//                                 let ty = TT::BOUNDING_BOX;
//                                 let dims = match &*from_dims {
//                                     &[x, y, z] => Vector3 { x, y, z },
//                                     _ => break None,
//                                 };
//                                 (ty, dims)
//                             }
//                             FT::CYLINDER => {
//                                 let ty = TT::CYLINDER;
//                                 let dims = match &*from_dims {
//                                     &[x, y, z] => Vector3 { x, y, z },
//                                     _ => break None,
//                                 };
//                                 (ty, dims)
//                             }
//                             FT::SPHERE | FT::CONE => break None,
//                         };

//                         break Some(Shape {
//                             type_: to_type as u8,
//                             footprint: polygon,
//                             dimensions: to_dims,
//                         });
//                     }
//                     .unwrap_or_default();

//                     DetectedObject {
//                         existence_probability: 1.0,
//                         classification: vec![ObjectClassification {
//                             label: to_class as u8,
//                             probability: classification_certainty as f32 / 255.0,
//                         }],
//                         kinematics: DetectedObjectKinematics {
//                             pose_with_covariance: PoseWithCovariance {
//                                 pose,
//                                 covariance: Array2::from_diag_elem(6, 1.0).into_raw_vec(),
//                             },
//                             has_position_covariance: true,
//                             orientation_availability: 0, // UNAVAILABLE
//                             twist_with_covariance: TwistWithCovariance {
//                                 twist,
//                                 covariance: Array2::from_diag_elem(6, 1.0).into_raw_vec(),
//                             },
//                             has_twist: true,
//                             has_twist_covariance: true,
//                         },
//                         shape: to_shape,
//                     }
//                 })
//                 .collect();
//             let out_msg = DetectedObjects { header, objects };
//             pub_.publish(&out_msg)?;
//             Ok(pub_)
//         })
//         .await?;
//     Ok(())
// }

// async fn forward_ackermann(
//     sub: Subscriber<AckermannControlCommand>,
//     pub_: Publisher<AckermannDrive>,
// ) -> Result<()> {
//     sub.map(anyhow::Ok)
//         .try_fold(pub_, |pub_, in_msg| async move {
//             let AckermannControlCommand {
//                 lateral:
//                     AckermannLateralCommand {
//                         steering_tire_angle,
//                         steering_tire_rotation_rate,
//                         ..
//                     },
//                 longitudinal:
//                     LongitudinalCommand {
//                         speed,
//                         acceleration,
//                         jerk,
//                         ..
//                     },
//                 ..
//             } = in_msg;
//             let out_msg = AckermannDrive {
//                 steering_angle: steering_tire_angle,
//                 steering_angle_velocity: steering_tire_rotation_rate,
//                 speed,
//                 acceleration,
//                 jerk,
//             };

//             pub_.publish(&out_msg)?;
//             Ok(pub_)
//         })
//         .await?;
//     Ok(())
// }

// async fn forward_odom(sub: Subscriber<Odometry>, pub_: Publisher<Odometry>) -> Result<()> {
//     sub.map(anyhow::Ok)
//         .try_fold(pub_, |pub_, msg| async move {
//             pub_.publish(&msg)?;
//             Ok(pub_)
//         })
//         .await?;
//     Ok(())
// }

// async fn forward_status(
//     info_sub: Subscriber<CarlaEgoVehicleInfo>,
//     status_sub: Subscriber<CarlaEgoVehicleStatus>,
//     accel_pub: Publisher<AccelWithCovarianceStamped>,
//     steer_pub: Publisher<SteeringReport>,
// ) -> Result<()> {
//     let mut info_sub = info_sub.fuse();
//     let mut status_sub = status_sub.fuse();
//     let (info, status) = join!(info_sub.next(), status_sub.next());
//     let Some(mut info) = info else { return Ok(()); };
//     let Some(mut status) = status else { return Ok(()); };

//     loop {
//         select! {
//             new_info = info_sub.next() => {
//                 let Some(new_info) = new_info else { break };
//                 info = new_info;
//             }
//             new_status = status_sub.next() => {
//                 let Some(new_status) = new_status else { break };
//                 status = new_status;
//             }
//         }

//         let CarlaEgoVehicleStatus {
//             ref header,
//             ref acceleration,
//             control: CarlaEgoVehicleControl { steer, .. },
//             ..
//         } = status;
//         let CarlaEgoVehicleInfo { ref wheels, .. } = info;

//         if let Some(wheel) = wheels.get(0) {
//             let CarlaEgoVehicleInfoWheel {
//                 max_steer_angle, ..
//             } = wheel;
//             let steering_tire_angle = max_steer_angle * steer;

//             let steer_msg = SteeringReport {
//                 stamp: header.stamp.clone(),
//                 steering_tire_angle,
//             };

//             steer_pub.publish(&steer_msg)?;
//         }

//         let accel_msg = AccelWithCovarianceStamped {
//             header: header.clone(),
//             accel: AccelWithCovariance {
//                 accel: acceleration.clone(),
//                 covariance: Array2::from_diag_elem(6, 1.0).into_raw_vec(),
//             },
//         };
//         accel_pub.publish(&accel_msg)?;
//     }

//     Ok(())
// }

// pub async fn forward_waypoints(sub: Subscriber<Path>, pub_: Publisher<Trajectory>) -> Result<()> {
//     let time_to_dur = |time: &Time| {
//         let Time { sec, nanosec } = *time;
//         Duration::from_nanos(sec as u64 * 1_000_000_000 + nanosec as u64)
//     };

//     sub.map(anyhow::Ok)
//         .try_fold(pub_, |pub_, in_msg| async move {
//             let Path { poses, header } = in_msg;
//             let Some(first_pose) = poses.get(0) else {
//                 return Ok(pub_);
//             };
//             let first_stamp = time_to_dur(&first_pose.header.stamp);
//             let points: Vec<_> = poses
//                 .into_iter()
//                 .map(|pose| {
//                     use builtin_interfaces::msg::Duration as RosDuration;
//                     let PoseStamped { header, pose } = pose;
//                     let elapsed = time_to_dur(&header.stamp)
//                         .checked_sub(first_stamp)
//                         .unwrap_or(Duration::ZERO);

//                     TrajectoryPoint {
//                         time_from_start: RosDuration {
//                             sec: elapsed.as_secs() as i32,
//                             nanosec: elapsed.subsec_nanos(),
//                         },
//                         pose,
//                         ..Default::default()
//                     }
//                 })
//                 .collect();
//             let out_msg = Trajectory { header, points };

//             pub_.publish(&out_msg)?;

//             Ok(pub_)
//         })
//         .await?;

//     Ok(())
// }

fn looper(
    mut node: Node,
    client: Client,
    future_tx: flume::Sender<BoxFuture<'static, ()>>,
) -> Result<()> {
    let world = client.world();

    let mut publishers: HashMap<ActorId, _> = HashMap::new();
    let bridge = Bridge::new(&mut node)?;
    let mut clock = Clock::create(ClockType::RosTime)?;

    'tick: loop {
        node.spin_once(Duration::from_millis(10));
        world.wait_for_tick();

        // Update actors
        {
            let mut actors: HashMap<ActorId, _> = world
                .actors()
                .iter()
                .map(|actor| (actor.id(), actor))
                .collect();

            let curr_keys: HashSet<_> = actors.keys().cloned().collect();
            let prev_keys: HashSet<_> = publishers.keys().cloned().collect();
            let new_keys = &curr_keys - &prev_keys;
            let del_keys = &prev_keys - &curr_keys;

            if !new_keys.is_empty() {
                log_info!(env!("CARGO_BIN_NAME"), "Adding new actors: {new_keys:?}");
            }

            if !del_keys.is_empty() {
                log_info!(env!("CARGO_BIN_NAME"), "Removing actors: {del_keys:?}");
            }

            // Inert new actors
            for id in new_keys {
                let actor = actors.remove(&id).unwrap();
                let (pub_, sub) = actor_node::new(&mut node, actor)?;
                publishers.insert(id, pub_);

                let result = future_tx.send(sub.into_future());
                if result.is_err() {
                    break 'tick;
                }
            }

            // Remove vanishing actors
            for id in del_keys {
                publishers.remove(&id).unwrap();
            }
        }

        // Poll actor publishers
        let time = Clock::to_builtin_time(&clock.get_now()?);
        publishers.iter_mut().try_for_each(|(_id, pub_)| {
            pub_.poll(&time)?;
            anyhow::Ok(())
        })?;

        // Publish tick
        bridge.tick.publish(&Empty {})?;
    }

    Ok(())
}
