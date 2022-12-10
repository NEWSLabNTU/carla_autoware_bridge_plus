use super::odom::OdomPub;
use crate::{
    types::{PointFieldType, SensorType},
    utils::{identity_matrix, ToRosType},
};
use anyhow::Result;
use carla::{
    client::{ActorBase, Sensor},
    geom::Location,
    sensor::data::{
        CollisionEvent, Color, Image as CarlaImage, ImuMeasurement, LidarDetection,
        LidarMeasurement, SemanticLidarDetection, SemanticLidarMeasurement,
    },
};
use nalgebra::UnitQuaternion;
use r2r::{
    builtin_interfaces::msg::Time,
    log_warn,
    sensor_msgs::msg::{Image as RosImage, Imu, PointCloud2, PointField},
    std_msgs::msg::{Header, String as RosString},
    Clock, ClockType, Node, Publisher, QosProfile,
};
use std::{mem, ops::RangeFrom};

pub fn new(node: &mut Node, actor: Sensor) -> Result<(SensorPub, SensorSub)> {
    let actor_id = actor.id();
    let type_id = actor.type_id();
    let type_: Option<SensorType> = type_id.parse().ok();
    let prefix = format!("sensor/id_{actor_id}");

    if let Some(type_) = type_ {
        use SensorType as T;

        let qos = QosProfile::default();
        let mut clock = Clock::create(ClockType::RosTime)?;
        let mut frame_counter = 0..;

        let mut next_header = move || {
            let frame_id = frame_counter.next().unwrap();
            let time = clock.get_now()?;
            let time = Clock::to_builtin_time(&time);
            let header = Header {
                stamp: time,
                frame_id: frame_id.to_string(),
            };
            anyhow::Ok(header)
        };

        match type_ {
            T::CameraRgb => {
                let mut pub_ = node.create_publisher(&format!("{prefix}/image"), qos)?;

                actor.listen(move |data| {
                    let header = next_header().unwrap();
                    camera_callback(header, data.try_into().unwrap(), &mut pub_);
                });
            }
            T::LidarRayCast => {
                let mut pub_ = node.create_publisher(&format!("{prefix}/pointcloud"), qos)?;

                actor.listen(move |data| {
                    let header = next_header().unwrap();
                    lidar_callback(header, data.try_into().unwrap(), &mut pub_);
                });
            }
            T::LidarRayCastSemantic => {
                let mut pub_ =
                    node.create_publisher(&format!("{prefix}/semantic_pointcloud"), qos)?;

                actor.listen(move |data| {
                    let header = next_header().unwrap();
                    semantic_lidar_callback(header, data.try_into().unwrap(), &mut pub_);
                });
            }
            T::Imu => {
                let mut pub_ = node.create_publisher(&format!("{prefix}/imu"), qos)?;

                actor.listen(move |data| {
                    let header = next_header().unwrap();
                    imu_callback(header, data.try_into().unwrap(), &mut pub_);
                });
            }
            T::Collision => {
                let mut pub_ = node.create_publisher(&format!("{prefix}/event"), qos)?;
                actor.listen(move |data| {
                    let header = next_header().unwrap();
                    collision_callback(header, data.try_into().unwrap(), &mut pub_);
                });
            }
        }
    } else {
        log_warn!(
            env!("CARGO_BIN_NAME"),
            "Unsupported sensor type '{}'",
            type_id
        );
    }

    let qos = QosProfile::default();
    let type_pub = node.create_publisher(&format!("{prefix}/type"), qos)?;
    let odom_pub = OdomPub::new(node, actor, &prefix)?;
    let pub_ = SensorPub {
        frame_counter: 0..,
        type_id,
        type_pub,
        odom_pub,
    };
    let sub = SensorSub {};
    Ok((pub_, sub))
}

pub struct SensorPub {
    frame_counter: RangeFrom<usize>,
    type_id: String,
    type_pub: Publisher<RosString>,
    odom_pub: OdomPub<Sensor>,
}

impl SensorPub {
    pub fn poll(&mut self, time: &Time) -> Result<()> {
        let frame_id = self.frame_counter.next().unwrap();
        let type_msg = RosString {
            data: self.type_id.to_string(),
        };
        self.type_pub.publish(&type_msg)?;
        self.odom_pub.poll(time, frame_id)?;
        Ok(())
    }
}

pub struct SensorSub {}

