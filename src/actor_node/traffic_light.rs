use super::odom::OdomPub;
use crate::{
    types::{TrafficLightColor, TrafficLightShape, TrafficLightStatus},
    utils::ToRosType,
};
use anyhow::Result;
use carla::{
    client::{ActorBase, TrafficLight as TrafficLightActor},
    rpc::TrafficLightState,
};
use r2r::{
    autoware_auto_perception_msgs::msg::{TrafficLight, TrafficSignal, TrafficSignalStamped},
    builtin_interfaces::msg::Time,
    moveit_msgs::msg::OrientedBoundingBox,
    std_msgs::msg::Header,
    Node, Publisher, QosProfile,
};

pub fn new(
    node: &mut Node,
    actor: TrafficLightActor,
) -> Result<(TrafficLightPub, TrafficLightSub)> {
    let actor_id = actor.id();
    let prefix = format!("traffic_light/id_{actor_id}");
    let qos = QosProfile::default();
    let status_pub = node.create_publisher(&format!("{prefix}/status"), qos.clone())?;
    let trigger_volume_pub = node.create_publisher(&format!("{prefix}/trigger_volume"), qos)?;
    let odom_pub = OdomPub::new(node, actor.clone(), &prefix)?;
    let pub_ = TrafficLightPub {
        actor,
        odom_pub,
        status_pub,
        trigger_volume_pub,
    };
    let sub = TrafficLightSub {};
    Ok((pub_, sub))
}

pub struct TrafficLightPub {
    actor: TrafficLightActor,
    odom_pub: OdomPub<TrafficLightActor>,
    status_pub: Publisher<TrafficSignalStamped>,
    trigger_volume_pub: Publisher<OrientedBoundingBox>,
}

impl TrafficLightPub {
    pub fn poll(&mut self, time: &Time) -> Result<()> {
        let bbox = self.actor.trigger_volume();
        let (rbit, ybit, gbit) = match self.actor.state() {
            TrafficLightState::Red => (true, false, false),
            TrafficLightState::Yellow => (false, true, false),
            TrafficLightState::Green => (false, false, true),
            _ => return Ok(()),
        };

        let to_status = |yes: bool| {
            if yes {
                TrafficLightStatus::SOLID_ON
            } else {
                TrafficLightStatus::SOLID_OFF
            }
        };
        let rstatus = to_status(rbit);
        let ystatus = to_status(ybit);
        let gstatus = to_status(gbit);

        let header = Header {
            stamp: time.clone(),
            frame_id: "".to_string(),
        };
        let status_msg = TrafficSignalStamped {
            header,
            signal: TrafficSignal {
                map_primitive_id: 0,
                lights: vec![
                    TrafficLight {
                        color: TrafficLightColor::RED as u8,
                        shape: TrafficLightShape::CIRCLE as u8,
                        status: rstatus as u8,
                        confidence: 0.0,
                    },
                    TrafficLight {
                        color: TrafficLightColor::AMBER as u8,
                        shape: TrafficLightShape::CIRCLE as u8,
                        status: ystatus as u8,
                        confidence: 0.0,
                    },
                    TrafficLight {
                        color: TrafficLightColor::GREEN as u8,
                        shape: TrafficLightShape::CIRCLE as u8,
                        status: gstatus as u8,
                        confidence: 0.0,
                    },
                ],
            },
        };
        let bbox_msg = bbox.to_ros_type();

        self.odom_pub.poll(time)?;
        self.status_pub.publish(&status_msg)?;
        self.trigger_volume_pub.publish(&bbox_msg)?;

        Ok(())
    }
}

pub struct TrafficLightSub {}
