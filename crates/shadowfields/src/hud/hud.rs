use super::internal::*;

pub(crate) struct HUD {
	slots: [Slot; 8],
	pub crosshair: bool,
	cache: SingleCache<Object>,
}

#[derive(Default)]
struct Slot {
	text: String,
	ttl_secs: f32,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum HUDPos {
	TopLeft = 0,
	TopRight = 1,
	BottomLeft = 2,
	BottomRight = 3,
	Center = 4,
	TopCenter = 5,
	TopCenter2 = 6,
	Debug = 7,
}

use HUDPos::*;

impl HUD {
	pub fn new() -> Self {
		Self {
			slots: default(),
			crosshair: true,
			cache: default(),
		}
	}

	pub fn clear(&mut self) {
		for s in &mut self.slots {
			s.text.clear();
			s.ttl_secs = 0.0;
		}
	}

	pub fn apply(&mut self, upd: HUDUpdate) -> Option<()> {
		*self.slots.get_mut(upd.pos as usize)? = Slot {
			text: upd.text,
			ttl_secs: upd.ttl_sec,
		};
		self.cache.clear();
		Some(())
	}

	pub fn show_info(&mut self, text: impl Into<String>) {
		self.set_text(HUDPos::TopLeft, text, 5.0)
	}

	pub fn set_text(&mut self, pos: HUDPos, text: impl Into<String>, ttl_secs: f32) {
		let text = text.into();
		self.slots[pos as usize] = Slot { text, ttl_secs };
		self.cache.clear();
	}

	pub fn tick(&mut self, dt: f32) {
		for slot in &mut self.slots {
			if slot.ttl_secs > 0.0 {
				slot.ttl_secs -= dt;
				if slot.ttl_secs < 0.0 {
					slot.text.clear();
					self.cache.clear();
				}
			}
		}
	}

	pub fn draw_on(&self, sg: &mut SceneGraph) {
		if self.crosshair {
			self.draw_crosshair(sg);
		}

		sg.push(self.cache.clone_or(|| self.render(sg.viewport_size)));
	}

	fn render(&self, viewport: uvec2) -> Object {
		let ctx = ctx();
		let mut buf = MeshBuffer::new();
		let text = |pos| &self.slots[pos as usize].text;

		buf.append(&layout_text(viewport, uvec2(0, 0), text(TopLeft)));
		buf.append(&layout_text(viewport, uvec2(32, 32), text(Debug)));
		buf.append(&layout_text_right(viewport, text(TopRight)));
		buf.append(&layout_text_bottom(viewport, text(BottomLeft)));

		{
			let text = text(Center);
			// some fixed-point arithmetic to get the text about 20% above the crosshairs
			let pos = (viewport * 1024) / uvec2(2 * 1024, 2 * 1024 + 512) - text_size_pix(text) / 2;
			buf.append(&layout_text(viewport, pos, text));
		}

		{
			let text = text(TopCenter);
			let pos = viewport / uvec2(2, 4) - text_size_pix(text) / 2;
			buf.append(&layout_text(viewport, pos, text));
		}

		{
			let text = text(TopCenter2);
			let pos = viewport / uvec2(2, 4) - text_size_pix(text) / 2 + uvec2(0, 2 * _EMBEDDED_CHAR_SIZE.y());
			buf.append(&layout_text(viewport, pos, text));
		}

		{
			let text = text(BottomRight);
			let pos = viewport - text_size_pix(text);
			buf.append(&layout_text(viewport, pos, text));
		}

		let vao = Arc::new(ctx.upload_meshbuffer(&buf));
		let shader = ctx.shader_pack.text();

		Object::new(vao, shader)
	}

	fn draw_crosshair(&self, sg: &mut SceneGraph) {
		let ctx = ctx();
		let center = sg.viewport_size / 2; // - (_EMBEDDED_CHAR_SIZE * UI_SCALE); // Note: - 2 * CHAR_SIZE / 2 (as crosshair is 2 chars wide)
		let text_x = center.x() - (_EMBEDDED_CHAR_SIZE.x() * UI_SCALE) * 2 / 2;
		let text_y = center.y() - (_EMBEDDED_CHAR_SIZE.y() * UI_SCALE) / 2;
		sg.push(Object::new(
			Arc::new(ctx.upload_meshbuffer(&layout_text(sg.viewport_size, uvec2(text_x, text_y), FONT_CROSSHAIR))),
			ctx.shader_pack.text(),
		));
	}
}

use std::cell::Cell;

/// Caches a single value.
pub struct SingleCache<T: Clone>(Cell<Option<T>>);

impl<T: Clone> SingleCache<T> {
	/// A clone of the cached value,
	/// or initialize first if needed.
	pub fn clone_or<F: FnOnce() -> T>(&self, f: F) -> T {
		let obj = self.0.take();
		let obj = match obj {
			Some(obj) => obj,
			None => f(),
		};
		let res = obj.clone();
		self.0.set(Some(obj));
		res
	}

	pub fn clear(&self) {
		self.0.set(None)
	}
}

impl<T: Clone> Default for SingleCache<T> {
	fn default() -> Self {
		Self(Cell::new(None))
	}
}
