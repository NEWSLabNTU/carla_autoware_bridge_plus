use super::odom::OdomPub;
use crate::{qos, utils::ToRosType};
use anyhow::Result;
use carla::client::{ActorBase, TrafficSign};
use r2r::{builtin_interfaces::msg::Time, moveit_msgs::msg::OrientedBoundingBox, Node, Publisher};

pub fn new(node: &mut Node, actor: TrafficSign) -> Result<(TrafficSignPub, TrafficSignSub)> {
    let actor_id = actor.id();
    let prefix = format!("traffic_sign/id_{actor_id}");
    let odom_pub = OdomPub::new(node, actor.clone(), &prefix)?;
    let trigger_volume_pub =
        node.create_publisher(&format!("{prefix}/trigger_volume"), qos::best_effort())?;
    let pub_ = TrafficSignPub {
        odom_pub,
        trigger_volume_pub,
        actor,
    };
    let sub = TrafficSignSub {};
    Ok((pub_, sub))
}

pub struct TrafficSignPub {
    actor: TrafficSign,
    odom_pub: OdomPub<TrafficSign>,
    trigger_volume_pub: Publisher<OrientedBoundingBox>,
}

impl TrafficSignPub {
    pub fn poll(&mut self, time: &Time) -> Result<()> {
        let bbox = self.actor.trigger_volume();
        let bbox_msg = bbox.to_ros_type();

        self.trigger_volume_pub.publish(&bbox_msg)?;
        self.odom_pub.poll(time)?;
        Ok(())
    }
}

pub struct TrafficSignSub {}
