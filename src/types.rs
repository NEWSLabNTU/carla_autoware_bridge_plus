use anyhow::{bail, Result};
use futures::{Stream, StreamExt};
use num_derive::FromPrimitive;
use r2r::{
    ackermann_msgs::msg::AckermannDrive,
    autoware_auto_control_msgs::msg::AckermannControlCommand,
    autoware_auto_perception_msgs::msg::DetectedObjects,
    autoware_auto_planning_msgs::msg::Trajectory,
    autoware_auto_vehicle_msgs::{msg::SteeringReport, srv::ControlModeCommand},
    carla_msgs::{
        msg::{CarlaEgoVehicleInfo, CarlaEgoVehicleStatus, CarlaStatus, CarlaWorldInfo},
        srv::GetBlueprints,
    },
    derived_object_msgs::msg::ObjectArray,
    geometry_msgs::msg::AccelWithCovarianceStamped,
    nav_msgs::msg::{Odometry, Path},
    rcl_interfaces::srv::{
        DescribeParameters, GetParameterTypes, GetParameters, ListParameters, SetParameters,
        SetParametersAtomically,
    },
    tier4_external_api_msgs::srv::InitializePose,
    Client, Context, Node, ParameterValue, Publisher, QosProfile, ServiceRequest,
};
use std::pin::Pin;

pub type Subscriber<T> = Pin<Box<dyn Stream<Item = T>>>;
pub type Service<T> = Pin<Box<dyn Stream<Item = ServiceRequest<T>>>>;

pub(crate) struct CarlaAutowareBridge {
    pub node: Node,
    pub autoware_pub: AutowarePublishers,
    pub autoware_sub: AutowareSubscribers,
    pub carla_pub: CarlaPublishers,
    pub carla_sub: CarlaSubscribers,
    pub carla_sc: CarlaServiceClients,
    pub srv: Services,
}

impl CarlaAutowareBridge {
    pub fn new() -> Result<Self> {
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

        let qos = QosProfile::default();

        Ok(Self {
            autoware_pub: AutowarePublishers {
                accel: node.create_publisher("/localization/acceleration", qos.clone())?,
                odom: node.create_publisher("/localization/kinematic_state", qos.clone())?,
                obj: node.create_publisher(
                    "/perception/object_recognition/detection/objects",
                    qos.clone(),
                )?,
                steer_status: node
                    .create_publisher("/vehicle/status/steering_status", qos.clone())?,
                trajectory: node
                    .create_publisher("/planning/scenario_planning/trajectory", qos.clone())?,
            },
            autoware_sub: AutowareSubscribers {
                control_cmd: node
                    .subscribe("/control/command/control_cmd", qos.clone())?
                    .boxed_local(),
            },
            carla_pub: CarlaPublishers {
                ackermann: node.create_publisher(
                    &format!("/carla/{}/ackermann_cmd", role_name),
                    qos.clone(),
                )?,
            },
            carla_sub: CarlaSubscribers {
                status: node.subscribe("/carla/status", qos.clone())?.boxed_local(),
                world_info: node
                    .subscribe("/carla/world_info", qos.clone())?
                    .boxed_local(),
                obj: node
                    .subscribe(&format!("/carla/{}/objects", role_name), qos.clone())?
                    .boxed_local(),
                odom: node
                    .subscribe(&format!("/carla/{}/odometry", role_name), qos.clone())?
                    .boxed_local(),
                vehicle_status: node
                    .subscribe(&format!("/carla/{}/vehicle_status", role_name), qos.clone())?
                    .boxed_local(),
                vehicle_info: node
                    .subscribe(&format!("/carla/{}/vehicle_info", role_name), qos.clone())?
                    .boxed_local(),
                waypoints: node
                    .subscribe(&format!("/carla/{}/waypoints", role_name), qos.clone())?
                    .boxed_local(),
            },
            srv: Services {
                init_pose: node
                    .create_service("/api/simulator/set/pose")?
                    .boxed_local(),
                control_mode: node
                    .create_service("/control/control_mode_request")?
                    .boxed_local(),
                desc_param: node
                    .create_service("/simulation/simple_planning_simulator/describe_parameters")?
                    .boxed_local(),
                get_param_ty: node
                    .create_service("/simulation/simple_planning_simulator/get_parameter_types")?
                    .boxed_local(),
                get_param: node
                    .create_service("/simulation/simple_planning_simulator/get_parameters")?
                    .boxed_local(),
                list_param: node
                    .create_service("/simulation/simple_planning_simulator/list_parameters")?
                    .boxed_local(),
                set_param: node
                    .create_service("/simulation/simple_planning_simulator/set_parameters")?
                    .boxed_local(),
                set_param_atomic: node
                    .create_service(
                        "/simulation/simple_planning_simulator/set_parameters_atomically",
                    )?
                    .boxed_local(),
            },
            carla_sc: CarlaServiceClients {
                get_blueprints: node.create_client("/carla/get_blueprints")?,
            },
            node,
        })
    }
}

pub(crate) struct AutowarePublishers {
    pub accel: Publisher<AccelWithCovarianceStamped>,
    pub odom: Publisher<Odometry>,
    pub obj: Publisher<DetectedObjects>,
    pub steer_status: Publisher<SteeringReport>,
    pub trajectory: Publisher<Trajectory>,
}

pub(crate) struct AutowareSubscribers {
    pub control_cmd: Subscriber<AckermannControlCommand>,
}

pub(crate) struct CarlaPublishers {
    pub ackermann: Publisher<AckermannDrive>,
}

pub(crate) struct CarlaSubscribers {
    pub status: Subscriber<CarlaStatus>,
    pub world_info: Subscriber<CarlaWorldInfo>,
    pub obj: Subscriber<ObjectArray>,
    pub odom: Subscriber<Odometry>,
    pub vehicle_status: Subscriber<CarlaEgoVehicleStatus>,
    pub vehicle_info: Subscriber<CarlaEgoVehicleInfo>,
    pub waypoints: Subscriber<Path>,
}

pub(crate) struct CarlaServiceClients {
    pub get_blueprints: Client<GetBlueprints::Service>,
}

pub(crate) struct Services {
    pub init_pose: Service<InitializePose::Service>,
    pub control_mode: Service<ControlModeCommand::Service>,
    pub desc_param: Service<DescribeParameters::Service>,
    pub get_param_ty: Service<GetParameterTypes::Service>,
    pub get_param: Service<GetParameters::Service>,
    pub list_param: Service<ListParameters::Service>,
    pub set_param: Service<SetParameters::Service>,
    pub set_param_atomic: Service<SetParametersAtomically::Service>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum DerivedObjClassification {
    UNKNOWN = 0,
    UNKNOWN_SMALL = 1,
    UNKNOWN_MEDIUM = 2,
    UNKNOWN_BIG = 3,
    PEDESTRIAN = 4,
    BIKE = 5,
    CAR = 6,
    TRUCK = 7,
    MOTORCYCLE = 8,
    OTHER_VEHICLE = 9,
    BARRIER = 10,
    SIGN = 11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum AutowareClassification {
    UNKNOWN = 0,
    CAR = 1,
    TRUCK = 2,
    BUS = 3,
    TRAILER = 4,
    MOTORCYCLE = 5,
    BICYCLE = 6,
    PEDESTRIAN = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum SolidPrimitiveType {
    BOX = 1,
    SPHERE = 2,
    CYLINDER = 3,
    CONE = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum AutowareShapeType {
    BOUNDING_BOX = 0,
    CYLINDER = 1,
    POLYGON = 2,
}
