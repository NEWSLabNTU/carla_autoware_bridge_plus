use std::time::Duration;

use carla::client::Timestamp;

#[derive(Debug)]
pub struct TimeBuffer {
    prev: Option<Time>,
}

#[derive(Debug)]
struct Time {
    frame: usize,
    time: Duration,
}

impl TimeBuffer {
    pub fn step(&mut self, timestamp: &Timestamp) -> TimeDelta {
        let curr_time = Duration::from_secs_f64(timestamp.elapsed_seconds);

        let delta = match &self.prev {
            Some(prev) => TimeDelta {
                frame: timestamp.frame,
                frame_delta: timestamp.frame - prev.frame,
                time: curr_time,
                time_delta: curr_time - prev.time,
            },
            None => TimeDelta {
                frame: timestamp.frame,
                frame_delta: 0,
                time: curr_time,
                time_delta: Duration::ZERO,
            },
        };
        self.prev = Some(Time {
            frame: timestamp.frame,
            time: curr_time,
        });

        delta
    }
}

impl Default for TimeBuffer {
    fn default() -> Self {
        Self { prev: None }
    }
}

#[derive(Debug, Clone)]
pub struct TimeDelta {
    pub frame: usize,
    pub frame_delta: usize,
    pub time: Duration,
    pub time_delta: Duration,
}
