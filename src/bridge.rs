use crate::qos;
use anyhow::Result;
use r2r::{std_msgs::msg::Empty, Node, Publisher};

/// Serves ROS topics about the simulation runtime.
pub struct Bridge {
    pub tick: Publisher<Empty>,
}

impl Bridge {
    pub fn new(node: &mut Node) -> Result<Self> {
        Ok(Self {
            tick: node.create_publisher("tick", qos::best_effort())?,
        })
    }
}
