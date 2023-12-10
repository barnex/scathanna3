use crate::prelude::*;
use std::task::Poll;

use futures;
use futures::Future;
use winit::dpi::PhysicalSize;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::CursorGrabMode;
use winit::window::WindowBuilder;

pub(crate) struct Shell3 {
	window: Window,
	canvas: Canvas,
	keymap: KeyMap,
	cursor_grabbed: bool,
	previous_tick: Instant,
	/// The inputs accumulated since the last call to `await_tick`.
	inputs: Inputs,
	mailbox: WinitMailbox,
}

impl Shell3 {
	pub fn run(settings: Settings) -> Result<()> {
		LOG.write(" >> Welcome to Scathanna 3.0 << ");

		#[cfg(debug_assertions)]
		LOG.write(DEBUG_WARNING);

		let viewport_size = uvec2(settings.graphics.width, settings.graphics.height);
		let keymap = KeyMap::parse(&settings.controls)?;
		let event_loop = EventLoop::new()?;
		event_loop.set_control_flow(ControlFlow::Poll); // 👈 best for games (https://docs.rs/winit/0.29.3/winit/)
		let window = WindowBuilder::new() //
			.with_inner_size(PhysicalSize::<u32> {
				width: settings.graphics.width,
				height: settings.graphics.height,
			})
			.with_fullscreen(settings.graphics.fullscreen.then(|| winit::window::Fullscreen::Borderless(None)))
			.with_title("Scathanna 3.0")
			.build(&event_loop)?;

		let canvas = Canvas::new(&settings.graphics, &window)?;

		let mailbox = WinitMailbox::new(viewport_size);

		let shell = Self {
			window,
			canvas,
			cursor_grabbed: false,
			previous_tick: Instant::now(), // 👈 ??
			keymap,
			inputs: Inputs::default(),
			mailbox,
		};

		shell.event_loop(settings, event_loop)
	}

	fn event_loop(mut self, settings: Settings, event_loop: EventLoop<()>) -> Result<()> {
		let mut profiler = Profiler::new(settings.debug.profile);

		let mut gameloop = Box::pin(Client::gameloop(settings, WinitWindow::new(self.mailbox.clone())));

		let mut noop_cx = std::task::Context::from_waker(futures::task::noop_waker_ref());

		let my_window_id = self.window.id();
		event_loop
			.run(move |event, elwt| {
				match event {
					Event::WindowEvent { ref event, window_id } if window_id == my_window_id => {
						if self.cursor_grabbed {
							self.handle_window_event(event);
						}
						match event {
							WindowEvent::MouseInput { .. } => self.grab_cursor(),
							// Needed for MacOSX where ESC gets delivered as WindowEvent, not DeviceEvent
							WindowEvent::KeyboardInput {
								event: KeyEvent {
									physical_key: PhysicalKey::Code(KeyCode::Escape),
									state: ElementState::Pressed,
									..
								},
								..
							} => self.release_cursor(),
							WindowEvent::Focused(false) => self.release_cursor(),
							WindowEvent::CloseRequested | WindowEvent::Destroyed => elwt.exit(),
							WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => self.handle_resize(),
							WindowEvent::RedrawRequested => {
								self.update_dt(); // 👈
								profiler.start_new_frame(self.previous_tick /* just set by update_dt */, self.inputs.tick_time.as_secs_f32());

								//                      👇
								match gameloop.as_mut().poll(&mut noop_cx) {
									Poll::Ready(Ok(())) => elwt.exit(), // gameloop done
									Poll::Ready(Err(e)) => {
										eprintln!("gameloop exited: {e:#}"); // 👈 TODO: take back to main menu or so
										elwt.exit()
									}
									Poll::Pending => (),
								}
								profiler.gameloop_polled();

								match self.mailbox.get() {
									WinitMailboxInner::RequestRender(sg) => {
										self.canvas.render(&sg);
										profiler.rendered();
										// TODO: could gather more inputs here (anti-lag)?
										self.mailbox.set(WinitMailboxInner::RequestTick(TickRequest {
											inputs: self.inputs.clone(),
											viewport_size: self.canvas.viewport_size(),
										}));
									}
									WinitMailboxInner::RequestTick(_) => (),
								};

								self.inputs.tick(); // 👈
								self.window.request_redraw(); // 👈 continuously redraw
							}
							_ => (),
						};
					}
					Event::DeviceEvent { event, .. } => {
						match event {
							DeviceEvent::MouseMotion { delta } => {
								if self.cursor_grabbed {
									self.handle_mouse_motion(delta.into())
								}
							}
							// Always handle ESC regardless of focus, so we don't steal the cursor.
							DeviceEvent::Key(
								RawKeyEvent {
									physical_key: PhysicalKey::Code(KeyCode::Escape),
									state: ElementState::Pressed,
									..
								},
								..,
							) => self.release_cursor(),
							_ => (),
						}
					}
					_ => (),
				}
			})
			.context("event_loop")
	}

	// Attempt to grab the mouse cursor if not yet grabbed.
	fn grab_cursor(&mut self) {
		self.window.set_cursor_visible(false);
		if !self.cursor_grabbed {
			if let Err(e) = self.window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Locked)) {
				eprintln!("confine cursor: {}", e);
				return;
			}
		}
		self.cursor_grabbed = true;
	}

	// Release the mouse cursor if grabbed.
	fn release_cursor(&mut self) {
		println!("release cursor");
		match self.window.set_cursor_grab(CursorGrabMode::None) {
			Ok(()) => (),
			Err(e) => eprintln!("release cursor: {}", e),
		}
		self.window.set_cursor_visible(true);
		self.cursor_grabbed = false;
		// Needed after focus loss on Wayland:
		// ESC DOWN gets recorded but ESC UP not (X11 sends both).
		self.inputs.clear();
	}

	/// Update the current time step, in preparation of a new `tick` call.
	fn update_dt(&mut self) {
		const MIN_DT: Duration = Duration::from_millis(1);
		const MAX_DT: Duration = Duration::from_millis(100);
		let now = Instant::now();
		self.inputs.tick_time = (now - self.previous_tick).clamp(MIN_DT, MAX_DT);
		self.previous_tick = now;
	}

	fn handle_resize(&mut self) {
		let size = self.window.inner_size();
		let size = uvec2(size.width, size.height);
		if !size.iter().all(|v| v > 0 && v < 16384) {
			return log::error!("resize: invalid viewport size: {size}");
		}
		if size != self.canvas.viewport_size() {
			self.canvas.resize(size);
			self.window.request_redraw();
		}
	}

	fn handle_window_event(&mut self, event: &WindowEvent) {
		self.inputs.record_window_event(&self.keymap, event);
	}

	fn handle_mouse_motion(&mut self, delta: dvec2) {
		self.inputs.record_mouse_motion(delta);
	}
}
