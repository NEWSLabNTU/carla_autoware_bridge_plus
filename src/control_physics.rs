use carla::rpc::VehiclePhysicsControl;
use noisy_float::types::r32;

pub struct VehiclePhysics {
    pub engine_brake_force: f32,
    pub slope_force: f32,
    pub lay_off_engine_acceleration: f32,
    pub weight_force: f32,
    pub rolling_resistance_force: f32,
    pub aerodynamic_drag_force: f32,
    pub driving_impedance_acceleration: f32,
    pub max_steering_angle: f32,
    pub max_speed: f32,
    pub max_accel: f32,
    pub max_deceleration: f32,
}

impl VehiclePhysics {
    pub fn new(
        physics_control: &VehiclePhysicsControl,
        speed: f32,
        tilt_angle: f32,
        reverse: bool,
    ) -> Self {
        let VehiclePhysicsControl {
            mass, ref wheels, ..
        } = *physics_control;
        let speed_squared = speed.powi(2);

        let rolling_resistance_coefficient = 0.01;
        let acceleration_of_gravity = 9.81;

        let engine_brake_force = 500.0;
        let slope_force = -acceleration_of_gravity * mass * tilt_angle.sin();
        let lay_off_engine_acceleration = -engine_brake_force / mass;
        let weight_force = mass * acceleration_of_gravity;
        let rolling_resistance_force = rolling_resistance_coefficient * weight_force;
        let aerodynamic_drag_force = {
            let default_aerodynamic_drag_coefficient = 0.3;
            let default_drag_reference_area = 2.37;
            let drag_area = default_aerodynamic_drag_coefficient * default_drag_reference_area;
            let rho_air_25 = 1.184;
            0.5 * drag_area * rho_air_25 * speed_squared
        };
        let driving_impedance_acceleration = {
            let slope_force = if reverse { -slope_force } else { slope_force };
            -(rolling_resistance_force + aerodynamic_drag_force + slope_force) / mass
        };
        let max_steering_angle = wheels
            .iter()
            .map(|wheel| r32(wheel.max_steer_angle))
            .min()
            .map(|min| min.raw())
            .unwrap_or_else(|| 70f32.to_radians());
        let max_speed = 180.0 / 3.6;
        let max_accel = 3.0;
        let max_deceleration = 8.0;

        Self {
            engine_brake_force,
            slope_force,
            lay_off_engine_acceleration,
            weight_force,
            rolling_resistance_force,
            aerodynamic_drag_force,
            driving_impedance_acceleration,
            max_steering_angle,
            max_speed,
            max_accel,
            max_deceleration,
        }
    }
}
