use super::internal::*;

/// Data passed to each instance.
/// Used by shaders like `entity.wgsl`, `animation.wgsl``,
/// by appending `globals.wgsl`
///
/// ! `repr(C)` required by WGPU.
/// ! Must be kept in sync with `globals.wgsl`
///
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct InstanceData {
	pub bounding_box_size: vec3,
	pub _padding2: f32,
	pub bounding_box_bottom: vec3,
	pub _padding3: f32,
	pub lightbox: LightBox,
	pub model_matrix: [[f32; 4]; 4],
	pub time: f32,
	pub unused: f32,
	pub _padding: [f32; 2],
}
