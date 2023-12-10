use crate::prelude::*;

#[derive(Default)]
pub(crate) struct KeyMap(HashMap<char, Button>);

impl KeyMap {
	pub fn map(&self, logical_key: &winit::keyboard::Key) -> Option<Button> {
		use winit::keyboard::NamedKey;
		match logical_key {
			winit::keyboard::Key::Character(c) => self.0.get(&c.chars().next()?).copied(),
			winit::keyboard::Key::Named(NamedKey::ArrowUp) => Some(Button::Forward),
			winit::keyboard::Key::Named(NamedKey::ArrowDown) => Some(Button::Backward),
			winit::keyboard::Key::Named(NamedKey::ArrowLeft) => Some(Button::Left),
			winit::keyboard::Key::Named(NamedKey::ArrowRight) => Some(Button::Right),
			winit::keyboard::Key::Named(NamedKey::Enter) => Some(Button::Enter),
			winit::keyboard::Key::Named(NamedKey::Tab) => Some(Button::Console),
			winit::keyboard::Key::Named(NamedKey::Space) => Some(Button::Jump), // 🪲 ignores config because winit does not send space as a character
			winit::keyboard::Key::Named(_) => None,
			winit::keyboard::Key::Unidentified(_) => None,
			winit::keyboard::Key::Dead(_) => None,
		}
	}

	pub fn parse(controls: &Controls) -> Result<Self> {
		let mut map = HashMap::default();
		map.insert(controls.forward.to_ascii_lowercase(), Button::Forward);
		map.insert(controls.forward.to_ascii_uppercase(), Button::Forward);
		map.insert(controls.left.to_ascii_lowercase(), Button::Left);
		map.insert(controls.left.to_ascii_uppercase(), Button::Left);
		map.insert(controls.backward.to_ascii_lowercase(), Button::Backward);
		map.insert(controls.backward.to_ascii_uppercase(), Button::Backward);
		map.insert(controls.right.to_ascii_lowercase(), Button::Right);
		map.insert(controls.right.to_ascii_uppercase(), Button::Right);
		map.insert(controls.crouch.to_ascii_lowercase(), Button::Crouch);
		map.insert(controls.crouch.to_ascii_uppercase(), Button::Crouch);
		Ok(Self(map))
	}
}
