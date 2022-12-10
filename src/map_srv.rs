use anyhow::{ensure, Result};
use carla::client::World;
use futures::{Future, Stream, StreamExt};
use r2r::{
    autoware_auto_mapping_msgs::{msg::HADMapBin, srv::HADMapService},
    std_msgs::msg::Header,
    Clock, ClockType, Node, ServiceRequest,
};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};
use tempfile::TempDir;

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
        fs::write(&self.input_path, opendrive)?;
        let Output {
            status,
            stdout,
            stderr,
        } = Command::new("opendrive2lanelet-convert")
            .arg("-f")
            .arg(&self.input_path)
            .output()?;

        let command_line = || {
            format!(
                "opendrive2lanelet-convert -f {}",
                &self.input_path.display()
            )
        };

        ensure!(
            status.success(),
            "Unable to run '{}':\n{}",
            command_line(),
            String::from_utf8_lossy(&stderr)
        );

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
    let mut cache = Cache::new(world)?;

    while let Some(req) = stream.next().await {
        let time = Clock::to_builtin_time(&clock.get_now()?);
        let resp = HADMapService::Response {
            map: HADMapBin {
                header: Header {
                    stamp: time,
                    frame_id: "map".to_string(),
                },
                map_format: 0,
                format_version: "".to_string(),
                map_version: "".to_string(),
                data: cache.get()?.to_vec(),
            },
            answer: 0,
        };
        req.respond(resp)?;
    }

    Ok(())
}
