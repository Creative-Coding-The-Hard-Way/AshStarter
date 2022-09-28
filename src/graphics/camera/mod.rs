use crate::math::Mat4;

/// Compute a raw projection matrix which transforms from view-space to
/// Vulkan clip-space coordinates.
///
/// # View Space
///
/// View Space is a vector space with bounds:
///
///   - X in [left, right]
///   - Y in [bottom, top]
///   - Z in [near, far]
///
/// # Vulkan Clip Space
///
/// Reference: https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#vertexpostproc-clipping
///
/// Thus, the mappings are:-
///
///   - [left, right] -> [-1.0, 1.0]
///   - [bottom, top] -> [1.0, -1.0]
///   - [near, far] -> [0.0, 1.0]
pub fn ortho_projection(
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
