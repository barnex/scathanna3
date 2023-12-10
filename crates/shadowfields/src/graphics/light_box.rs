use super::internal::*;

/// Entities (things drawn with `entity.wgsl`, `animation.wgsl`) are
/// illuminated via light samples on the vertices of a cuboid: the `LightBox`.
///
/// These light samples are obtained via ray tracing.
///
///        *------*  <- light sample
///       /      /|
///      *------* | <- light sample
///      |  En  | |
///      |  ti  | * <- light sample
///      |  ty  |/
///      *------* <- light sample
///
/// The vertex shader interpolates between those 8 light samples.
/// The LightBox data is passed as part of `InstanceData``.
///
/// ! `repr(C)` required by WGPU.
/// ! Must be kept in sync with `globals.wgsl`
///
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct LightBox {
	pub vertex_light: [[[vec4; 2]; 2]; 2], // [z][y][x]
}

impl LightBox {
	/// Full white light on every vertex.
	pub const WHITE: Self = Self {
		vertex_light: [[[vec4::ONES; 2]; 2]; 2],
	};

	pub fn test() -> Self {
		Self::test_x()
	}

	/// Test pattern along x
	pub fn test_x() -> Self {
		let w = WHITE.append(1.0);
		let r = RED.append(1.0);

		Self {
			vertex_light: [
				[
					[w, r], // z0 y0
					[w, r], // z0 y1
				], //.
				//.
				[
					[w, r], //z1 y0
					[w, r], //z1 y1
				],
			],
		}
	}

	/// Test pattern along y
	pub fn test_y() -> Self {
		let w = WHITE.append(1.0);
		let g = GREEN.append(1.0);

		Self {
			vertex_light: [
				[
					[w, w], // z0 y0
					[g, g], // z0 y1
				], //.
				//.
				[
					[w, w], //z1 y0
					[g, g], //z1 y1
				],
			],
		}
	}

	/// Test pattern along y
	pub fn test_z() -> Self {
		let w = WHITE.append(1.0);
		let b = BLUE.append(1.0);

		Self {
			vertex_light: [
				[
					[w, w], // z0 y0
					[w, w], // z0 y1
				], //.
				//.
				[
					[b, b], //z1 y0
					[b, b], //z1 y1
				],
			],
		}
	}
}
