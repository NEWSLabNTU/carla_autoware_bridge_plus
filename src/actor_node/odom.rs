use crate::{
    qos,
    utils::{ActorExt, ActorPhysics},
};
use anyhow::Result;
use carla::client::ActorBase;
use r2r::{
    builtin_interfaces::msg::Time, geometry_msgs::msg::AccelWithCovarianceStamped,
    nav_msgs::msg::Odometry, Node, Publisher,
};

pub struct OdomPub<T>
where
    T: ActorBase,
{
    actor: T,
    accel: Publisher<AccelWithCovarianceStamped>,
    odom: Publisher<Odometry>,
}

impl<T> OdomPub<T>
where
    T: ActorBase,
{
    pub fn new(node: &mut Node, actor: T, prefix: &str) -> Result<Self> {
        Ok(OdomPub {
            actor,
            accel: node.create_publisher(&format!("{prefix}/acceleration"), qos::best_effort())?,
            odom: node.create_publisher(&format!("{prefix}/odometry"), qos::best_effort())?,
        })
    }

    pub fn poll(&mut self, time: &Time) -> Result<ActorPhysics> {
        let physics = self.actor.create_physics_msg(time.clone());
        let ActorPhysics {
            odom_msg,
            accel_msg,
            ..
        } = &physics;
        self.odom.publish(odom_msg)?;
        self.accel.publish(accel_msg)?;
        Ok(physics)
    }
}
