use super::odom::OdomPub;
use crate::{qos, time::TimeDelta, utils::ToRosType};
use anyhow::{anyhow, Result};
use carla::{
    client::{ActorBase, Vehicle},
    geom::Vector3DExt,
    rpc::{VehicleControl, VehiclePhysicsControl, WheelPhysicsControl},
};
use carla_ackermann::{
    vehicle_control::{Output, TargetRequest},
    VehicleController,
};
use futures::{future::BoxFuture, stream, FutureExt, Stream, StreamExt};
use r2r::{
    autoware_control_msgs::msg::{Control, Lateral, Longitudinal},
    builtin_interfaces::msg::Time,
    carla_msgs::msg::{CarlaEgoVehicleControl, CarlaEgoVehicleInfo, CarlaEgoVehicleInfoWheel},
    log_warn, Node, Publisher,
};
use std::{future::IntoFuture, sync::Once};
use tokio::{spawn, sync::watch};

pub fn new(node: &mut Node, actor: Vehicle) -> Result<(VehiclePub, VehicleSub)> {
    let role_name = actor
        .attributes()
        .iter()
        .find(|attr| attr.id() == "role_name")
        .ok_or_else(|| anyhow!("The actor does not have a 'role_name' attribute"))?
        .value_string();
    let (control_tx, control_rx) = watch::channel(None);

    let physics_control = actor.physics_control();
    let controller = VehicleController::from_physics_control(&physics_control, None);

    let prefix = format!("vehicle/{role_name}");
    let vehicle_info_pub =
        node.create_publisher(&format!("{prefix}/vehicle_info"), qos::latched())?;
    let odom_pub = OdomPub::new(node, actor.clone(), &prefix)?;

    let control_sub = node.subscribe(&format!("{prefix}/control_cmd"), qos::best_effort())?;
    let ackermann_sub = node.subscribe(&format!("{prefix}/ackermann_cmd"), qos::best_effort())?;
    let forward_control = spawn(forward_control_cmd(control_sub, ackermann_sub, control_tx))
        .map(|result| result.unwrap());

    let pub_ = VehiclePub {
        actor,
        role_name,
        odom_pub,
        vehicle_info_pub,
        controller,
        physics_control,
        control_rx,
    };
    let sub = VehicleSub {
        future: forward_control.boxed(),
    };

    // Publish vehicle info once
    pub_.publish_vehicle_info()?;

    Ok((pub_, sub))
}

pub struct VehiclePub {
    actor: Vehicle,
    role_name: String,
    physics_control: VehiclePhysicsControl,
    odom_pub: OdomPub<Vehicle>,
    vehicle_info_pub: Publisher<CarlaEgoVehicleInfo>,
    controller: VehicleController,
    control_rx: watch::Receiver<Option<ControlKind>>,
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
    pub fn poll(&mut self, ros_time: &Time, time_delta: TimeDelta) -> Result<()> {
        let control_msg = (*self.control_rx.borrow()).clone();

        if let Some(command) = control_msg {
            match command {
                ControlKind::Direct(msg) => self.apply_direct_control(msg),
                ControlKind::Ackermann(msg) => self.apply_ackermann_control(msg, time_delta),
            }
        }

        self.odom_pub.poll(ros_time)?;

        Ok(())
    }

    fn publish_vehicle_info(&self) -> Result<()> {
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
            ref wheels,
            ..
        } = self.physics_control;

        let vehicle_info_msg = CarlaEgoVehicleInfo {
            id: self.actor.id(),
            type_: self.actor.type_id(),
            rolename: self.role_name.clone(),
            wheels: wheels
                .iter()
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
                    } = *wheel;
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
                .collect(),
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

        self.vehicle_info_pub.publish(&vehicle_info_msg)?;
        Ok(())
    }

    fn apply_direct_control(&mut self, msg: CarlaEgoVehicleControl) {
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
        self.actor.apply_control(&VehicleControl {
            throttle,
            steer,
            brake,
            hand_brake,
            reverse,
            manual_gear_shift,
            gear,
        });
    }

    fn apply_ackermann_control(&mut self, msg: Control, time_delta: TimeDelta) {
        let Control {
            lateral:
                Lateral {
                    steering_tire_angle,
                    is_defined_steering_tire_rotation_rate,
                    ..
                },
            longitudinal:
                Longitudinal {
                    velocity,
                    acceleration,
                    is_defined_acceleration,
                    is_defined_jerk,
                    ..
                },
            ..
        } = msg;

        if is_defined_steering_tire_rotation_rate {
            static ONCE: Once = Once::new();
            ONCE.call_once(|| {
                log_warn!(
                    env!("CARGO_BIN_NAME"),
                    "`is_defined_steering_tire_rotation_rate` is enabled but it is unsupported."
                );
            });
        }

        if is_defined_jerk {
            static ONCE: Once = Once::new();
            ONCE.call_once(|| {
                log_warn!(
                    env!("CARGO_BIN_NAME"),
                    "`is_defined_jerk` is enabled but it is unsupported."
                );
            });
        }

        let target_accel = if is_defined_acceleration {
            acceleration
        } else {
            static ONCE: Once = Once::new();
            ONCE.call_once(|| {
                log_warn!(
                    env!("CARGO_BIN_NAME"),
                    "`is_defined_acceleration` is required but it is disabled."
                );
            });
            0.0
        };

        self.controller.set_target(TargetRequest {
            steering_angle: steering_tire_angle as f64,
            speed: velocity as f64,
            accel: target_accel as f64,
        });

        let elapsed_secs = time_delta.time_delta.as_secs_f64();
        let current_speed = self.actor.velocity().norm();
        let (_, pitch_radians, _) = self.actor.transform().rotation.euler_angles();

        let (
            Output {
                throttle,
                brake,
                steer,
                reverse,
                hand_brake,
            },
            _,
        ) = self
            .controller
            .step(elapsed_secs, current_speed as f64, pitch_radians as f64);

        self.actor.apply_control(&VehicleControl {
            throttle: throttle as f32,
            steer: steer as f32,
            brake: brake as f32,
            hand_brake,
            reverse,
            manual_gear_shift: false,
            gear: 0,
        });
    }
}

#[derive(Debug, Clone)]
enum ControlKind {
    Direct(CarlaEgoVehicleControl),
    Ackermann(Control),
}

impl From<Control> for ControlKind {
    fn from(v: Control) -> Self {
        Self::Ackermann(v)
    }
}

impl From<CarlaEgoVehicleControl> for ControlKind {
    fn from(v: CarlaEgoVehicleControl) -> Self {
        Self::Direct(v)
    }
}

async fn forward_control_cmd(
    control_stream: impl Stream<Item = CarlaEgoVehicleControl> + Unpin,
    ackermann_stream: impl Stream<Item = Control> + Unpin,
    control_tx: watch::Sender<Option<ControlKind>>,
) {
    let mut stream = stream::select(
        control_stream.map(ControlKind::from),
        ackermann_stream.map(ControlKind::from),
    );

    while let Some(msg) = stream.next().await {
        let ok = control_tx.send(Some(msg)).is_ok();
        if !ok {
            break;
        }
    }
}
