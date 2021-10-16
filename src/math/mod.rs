use nalgebra;

pub type Mat4 = nalgebra::Matrix4<f32>;

pub mod projections {
    use super::Mat4;

    /// Build an orthographic projection matrix which transforms the given
    /// coordinate bounds to the Vulkan view volume.
    /// e.g. Input Values will be bounded by:
    ///  - x in [left, right]
    ///  - y in [bottom, top]
    ///  - z in [near, far]
    ///
    /// Output coordinates will be transformed to:
    ///  - left -> -1.0, right -> 1.0
    ///  - bottom -> 1.0, top -> -1.0
    ///    (because Vulkan's Y coordinate is negative at the top of the screen)
    ///  - near -> 0, far -> 1.0
    ///
    pub fn ortho(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        let mh = 2.0 / (right - left);
        let bh = (right + left) / (left - right);
        let mv = 2.0 / (bottom - top);
        let bv = (top + bottom) / (top - bottom);
        let mz = 1.0 / (far - near);
        let bz = near / (near - far);
        Mat4::new(
            mh, 0.0, 0.0, bh, //
            0.0, mv, 0.0, bv, //
            0.0, 0.0, mz, bz, //
            0.0, 0.0, 0.0, 1.0,
        )
    }
}
