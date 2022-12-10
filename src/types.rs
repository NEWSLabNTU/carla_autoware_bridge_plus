use anyhow::bail;
use num_derive::FromPrimitive;
use std::str::FromStr;

// pub type Subscriber<T> = Pin<Box<dyn Stream<Item = T>>>;
// pub type Service<T> = Pin<Box<dyn Stream<Item = ServiceRequest<T>>>>;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum PointFieldType {
    INT8 = 1,
    UINT8 = 2,
    INT16 = 3,
    UINT16 = 4,
    INT32 = 5,
    UINT32 = 6,
    FLOAT32 = 7,
    FLOAT64 = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    CameraRgb,
    LidarRayCast,
    LidarRayCastSemantic,
    Imu,
    Collision,
}

impl FromStr for SensorType {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(match text {
            "sensor.camera.rgb" => Self::CameraRgb,
            "sensor.lidar.ray_cast" => Self::LidarRayCast,
            "sensor.lidar.ray_cast_semantic" => Self::LidarRayCastSemantic,
            "sensor.other.imu" => Self::Imu,
            "sensor.other.collision" => Self::Collision,
            _ => bail!("Unsupported type '{}'", text),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum TrafficLightColor {
    RED = 1,
    AMBER = 2,
    GREEN = 3,
    WHITE = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum TrafficLightShape {
    CIRCLE = 5,
    LEFT_ARROW = 6,
    RIGHT_ARROW = 7,
    UP_ARROW = 8,
    DOWN_ARROW = 9,
    DOWN_LEFT_ARROW = 10,
    DOWN_RIGHT_ARROW = 11,
    CROSS = 12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum TrafficLightStatus {
    SOLID_OFF = 13,
    SOLID_ON = 14,
    FLASHING = 15,
}
