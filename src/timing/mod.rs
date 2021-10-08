mod frame_rate_limit;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// A Vulkan application will generally run as fast as it possibly can to
/// get images on screen. Often this is desirable, but when workloads are low
/// it can cause unreasonably high frame-rates and therefore unexpectedly high
/// CPU/GPU utilization. To prevent this, a frame rate limit can be imposed
/// which just sleeps or yields for a bit of time each frame.
pub struct FrameRateLimit {
    frames_to_track: usize,
    frame_starts: VecDeque<Instant>,
    target_duration: Duration,
}
