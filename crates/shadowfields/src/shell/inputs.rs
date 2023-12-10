use crate::prelude::*;
use winit::{
	dpi::PhysicalPosition,
	event::{KeyEvent, MouseScrollDelta},
};

/// Accumulates input events since the last tick,
/// allowing for queries like "is this key currently held down?".
///
/// Also de-bounces events faster than a tick,
/// and removes OS key repeats.
#[derive(Debug, Clone, Default)]
pub(crate) struct Inputs {
	pub buttons_down: Set<Button>,
	pub buttons_pressed: Set<Button>,
	pub buttons_released: Set<Button>,
	pub received_characters: String,
	pub mouse_delta: ivec2,
	pub tick_time: Duration, // ??????????????????????????
}

/// A keystroke or mouse click or scroll action
/// are all uniformly treated as "button" pushes.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Button {
	Backward,
	Console,
	Crouch,
	Enter,
	Esc,
	Forward,
	Jump,
	Left,
	Right,
	Mouse1,
	Mouse2,
	MouseWheelUp,
	MouseWheelDown,
}

impl Inputs {
	/// Time since the last tick, in seconds.
	/// This tells us how long buttons were pressed.
	//pub fn dt(&self) -> f32 {
	//	self.tick_time.as_secs_f32()
	//}

	/// Forget all pending inputs.
	/// (E.g. needed after focus loss on Wayland:
	/// ESC DOWN gets recorded but ESC UP not (X11 sends both))
	pub fn clear(&mut self) {
		self.buttons_down.clear();
		self.buttons_pressed.clear();
		self.buttons_released.clear();
		self.received_characters.clear();
		self.mouse_delta = default();
	}

	/// Record that an event happened since the last `forget` call
	/// (i.e. since the last tick).
	/// (Called by the event loop, not to be called by the event consumer (i.e. App)).
	pub fn record_window_event(&mut self, keymap: &KeyMap, event: &WindowEvent) {
		use WindowEvent::*;
		match event {
			KeyboardInput {
				event: KeyEvent {
					logical_key,
					text,
					state,
					repeat: false,
					..
				},
				..
			} => {
				if let Some(text) = text {
					self.received_characters.push_str(text);
				}
				if let Some(button) = keymap.map(logical_key) {
					self.record_button(button, *state)
				}
			}
			MouseInput { button, state, .. } => {
				match button {
					winit::event::MouseButton::Left => self.record_button(Button::Mouse1, *state),
					winit::event::MouseButton::Right => self.record_button(Button::Mouse2, *state),
					_ => (),
				};
			}
			MouseWheel { delta, .. } => self.record_mouse_wheel(delta),
			_ => (),
		}
	}

	/// Record mouse motion.
	/// All mouse motion between ticks is added up,
	/// and presented as a single motion.
	/// (Called by the event loop, not to be called by the event consumer (i.e. App)).
	pub fn record_mouse_motion(&mut self, delta: dvec2) {
		let delta = delta.convert::<i32>();
		self.mouse_delta += delta;
	}

	/// Record a key or mouse button event (handled uniformly).
	pub fn record_button(&mut self, but: Button, state: ElementState) {
		use ElementState::*;
		match state {
			Pressed => self.press_button(but),
			Released => self.release_button(but),
		}
	}

	//pub fn tap_button(&mut self, but: Button) {
	//	self.press_button(but);
	//	self.release_button(but);
	//}

	pub fn press_button(&mut self, but: Button) {
		if !self.buttons_down.contains(&but) {
			self.buttons_down.insert(but);
			// only record as pressed if not down yet
			// to remove key repeats.
			self.buttons_pressed.insert(but);
		}
	}

	pub fn release_button(&mut self, but: Button) {
		self.buttons_down.remove(&but);
		// do not removed from pressed (yet)
		// a button can be pressed and released within the same tick.
		self.buttons_released.insert(but);
	}

	fn record_mouse_wheel(&mut self, delta: &MouseScrollDelta) {
		/*
			Mouse wheel delta's can vary wildly,
			reduce them just a single Up / Down event
			discarding the scroll amount.
		*/
		let dy = match delta {
			MouseScrollDelta::LineDelta(_, y) => *y,
			MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => *y as f32,
		};
		let button = match dy {
			_ if dy > 0.0 => Some(Button::MouseWheelUp),
			_ if dy < 0.0 => Some(Button::MouseWheelDown),
			_ => None,
		};
		/*
			Record both a press and release
			to make the scroll event appear as a button press
			(the scroll wheel cannot be "held down" continuously like a mouse button).
		*/
		if let Some(button) = button {
			self.buttons_pressed.insert(button);
			self.buttons_released.insert(button);
		}
	}

	/// Is a button currently held down?
	/// (This repeats on every tick for as long as the button is held)
	pub fn is_down(&self, but: Button) -> bool {
		self.buttons_down.contains(&but)
	}

	pub fn was_pressed(&self, but: Button) -> bool {
		self.is_down(but) || self.just_pressed(but)
	}

	/// Was a button pressed right before the current tick?
	/// This triggers only once per physical keypress.
	/// OS keyboard repeats are ignored.
	pub fn just_pressed(&self, but: Button) -> bool {
		self.buttons_pressed.contains(&but)
	}

	/// Was a button released right before the current tick?
	pub fn just_released(&self, but: Button) -> bool {
		self.buttons_released.contains(&but)
	}

	/// Iterate over all keys currently held down.
	pub fn buttons_down(&self) -> impl Iterator<Item = Button> + '_ {
		self.buttons_down.iter().copied()
	}

	/// Iterate over all keys pressed down right before this tick.
	pub fn buttons_pressed(&self) -> impl Iterator<Item = Button> + '_ {
		self.buttons_pressed.iter().copied()
	}

	/// Iterate over all keys released right before this tick.
	pub fn buttons_released(&self) -> impl Iterator<Item = Button> + '_ {
		self.buttons_released.iter().copied()
	}

	/// The button that was pressed during the last tick, assuming there was only one.
	/// (More than one pressed causes the superfluous ones to be dropped arbitrarily).
	/// Used for the editor where pressing two buttons at the same time is rare and useless.
	pub fn pressed_button(&self) -> Option<Button> {
		self.buttons_pressed.iter().next().copied()
	}

	/// The relative mouse movement since the last tick.
	pub fn mouse_delta(&self) -> vec2 {
		self.mouse_delta.convert()
	}

	/// The relative mouse wheel movement since last tick.
	pub fn mouse_wheel_delta(&self) -> i32 {
		let mut delta = 0;
		if self.just_pressed(Button::MouseWheelDown) {
			delta -= 1;
		}
		if self.just_pressed(Button::MouseWheelUp) {
			delta += 1;
		}
		delta
	}

	/// The unicode characters typed since the last tick.
	pub fn received_characters(&self) -> &str {
		&self.received_characters
	}

	/// Forget all changes since previous `forget` call.
	/// Called by the event loop after the inputs have been used. I.e. after each tick.
	pub fn tick(&mut self) {
		self.buttons_pressed.clear();
		self.buttons_released.clear();
		self.mouse_delta = ivec2(0, 0);
		self.received_characters.clear();
		self.tick_time = Duration::ZERO;
	}
}
