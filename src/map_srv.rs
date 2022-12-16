use anyhow::{bail, Context, Result};
use carla::client::World;
use futures::{Future, Stream, StreamExt};
use r2r::{
    autoware_auto_mapping_msgs::{msg::HADMapBin, srv::HADMapService},
    log_error,
    std_msgs::msg::Header,
    Clock, ClockType, Node, ServiceRequest,
};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::Duration,
};
use tempfile::TempDir;
use tokio::time::sleep;

pub fn new(node: &mut Node, world: World) -> Result<impl Future<Output = Result<()>>> {
    let stream = node.create_service::<HADMapService::Service>("map")?;
    let srv = run_service(stream.boxed(), world);
    Ok(srv)
}

struct MapConverter {
    _tmpdir: TempDir,
    input_path: PathBuf,
}

impl MapConverter {
    pub fn new() -> Result<Self> {
        let tmpdir = TempDir::new()?;
        let input_path = tmpdir.path().join("input.xodr");

        Ok(Self {
            _tmpdir: tmpdir,
            input_path,
        })
    }

    pub fn opendrive_to_lanelet2(&self, opendrive: &str) -> Result<Vec<u8>> {
        let err = |stderr| {
            format!(
                "`opendrive2lanelet-convert -f {}` failed.\
                 Please make sure opendrive2lanelet is installed. Try this:\
                 pip install https://github.com/usdot-fhwa-stol/opendrive2lanelet/archive/develop.zip\
                 ---- stderr ----\
                 {}",
                &self.input_path.display(),
                stderr
            )
        };

        fs::write(&self.input_path, opendrive)?;
        let Output {
            status,
            stdout,
            stderr,
        } = Command::new("opendrive2lanelet-convert")
            .arg("-f")
            .arg(&self.input_path)
            .output()
            .with_context(|| err(""))?;

        if !status.success() {
            bail!("{}", err(&String::from_utf8_lossy(&stderr)));
        }

        let _ = fs::remove_file(&self.input_path);

        Ok(stdout)
    }
}

struct Cache {
    world: World,
    world_id: u64,
    lanelet2: Vec<u8>,
    converter: MapConverter,
}

impl Cache {
    pub fn new(world: World) -> Result<Self> {
        let world_id = world.id();
        let converter = MapConverter::new()?;
        let opendrive = world.map().to_open_drive();
        let lanelet2 = converter.opendrive_to_lanelet2(&opendrive)?;

        Ok(Self {
            world,
            world_id,
            lanelet2,
            converter,
        })
    }

    pub fn get(&mut self) -> Result<&[u8]> {
        let curr_id = self.world.id();

        if curr_id != self.world_id {
            self.world_id = curr_id;
            let opendrive = self.world.map().to_open_drive();
            self.lanelet2 = self.converter.opendrive_to_lanelet2(&opendrive)?;
        }

        Ok(&self.lanelet2)
    }
}

async fn run_service(
    mut stream: impl Stream<Item = ServiceRequest<HADMapService::Service>> + Unpin,
    world: World,
) -> Result<()> {
    let mut clock = Clock::create(ClockType::RosTime)?;
    let mut cache = loop {
        let result = Cache::new(world.clone()).with_context(|| "Unable to publish world map.");

        match result {
            Ok(cache) => break cache,
            Err(err) => {
                log_error!(env!("CARGO_BIN_NAME"), "{:?}", err);
                sleep(Duration::from_secs(1)).await;
            }
        }
    };

    while let Some(req) = stream.next().await {
        let time = Clock::to_builtin_time(&clock.get_now()?);
        let data = match cache.get().with_context(|| "Unable to publish world map") {
            Ok(data) => data,
            Err(err) => {
                log_error!(env!("CARGO_BIN_NAME"), "{:?}", err);
                continue;
            }
        };

        let resp = HADMapService::Response {
            map: HADMapBin {
                header: Header {
                    stamp: time,
                    frame_id: "map".to_string(),
                },
                map_format: 0,
                format_version: "".to_string(),
                map_version: "".to_string(),
                data: data.to_vec(),
            },
            answer: 0,
        };
        req.respond(resp)?;
    }

    Ok(())
}
