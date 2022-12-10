use super::odom::OdomPub;
use crate::utils::ToRosType;
use anyhow::Result;
use carla::client::{ActorBase, TrafficSign};
use r2r::{
    builtin_interfaces::msg::Time, moveit_msgs::msg::OrientedBoundingBox, Node, Publisher,
    QosProfile,
};
use std::ops::RangeFrom;

pub fn new(node: &mut Node, actor: TrafficSign) -> Result<(TrafficSignPub, TrafficSignSub)> {
    let actor_id = actor.id();
    let prefix = format!("traffic_sign/id_{actor_id}");
    let qos = QosProfile::default();
    let odom_pub = OdomPub::new(node, actor.clone(), &prefix)?;
    let trigger_volume_pub = node.create_publisher(&format!("{prefix}/trigger_volume"), qos)?;
    let pub_ = TrafficSignPub {
        frame_counter: 0..,
        odom_pub,
        trigger_volume_pub,
        actor,
    };
    let sub = TrafficSignSub {};
    Ok((pub_, sub))
}

pub struct TrafficSignPub {
    actor: TrafficSign,
    frame_counter: RangeFrom<usize>,
    odom_pub: OdomPub<TrafficSign>,
    trigger_volume_pub: Publisher<OrientedBoundingBox>,
}

impl TrafficSignPub {
    pub fn poll(&mut self, time: &Time) -> Result<()> {
        let frame_id = self.frame_counter.next().unwrap();
        let bbox = self.actor.trigger_volume();
        let bbox_msg = bbox.to_ros_type();

        self.trigger_volume_pub.publish(&bbox_msg)?;
        self.odom_pub.poll(time, frame_id)?;
        Ok(())
    }
}

pub struct TrafficSignSub {}
