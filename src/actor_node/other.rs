use super::odom::OdomPub;
use anyhow::Result;
use carla::client::{Actor, ActorBase};
use r2r::{builtin_interfaces::msg::Time, Node};
use std::ops::RangeFrom;

pub fn new(node: &mut Node, actor: Actor) -> Result<(OtherPub, OtherSub)> {
    let actor_id = actor.id();
    let odom_pub = OdomPub::new(node, actor, &format!("other/id_{actor_id}"))?;
    let pub_ = OtherPub {
        frame_counter: 0..,
        odom_pub,
    };
    let sub = OtherSub {};
    Ok((pub_, sub))
}

pub struct OtherPub {
    frame_counter: RangeFrom<usize>,
    odom_pub: OdomPub<Actor>,
}

impl OtherPub {
    pub fn poll(&mut self, time: &Time) -> Result<()> {
        let frame_id = self.frame_counter.next().unwrap();
        self.odom_pub.poll(time, frame_id)?;
        Ok(())
    }
}

pub struct OtherSub {}