fn camera_callback(header: Header, image: CarlaImage, pub_: &mut Publisher<RosImage>) {
    let slice = image.as_slice();
    if slice.is_empty() {
        return;
    }
    let width = image.width();
    let height = image.height();
    let data: Vec<_> = slice
        .iter()
        .flat_map(|&Color { b, g, r, a }| [b, g, r, a])
        .collect();

    let msg = RosImage {
        header,
        height: height as u32,
        width: width as u32,
        encoding: "bgra8".to_string(),
        is_bigendian: is_bigendian().into(),
        step: (width * 4) as u32,
        data,
    };

    pub_.publish(&msg).unwrap();
}

fn lidar_callback(header: Header, measure: LidarMeasurement, pub_: &mut Publisher<PointCloud2>) {
    let slice = measure.as_slice();
    if slice.is_empty() {
        return;
    }
    let point_step = mem::size_of_val(&slice[0]);
    let row_step = slice.len();
    let data: Vec<_> = slice
        .iter()
        .flat_map(
            |&LidarDetection {
                 point: Location { x, y, z },
                 intensity,
             }| { [x, y, z, intensity] },
        )
        .flat_map(|elem| elem.to_ne_bytes())
        .collect();
    let fields = vec![
        PointField {
            name: "x".to_string(),
            offset: 0,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "y".to_string(),
            offset: 4,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "z".to_string(),
            offset: 8,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "intensity".to_string(),
            offset: 12,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
    ];

    let msg = PointCloud2 {
        header,
        height: 1,
        width: slice.len() as u32,
        fields,
        is_bigendian: is_bigendian(),
        point_step: point_step as u32,
        row_step: row_step as u32,
        data,
        is_dense: true,
    };

    pub_.publish(&msg).unwrap();
}

fn semantic_lidar_callback(
    header: Header,
    measure: SemanticLidarMeasurement,
    pub_: &mut Publisher<PointCloud2>,
) {
    let slice = measure.as_slice();
    if slice.is_empty() {
        return;
    }
    let point_step = mem::size_of_val(&slice[0]);
    let row_step = slice.len();
    let data: Vec<_> = slice
        .iter()
        .flat_map(
            |&SemanticLidarDetection {
                 point: Location { x, y, z },
                 cos_inc_angle,
                 object_idx,
                 object_tag,
             }| {
                [
                    x.to_ne_bytes(),
                    y.to_ne_bytes(),
                    z.to_ne_bytes(),
                    cos_inc_angle.to_ne_bytes(),
                    object_idx.to_ne_bytes(),
                    object_tag.to_ne_bytes(),
                ]
            },
        )
        .flatten()
        .collect();
    let fields = vec![
        PointField {
            name: "x".to_string(),
            offset: 0,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "y".to_string(),
            offset: 4,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "z".to_string(),
            offset: 8,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "cos_inc_angle".to_string(),
            offset: 12,
            datatype: PointFieldType::FLOAT32 as u8,
            count: 1,
        },
        PointField {
            name: "object_idx".to_string(),
            offset: 16,
            datatype: PointFieldType::UINT32 as u8,
            count: 1,
        },
        PointField {
            name: "object_tag".to_string(),
            offset: 20,
            datatype: PointFieldType::UINT32 as u8,
            count: 1,
        },
    ];

    let msg = PointCloud2 {
        header,
        height: 1,
        width: slice.len() as u32,
        fields,
        is_bigendian: is_bigendian(),
        point_step: point_step as u32,
        row_step: row_step as u32,
        data,
        is_dense: true,
    };

    pub_.publish(&msg).unwrap();
}

fn imu_callback(header: Header, measure: ImuMeasurement, pub_: &mut Publisher<Imu>) {
    let accel = measure.accelerometer();
    let compass = measure.compass();
    let orientation = UnitQuaternion::from_euler_angles(0.0, 0.0, -compass);
    let gyro = measure.gyroscope();

    let msg = Imu {
        header,
        orientation: orientation.to_ros_type(),
        orientation_covariance: identity_matrix(3).into_raw_vec(),
        angular_velocity: gyro.to_ros_type(),
        angular_velocity_covariance: identity_matrix(3).into_raw_vec(),
        linear_acceleration: accel.to_ros_type(),
        linear_acceleration_covariance: identity_matrix(3).into_raw_vec(),
    };

    pub_.publish(&msg).unwrap();
}

fn collision_callback(header: Header, _event: CollisionEvent, pub_: &mut Publisher<Time>) {
    pub_.publish(&header.stamp).unwrap();
}

const fn is_bigendian() -> bool {
    cfg!(target_endian = "big")
}
