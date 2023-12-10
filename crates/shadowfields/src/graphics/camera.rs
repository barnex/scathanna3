use super::internal::*;

// from https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/.
#[derive(Clone)]
pub(crate) struct Camera {
	pub position: vec3,
	pub orientation: Orientation,
	pub fovx_deg: f32,
	pub znear: f32,
	pub zfar: f32,
}

impl Camera {
	// Ray starting from the camera, going through the crosshair.
	pub fn crosshair_ray(&self) -> Ray<f64> {
		Ray64::new(self.position.into(), self.orientation.look_dir().into())
	}

	/// Does `pos` lie inside the camera frustum?
	/// TODO: this is a crude approximation that yields many false positives,
	/// use precise frustum instead.
	pub fn can_see(&self, pos: vec3) -> bool {
		(pos - self.position).dot(self.orientation.look_dir()) >= 0.0
	}

	// TODO: fov does not seem 100% correct. Test with test scene. Remove cgmath dependency.
	pub fn matrix(&self, viewport_size: uvec2) -> [[f32; 4]; 4] {
		let size = viewport_size.to_f32();
		let aspect = size.x() / size.y();
		let eye: [f32; 3] = self.position.into();
		let dir: [f32; 3] = self.orientation.look_dir().into();
		let up: [f32; 3] = vec3::EY.into();

		let view = cgmath::Matrix4::look_to_rh(eye.into(), dir.into(), up.into());

		// cgmath wants a vertical FOV, but horizontal is nicer to specify, so convert.
		let fovx_rad = self.fovx_deg * DEG;
		let fovy_rad = 2.0 * f32::asin(f32::sin(fovx_rad / 2.0) / aspect);
		let mut fovy_deg = fovy_rad / DEG;
		if !(fovy_deg >= 10.0) {
			fovy_deg = 10.0;
		}
		if !(fovy_deg < 160.0) {
			fovy_deg = 160.0;
		}
		let proj = cgmath::perspective(cgmath::Deg(fovy_deg), aspect, self.znear, self.zfar);

		let proj_view = proj * view;

		let matrix = OPENGL_TO_WGPU_MATRIX * proj_view;
		matrix.into()
	}
}

impl Default for Camera {
	fn default() -> Self {
		Self {
			position: (0.0, 0.0, -10.0).into(),
			orientation: default(),
			fovx_deg: 100.0,
			znear: 0.1,
			zfar: 1023.9,
		}
	}
}

const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
	1.0, 0.0, 0.0, 0.0, //
	0.0, 1.0, 0.0, 0.0, //
	0.0, 0.0, 0.5, 0.0, //
	0.0, 0.0, 0.5, 1.0, //
);
