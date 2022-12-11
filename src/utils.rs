use carla::{client::ActorBase, geom::BoundingBox};
use nalgebra as na;
use ndarray::Array2;
use r2r::{
    builtin_interfaces::msg::Time,
    geometry_msgs::msg::{
        Accel, AccelWithCovariance, AccelWithCovarianceStamped, Point, Point32, Pose,
        PoseWithCovariance, Quaternion, Twist, TwistWithCovariance, Vector3,
    },
    moveit_msgs::msg::OrientedBoundingBox,
    nav_msgs::msg::Odometry,
    std_msgs::msg::Header,
};

pub struct ActorPhysics {
    pub transform: na::Isometry3<f32>,
    pub velocity: na::Vector3<f32>,
    pub angular_velocity: na::Vector3<f32>,
    pub accel: na::Vector3<f32>,
    pub odom_msg: Odometry,
    pub accel_msg: AccelWithCovarianceStamped,
}

pub trait ActorExt: ActorBase {
    fn create_physics_msg(&self, time: Time) -> ActorPhysics {
        let transform = self.transform();
        let velocity = self.velocity();
        let angular_velocity = self.angular_velocity();
        let accel = self.acceleration();

        let header = Header {
            stamp: time,
            frame_id: "".to_string(),
        };
        let odom_msg = Odometry {
            header: header.clone(),
            child_frame_id: "".to_string(),
            pose: transform.to_ros_type(),
            twist: TwistWithCovariance {
                twist: Twist {
                    linear: velocity.to_ros_type(),
                    angular: angular_velocity.to_ros_type(),
                },
                covariance: identity_matrix(6).into_raw_vec(),
            },
        };

        let accel_msg = AccelWithCovarianceStamped {
            header,
            accel: AccelWithCovariance {
                accel: Accel {
                    linear: accel.to_ros_type(),
                    angular: Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                covariance: identity_matrix(6).into_raw_vec(),
            },
        };

        ActorPhysics {
            transform,
            velocity,
            angular_velocity,
            accel,
            odom_msg,
            accel_msg,
        }
    }
}

impl<T> ActorExt for T where T: ActorBase {}

pub trait ToRosType<T> {
    fn to_ros_type(&self) -> T;
}

impl ToRosType<Pose> for na::Isometry3<f64> {
    fn to_ros_type(&self) -> Pose {
        let na::Isometry3 {
            rotation,
            translation,
        } = self;

        Pose {
            position: Point {
                x: translation.x,
                y: translation.y,
                z: translation.z,
            },
            orientation: Quaternion {
                x: rotation.i,
                y: rotation.j,
                z: rotation.k,
                w: rotation.w,
            },
        }
    }
}

impl ToRosType<Pose> for na::Isometry3<f32> {
    fn to_ros_type(&self) -> Pose {
        let val: na::Isometry3<f64> = na::convert_ref(self);
        val.to_ros_type()
    }
}

impl ToRosType<PoseWithCovariance> for na::Isometry3<f64> {
    fn to_ros_type(&self) -> PoseWithCovariance {
        PoseWithCovariance {
            pose: self.to_ros_type(),
            covariance: identity_matrix(6).into_raw_vec(),
        }
    }
}

impl ToRosType<PoseWithCovariance> for na::Isometry3<f32> {
    fn to_ros_type(&self) -> PoseWithCovariance {
        let val: na::Isometry3<f64> = na::convert_ref(self);
        val.to_ros_type()
    }
}

impl ToRosType<Vector3> for na::Vector3<f64> {
    fn to_ros_type(&self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl ToRosType<Vector3> for na::Vector3<f32> {
    fn to_ros_type(&self) -> Vector3 {
        let val: na::Vector3<f64> = na::convert_ref(self);
        val.to_ros_type()
    }
}

impl ToRosType<Point32> for na::Vector3<f32> {
    fn to_ros_type(&self) -> Point32 {
        Point32 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl ToRosType<Quaternion> for na::UnitQuaternion<f64> {
    fn to_ros_type(&self) -> Quaternion {
        Quaternion {
            x: self.i,
            y: self.j,
            z: self.k,
            w: self.w,
        }
    }
}

impl ToRosType<Quaternion> for na::UnitQuaternion<f32> {
    fn to_ros_type(&self) -> Quaternion {
        let val: na::UnitQuaternion<f64> = na::convert_ref(self);
        val.to_ros_type()
    }
}

impl ToRosType<OrientedBoundingBox> for BoundingBox<f32> {
    fn to_ros_type(&self) -> OrientedBoundingBox {
        OrientedBoundingBox {
            pose: self.transform.to_ros_type(),
            extents: self.extent.to_ros_type(),
        }
    }
}

pub fn identity_matrix(size: usize) -> Array2<f64> {
    Array2::from_diag_elem(size, 1.0)
}
