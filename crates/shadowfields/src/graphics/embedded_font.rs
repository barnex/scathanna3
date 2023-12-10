use super::internal::*;

/// Size (in pixels) of a single character in the embedded font map.
pub const _EMBEDDED_CHAR_SIZE: uvec2 = uvec2(8, 16);

/// Overall size (in pixels) of the embedded font map.
pub const EMBEDDED_FONTMAP_SIZE: uvec2 = uvec2(128, 128);

/// Special symbol in font.png.
#[allow(unused)]
pub const FONT_HEART: &str = "\x02\x03";

/// Special symbol in font.png.
#[allow(unused)]
pub const FONT_PLUS: &str = "\x04\x05";

/// Special symbol in font.png.
#[allow(unused)]
pub const FONT_CROSSHAIR: &str = "\x06\x07";

/// Special symbol in font.png.
#[allow(unused)]
pub const FONT_SHIELD: &str = "\x08\x09";
