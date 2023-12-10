use super::internal::*;

pub const UI_SCALE: u32 = 2;

/// A mesh for rendering text at a given position on the screen (using the embedded bitmap font).
/// Wraps long lines as shown below:
///
///   viewport size
///  +----------------+
///  |  `pos`+        |
///  |        your tex|
///  |        t here  |
///  |                |
///  +----------------+
///
pub(crate) fn layout_text(viewport_size: uvec2, pos: uvec2, text: &str) -> MeshBuffer {
	let char_stride = UI_SCALE * _EMBEDDED_CHAR_SIZE;

	let mut buf = MeshBuffer::new();

	let mut char_pos = pos;
	for &byte in text.as_bytes() {
		// newline
		if byte == b'\n' {
			char_pos[0] = pos.x();
			char_pos[1] += char_stride.y();
			continue;
		}

		// wrap long lines
		if char_pos.x() > viewport_size.x() - char_stride.x() {
			char_pos[0] = pos.x();
			char_pos[1] += char_stride.y();
		}

		buf.append(&blit_chr(viewport_size, char_pos, byte, UI_SCALE));

		char_pos[0] += char_stride.x();
	}

	buf
}

// TODO: impl Scenegraph
pub(crate) fn push_text(sg: &mut SceneGraph, text: &str) {
	let ctx = ctx();
	let pos = uvec2(0, 0);
	let buf = layout_text(sg.viewport_size, pos, text);
	let vao = Arc::new(ctx.upload_meshbuffer(&buf));
	let shader = ctx.shader_pack.text();
	sg.push(Object::new(vao, shader))
}

/// Like `layout_text`, but puts the text at the bottom left of the screen.
///
///    viewport size
///  +----------------+
///  |                |
///  |                |
///  |your text       |
///  |here            |
///  +----------------+
///
pub(crate) fn layout_text_bottom(scrn_pixels: uvec2, text: &str) -> MeshBuffer {
	let y = scrn_pixels.y() - UI_SCALE * _EMBEDDED_CHAR_SIZE.y() * text_height_chars(text);
	let x = 0;
	let pos = uvec2(x, y);
	layout_text(scrn_pixels, pos, text)
}

pub(crate) fn layout_text_right(scrn_pixels: uvec2, text: &str) -> MeshBuffer {
	let y = 0;
	let x = scrn_pixels.x() - UI_SCALE * _EMBEDDED_CHAR_SIZE.x() * text_width_chars(text);
	let pos = uvec2(x, y);
	layout_text(scrn_pixels, pos, text)
}

pub(crate) fn text_height_chars(text: &str) -> u32 {
	text.lines().count() as u32
}

pub(crate) fn text_width_chars(text: &str) -> u32 {
	text.lines().map(str::len).max().unwrap_or(0) as u32
}

pub(crate) fn text_size_chars(text: &str) -> uvec2 {
	uvec2(text_width_chars(text), text_height_chars(text))
}

pub(crate) fn text_size_pix(text: &str) -> uvec2 {
	uvec2(text_width_chars(text), text_height_chars(text)).mul(_EMBEDDED_CHAR_SIZE)
}

pub(crate) fn viewport_size_chars(viewport_size_pix: uvec2) -> uvec2 {
	viewport_size_pix / (_EMBEDDED_CHAR_SIZE * UI_SCALE)
}

/// A mesh for copying a single character to the screen.
fn blit_chr(scrn_pixels: uvec2, scrn_pos: uvec2, char: u8, scale: u32) -> MeshBuffer {
	let tex_pixels = EMBEDDED_FONTMAP_SIZE;
	let sprite_pixels = _EMBEDDED_CHAR_SIZE;
	let tex_pos = chr_tex_pos_16x8(char, sprite_pixels);

	blit(tex_pixels, tex_pos, sprite_pixels, scrn_pixels, scrn_pos, scale)
}

/// Pixel position (top-left corner) of an ascii character in the embedded font map.
fn chr_tex_pos_16x8(char: u8, sprite_pixels: uvec2) -> uvec2 {
	let x = (char & 0xf) as u32;
	let y = (char >> 4) as u32;
	uvec2(x, y) * sprite_pixels
}
