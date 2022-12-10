use anyhow::{anyhow, Context, Result};
use r2r::{Node, ParameterValue};

const PARAM_CARLA_HOST: &str = "carla_host";
const PARAM_CARLA_PORT: &str = "carla_port";

pub struct Params {
    pub carla_host: String,
    pub carla_port: u16,
}

impl Params {
    pub fn load(node: &Node) -> Result<Self> {
        let params = node.params.lock().unwrap();
        let host = params
            .get(PARAM_CARLA_HOST)
            .ok_or_else(|| anyhow!("{PARAM_CARLA_HOST} is not set"))?
            .to_str()
            .ok_or_else(|| anyhow!("{PARAM_CARLA_HOST} has invalid type"))?;
        let port: u16 = params
            .get(PARAM_CARLA_PORT)
            .ok_or_else(|| anyhow!("{PARAM_CARLA_PORT} is not set"))?
            .to_i64()
            .ok_or_else(|| anyhow!("{PARAM_CARLA_PORT} has invalid type"))?
            .try_into()
            .with_context(|| "invalid port number")?;

        Ok(Self {
            carla_host: host.to_string(),
            carla_port: port,
        })
    }
}

pub trait ParameterValueExt {
    fn to_str(&self) -> Option<&str>;
    fn to_i64(&self) -> Option<i64>;
}

impl ParameterValueExt for ParameterValue {
    fn to_str(&self) -> Option<&str> {
        if let Self::String(val) = self {
            Some(val)
        } else {
            None
        }
    }

    fn to_i64(&self) -> Option<i64> {
        if let Self::Integer(val) = *self {
            Some(val)
        } else {
            None
        }
    }
}
