mod graphics;
mod integrator;

pub use self::{graphics::Graphics, integrator::Integrator};

/// The datastructure used to represent a particle on the CPU and GPU.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Particle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct SimulationConfig {
    /// The width and height of the simulation area.
    dimensions: [f32; 2],
    particle_count: u32,
}

impl SimulationConfig {
    pub fn new(
        world_height: f32,
        aspect_ratio: f32,
        particle_count: u32,
    ) -> Self {
        Self {
            dimensions: [world_height * aspect_ratio, world_height],
            particle_count,
        }
    }

    /// Resize the simulation area's so that the width/height ratio
    /// matches the given aspect ratio. The simulation height is never changed
    /// when resizing.
    pub fn resize(&mut self, aspect_ratio: f32) {
        self.dimensions[0] = self.dimensions[1] * aspect_ratio;
    }

    pub fn particle_count(&self) -> u32 {
        self.particle_count
    }
}
