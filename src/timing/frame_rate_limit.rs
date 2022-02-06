use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use super::FrameRateLimit;

impl FrameRateLimit {
    /// Create a new frame rate limit for a given target fps
    pub fn new(target_fps: u32, frames_to_track: usize) -> Self {
        Self {
            frames_to_track,
            frame_starts: VecDeque::with_capacity(frames_to_track),
            target_duration: Duration::from_secs(1) / target_fps,
        }
    }

    /// Call at the beginning of each frame to establish the start-point when
    /// computing elapsed time.
    pub fn start_frame(&mut self) {
        if self.frame_starts.len() > self.frames_to_track {
            self.frame_starts.pop_back();
        }
        self.frame_starts.push_front(Instant::now());
    }

    /// Sleep for any remaining time in the target fps.
    pub fn sleep_to_limit(&self) {
        let elapsed = Instant::now() - *self.frame_starts.front().unwrap();
        if elapsed < self.target_duration {
            spin_sleep::sleep(self.target_duration - elapsed);
        }
    }

    /// Return the average amount of time spent on the last n frames.
    /// N is the value given for `frames_to_track` when creating the frame
    /// rate limit.
    pub fn avg_frame_time(&self) -> Duration {
        let oldest_frame = self.frame_starts.back().unwrap();
        let total_duration = Instant::now() - *oldest_frame;
        return total_duration / self.frame_starts.len() as u32;
    }
}
