use super::odom::OdomPub;
use crate::{qos, utils::ToRosType};
use anyhow::{anyhow, Result};
use carla::{
    client::{ActorBase, Vehicle},
    geom::Vector3DExt,
    rpc::{VehicleControl, VehiclePhysicsControl, WheelPhysicsControl},
};
use futures::{future::BoxFuture, FutureExt, Stream, StreamExt};
use r2r::{
    builtin_interfaces::msg::Time,
    carla_msgs::msg::{CarlaEgoVehicleControl, CarlaEgoVehicleInfo, CarlaEgoVehicleInfoWheel},
    Node, Publisher,
};
use std::future::IntoFuture;
use tokio::spawn;
// use tokio::sync::watch;

pub fn new(node: &mut Node, actor: Vehicle) -> Result<(VehiclePub, VehicleSub)> {
    let role_name = actor
        .attributes()
        .iter()
        .find(|attr| attr.id() == "role_name")
        .ok_or_else(|| anyhow!("The actor does not have a 'role_name' attribute"))?
        .value_string();
    // let (shared_tx, shared_rx) = watch::channel(None);

    let prefix = format!("vehicle/{role_name}");
    let vehicle_info_pub =
        node.create_publisher(&format!("{prefix}/vehicle_info"), qos::best_effort())?;
    let odom_pub = OdomPub::new(node, actor.clone(), &prefix)?;
    let acker_sub = node.subscribe(&format!("{prefix}/control_cmd"), qos::best_effort())?;
    let forward_ackermann = spawn(forward_control_cmd(
        acker_sub,
        actor.clone(), // shared_rx
    ))
    .map(|result| result.unwrap());

    let pub_ = VehiclePub {
        actor,
        role_name,
        // shared_tx,
        odom_pub,
        vehicle_info_pub,
    };
    let sub = VehicleSub {
        future: forward_ackermann.boxed(),
    };

    Ok((pub_, sub))
}

pub struct VehiclePub {
    actor: Vehicle,
    role_name: String,
    // shared_tx: watch::Sender<Option<Shared>>,
    odom_pub: OdomPub<Vehicle>,
    vehicle_info_pub: Publisher<CarlaEgoVehicleInfo>,
}

pub struct VehicleSub {
    future: BoxFuture<'static, ()>,
}

impl IntoFuture for VehicleSub {
    type Output = ();
    type IntoFuture = BoxFuture<'static, ()>;

    fn into_future(self) -> Self::IntoFuture {
        self.future
    }
}

impl VehiclePub {
    pub fn poll(&mut self, time: &Time) -> Result<()> {
        let VehiclePhysicsControl {
            max_rpm,
            moi,
            damping_rate_full_throttle,
            damping_rate_zero_throttle_clutch_engaged,
            damping_rate_zero_throttle_clutch_disengaged,
            use_gear_autobox,
            gear_switch_time,
            clutch_strength,
            mass,
            drag_coefficient,
            center_of_mass,
            wheels,
            ..
        } = self.actor.physics_control();

        let wheels_msg = wheels
            .into_iter()
            .map(|wheel| {
                let WheelPhysicsControl {
                    tire_friction,
                    damping_rate,
                    max_steer_angle,
                    radius,
                    max_brake_torque,
                    max_handbrake_torque,
                    ref position,
                    ..
                } = wheel;
                CarlaEgoVehicleInfoWheel {
                    tire_friction,
                    damping_rate,
                    max_steer_angle,
                    radius,
                    max_brake_torque,
                    max_handbrake_torque,
                    position: position.to_na().to_ros_type(),
                }
            })
            .collect();

        let vehicle_info_msg = CarlaEgoVehicleInfo {
            id: self.actor.id(),
            type_: self.actor.type_id(),
            rolename: self.role_name.clone(),
            wheels: wheels_msg,
            max_rpm,
            moi,
            damping_rate_full_throttle,
            damping_rate_zero_throttle_clutch_engaged,
            damping_rate_zero_throttle_clutch_disengaged,
            use_gear_autobox,
            gear_switch_time,
            clutch_strength,
            mass,
            drag_coefficient,
            center_of_mass: center_of_mass.vector.to_ros_type(),
        };

        self.odom_pub.poll(time)?;
        self.vehicle_info_pub.publish(&vehicle_info_msg)?;

        // let transform = self.actor.transform();
        // let ok = self
        //     .shared_tx
        //     .send(Some(Shared {
        //         velocity,
        //         accel,
        //         transform,
        //         physics_control,
        //     }))
        //     .is_ok();

        Ok(())
    }
}

// struct Shared {
//     pub velocity: Vector3<f32>,
//     pub accel: Vector3<f32>,
//     pub transform: Isometry3<f32>,
//     pub physics_control: VehiclePhysicsControl,
// }

async fn forward_control_cmd(
    mut stream: impl Stream<Item = CarlaEgoVehicleControl> + Unpin,
    mut actor: Vehicle,
    // _shared_rx: watch::Receiver<Option<Shared>>,
) {
    while let Some(msg) = stream.next().await {
        let CarlaEgoVehicleControl {
            throttle,
            steer,
            brake,
            hand_brake,
            reverse,
            gear,
            manual_gear_shift,
            ..
        } = msg;
        actor.apply_control(&VehicleControl {
            throttle,
            steer,
            brake,
            hand_brake,
            reverse,
            manual_gear_shift,
            gear,
        });
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum SpeedStatus {
//     FullStop,
//     StandStill,
//     Driving,
// }

// impl SpeedStatus {
//     pub fn from_speed(speed: f32) -> Self {
//         let abs = speed.abs();

//         let standing_still_epsilon = 0.1;
//         let full_stop_epsilon = 0.00001;

//         if abs < full_stop_epsilon {
//             Self::FullStop
//         } else if abs < standing_still_epsilon {
//             Self::StandStill
//         } else {
//             Self::Driving
//         }
//     }
// }
