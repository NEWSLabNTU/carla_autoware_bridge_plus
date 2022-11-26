use num_derive::FromPrimitive;

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
