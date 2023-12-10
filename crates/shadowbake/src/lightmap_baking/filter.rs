use crate::prelude::*;

pub(crate) fn hole_filter2(lm: &HashMap<Handle, Img<Accum>>) -> HashMap<Handle, Img<Accum>> {
	hole_filter(&hole_filter(lm))
}

pub(crate) fn hole_filter(lm: &HashMap<Handle, Img<Accum>>) -> HashMap<Handle, Img<Accum>> {
	lm.iter().map(|(k, v)| (*k, hole_filter_(v))).collect()
}

// Texels with zero samples get replaced by the weighted average of their neighbors
fn hole_filter_(img: &Img<Accum>) -> Img<Accum> {
	let mut dst = Img::<Accum>::new(img.size());

	for src_pix in img.pixel_positions() {
		if img.ref_at(src_pix).num_samples() == 0 {
			// Texel does not have samples: replace by average of neighbors
			let src_pix = src_pix.to_i32();

			for delta in cross([-1, 0, 1], [-1, 0, 1]) {
				let neigh = src_pix + delta;
				if in_bounds(img.size(), neigh) {
					let neigh = img.ref_at(neigh.to_u32());
					if neigh.num_samples() != 0 {
						dst.at_mut(src_pix.to_u32()).add_other(neigh);
					}
				}
			}
		} else {
			// Texel has samples: keep value
			*dst.at_mut(src_pix) = img.ref_at(src_pix).clone();
		}
	}

	dst
}

fn in_bounds(size: uvec2, pix: ivec2) -> bool {
	let (w, h) = size.to_i32().into();
	pix.x() >= 0 && pix.x() < w && pix.y() >= 0 && pix.y() < h
}

pub(crate) fn superblock_filter(lm: &HashMap<Handle, Img<Accum>>) -> HashMap<Handle, Img<Accum>> {
	lm.iter().map(|(k, v)| (*k, superblock_filter_(v))).collect()
}

fn superblock_filter_(img: &Img<Accum>) -> Img<Accum> {
	let mut dst = Img::<Accum>::new(img.size());

	for dst_pix in dst.pixel_positions() {
		let dst_pix = dst_pix.to_i32();

		const D: i32 = (SUPERBLK as i32 - 1) / 2;
		debug_assert!(D > 0);
		for delta in cross(-D..=D, -D..=D) {
			let src_pix = dst_pix + delta;

			if in_bounds(dst.size(), src_pix) {
				dst.at_mut(dst_pix.to_u32()).add_other(img.ref_at(src_pix.to_u32()));
			}
		}
	}

	dst
}
