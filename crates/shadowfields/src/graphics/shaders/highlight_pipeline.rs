use super::internal::*;

// TODO: dead, remove (editor functinality)
pub(crate) struct HighlightPipeline {
	pub pipeline: wgpu::RenderPipeline,
	texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl HighlightPipeline {
	pub fn new(opts: &GraphicsOpts, device: &wgpu::Device, surface_format: wgpu::TextureFormat, global_uniforms_layout: &wgpu::BindGroupLayout, lines: bool) -> Self {
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some(file!()),
			source: wgpu::ShaderSource::Wgsl(include_str!("highlight.wgsl").into()),
		});

		let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				// Binding 0: texture data
				wgpu::BindGroupLayoutEntry {
					binding: 0, // Fragment shader: t_diffuse;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				// Binding 1: texture sampler
				wgpu::BindGroupLayoutEntry {
					binding: 1, // Fragment shader: s_diffuse;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
			label: Some(file!()),
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some(file!()),
			layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some(file!()),
				bind_group_layouts: &[&texture_bind_group_layout, global_uniforms_layout],
				push_constant_ranges: &[],
			})),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[VertexLM::desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_format,
					blend: Some(wgpu::BlendState {
						color: wgpu::BlendComponent::OVER, // TODO ////////////////////
						alpha: wgpu::BlendComponent::OVER,
					}),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: match lines {
					true => wgpu::PrimitiveTopology::LineList,
					false => wgpu::PrimitiveTopology::TriangleList,
				},
				strip_index_format: None,
				front_face: wgpu::FrontFace::Cw,
				cull_mode: Some(wgpu::Face::Front),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: Canvas::DEPTH_FORMAT,
				depth_write_enabled: false,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState {
					// https://gpuweb.github.io/gpuweb/#abstract-opdef-biased-fragment-depth
					constant: -10,
					slope_scale: 0.0,
					clamp: -100.0,
				},
			}),
			multisample: wgpu::MultisampleState {
				count: opts.msaa_sample_count(),
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		Self { pipeline, texture_bind_group_layout }
	}

	fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.texture_bind_group_layout
	}

	pub fn texture_bind_group(&self, device: &Device, texture: &Texture) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: self.texture_bind_group_layout(),
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0, //
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
			label: Some(file!()),
		})
	}
}
