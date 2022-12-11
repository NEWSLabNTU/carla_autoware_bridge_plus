use anyhow::Result;
use r2r::{std_msgs::msg::Empty, Node, Publisher, QosProfile};

pub struct Bridge {
    pub tick: Publisher<Empty>,
}

impl Bridge {
    pub fn new(node: &mut Node) -> Result<Self> {
        let qos = QosProfile::default();

        Ok(Self {
            tick: node.create_publisher("tick", qos)?,
        })
    }
}
