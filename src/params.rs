use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use r2r::{log_warn, Node, ParameterValue};

const PARAM_CARLA_HOST: &str = "carla_host";
const PARAM_CARLA_PORT: &str = "carla_port";
const PARAM_CARLA_TIMEOUT_MILLIS: &str = "carla_timeout_millis";
const DEFAULT_CARLA_HOST: &str = "127.0.0.1";
const DEFAULT_CARLA_PORT: u16 = 2000;
const DEFAULT_CARLA_TIMEOUT_MILLIS: u64 = 20000;

type ParamsMap = HashMap<String, ParameterValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub carla_host: String,
    pub carla_port: u16,
    pub carla_timeout_millis: u64,
}

impl Params {
    pub fn load(node: &Node) -> Result<Self> {
        let params = node.params.lock().unwrap();
        let carla_host = get_carla_host(&params)?;
        let carla_port = get_carla_port(&params)?;
        let carla_timeout_millis = get_carla_timeout_millis(&params)?;
        Ok(Self {
            carla_host,
            carla_port,
            carla_timeout_millis,
        })
    }
}

fn get_carla_host(params: &ParamsMap) -> Result<String> {
    let Some(value) = params.get(PARAM_CARLA_HOST) else {
        log_warn!(env!("CARGO_BIN_NAME"), "Using default value '{}' for parameter '{}'", DEFAULT_CARLA_HOST, PARAM_CARLA_HOST);
        return Ok(DEFAULT_CARLA_HOST.to_string());
    };
    let value = value
        .to_str()
        .ok_or_else(|| anyhow!("{PARAM_CARLA_HOST} has invalid type"))?;
    Ok(value.to_string())
}

fn get_carla_port(params: &ParamsMap) -> Result<u16> {
    let Some(value) = params.get(PARAM_CARLA_PORT) else {
        log_warn!(env!("CARGO_BIN_NAME"), "Using default value '{}' for parameter '{}'", DEFAULT_CARLA_PORT, PARAM_CARLA_PORT);
        return Ok(DEFAULT_CARLA_PORT);
    };
    let value = value
        .to_i64()
        .ok_or_else(|| anyhow!("{PARAM_CARLA_PORT} has invalid type"))?;
    let value = value
        .try_into()
        .with_context(|| format!("invalid {PARAM_CARLA_PORT} number {}", value))?;
    Ok(value)
}

fn get_carla_timeout_millis(params: &ParamsMap) -> Result<u64> {
    let Some(value) = params.get(PARAM_CARLA_TIMEOUT_MILLIS) else {
        log_warn!(env!("CARGO_BIN_NAME"), "Using default value '{}' for parameter '{}'", DEFAULT_CARLA_TIMEOUT_MILLIS, PARAM_CARLA_TIMEOUT_MILLIS);
        return Ok(DEFAULT_CARLA_TIMEOUT_MILLIS);
    };
    let value = value
        .to_i64()
        .ok_or_else(|| anyhow!("{PARAM_CARLA_TIMEOUT_MILLIS} has invalid type"))?;
    let value = value
        .try_into()
        .with_context(|| format!("invalid {PARAM_CARLA_TIMEOUT_MILLIS} number {}", value))?;
    Ok(value)
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
