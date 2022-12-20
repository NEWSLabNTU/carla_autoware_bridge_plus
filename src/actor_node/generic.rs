use std::future::IntoFuture;

use crate::time::TimeDelta;

use super::{
    other::{OtherPub, OtherSub},
    sensor::{SensorPub, SensorSub},
    traffic_light::{TrafficLightPub, TrafficLightSub},
    traffic_sign::{TrafficSignPub, TrafficSignSub},
    vehicle::{VehiclePub, VehicleSub},
};
use anyhow::Result;
use carla::client::{Actor, ActorKind};
use futures::{future::BoxFuture, FutureExt};
use r2r::{builtin_interfaces::msg::Time, Node};

pub fn new(node: &mut Node, actor: Actor) -> Result<(ActorPub, ActorSub)> {
    use ActorKind as K;
    let (pub_, sub) = match actor.into_kinds() {
        K::Vehicle(actor) => {
            let (pub_, sub) = super::vehicle::new(node, actor)?;
            (pub_.into(), sub.into())
        }
        K::Sensor(actor) => {
            let (pub_, sub) = super::sensor::new(node, actor)?;
            (pub_.into(), sub.into())
        }
        K::TrafficLight(actor) => {
            let (pub_, sub) = super::traffic_light::new(node, actor)?;
            (pub_.into(), sub.into())
        }
        K::TrafficSign(actor) => {
            let (pub_, sub) = super::traffic_sign::new(node, actor)?;
            (pub_.into(), sub.into())
        }
        K::Other(actor) => {
            let (pub_, sub) = super::other::new(node, actor)?;
            (pub_.into(), sub.into())
        }
    };

    Ok((pub_, sub))
}

pub enum ActorPub {
    Vehicle(VehiclePub),
    Sensor(SensorPub),
    TrafficLight(TrafficLightPub),
    TrafficSign(TrafficSignPub),
    Other(OtherPub),
}

impl From<TrafficLightPub> for ActorPub {
    fn from(v: TrafficLightPub) -> Self {
        Self::TrafficLight(v)
    }
}

impl From<OtherPub> for ActorPub {
    fn from(v: OtherPub) -> Self {
        Self::Other(v)
    }
}

impl From<TrafficSignPub> for ActorPub {
    fn from(v: TrafficSignPub) -> Self {
        Self::TrafficSign(v)
    }
}

impl From<SensorPub> for ActorPub {
    fn from(v: SensorPub) -> Self {
        Self::Sensor(v)
    }
}

impl From<VehiclePub> for ActorPub {
    fn from(v: VehiclePub) -> Self {
        Self::Vehicle(v)
    }
}

impl ActorPub {
    pub fn poll(&mut self, time: &Time, time_delta: TimeDelta) -> Result<()> {
        match self {
            ActorPub::Vehicle(pub_) => pub_.poll(time, time_delta)?,
            ActorPub::Sensor(pub_) => pub_.poll(time)?,
            ActorPub::TrafficSign(pub_) => pub_.poll(time)?,
            ActorPub::Other(pub_) => pub_.poll(time)?,
            ActorPub::TrafficLight(pub_) => pub_.poll(time)?,
        }

        Ok(())
    }
}

pub enum ActorSub {
    Vehicle(VehicleSub),
    Sensor(SensorSub),
    TrafficTraffic(TrafficLightSub),
    TrafficSign(TrafficSignSub),
    Other(OtherSub),
}

impl IntoFuture for ActorSub {
    type Output = ();
    type IntoFuture = BoxFuture<'static, ()>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            match self {
                ActorSub::Vehicle(sub) => sub.await,
                ActorSub::Sensor(_) => (),
                ActorSub::TrafficSign(_) => (),
                ActorSub::Other(_) => (),
                ActorSub::TrafficTraffic(_) => (),
            }
        }
        .boxed()
    }
}

impl From<TrafficLightSub> for ActorSub {
    fn from(v: TrafficLightSub) -> Self {
        Self::TrafficTraffic(v)
    }
}

impl From<OtherSub> for ActorSub {
    fn from(v: OtherSub) -> Self {
        Self::Other(v)
    }
}

impl From<TrafficSignSub> for ActorSub {
    fn from(v: TrafficSignSub) -> Self {
        Self::TrafficSign(v)
    }
}

impl From<SensorSub> for ActorSub {
    fn from(v: SensorSub) -> Self {
        Self::Sensor(v)
    }
}

impl From<VehicleSub> for ActorSub {
    fn from(v: VehicleSub) -> Self {
        Self::Vehicle(v)
    }
}
