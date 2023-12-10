pub use crate::*; // ☠️

pub use shadowfields::game::*;
pub use shadowfields::geom::*;
pub use shadowfields::img::*;
pub use shadowfields::map::*;
pub use shadowfields::prelude::*;
pub use shadowfields::raytrace::*;
pub use shadowfields::resources::*;
pub use shadowfields::util::*;

pub type Buffers = Vec<gltf::buffer::Data>;
pub type Position = vec3;
pub type Normal = vec3;
pub type TexCoord = vec2;
pub type LightCoord = vec2;
pub type Color = vec3;
pub type MaterialHandle = Handle;
pub type LightmapHandle = Handle;

pub use image::Rgb;
pub use image::RgbImage;
pub use rand::{prelude::StdRng, Rng};
pub use rand_xoshiro::rand_core::SeedableRng;
pub use rand_xoshiro::Xoshiro256PlusPlus;
pub use rayon::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use smallvec::*;
pub use std::ops::{Add, Div};
pub use vector::*;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type Set<T> = fnv::FnvHashSet<T>;

/// Shorthand for Default::default()
pub fn default<T: Default>() -> T {
	T::default()
}

pub use anyhow::Context;
pub use anyhow::Error;
pub use anyhow::Result;
pub use anyhow::anyhow;
pub use anyhow::bail;
pub use itertools::Itertools;
pub use log::debug;
pub use matrix::*;
pub use std::iter::zip;
pub use std::ops::Mul;
pub use std::path::Path;
pub use std::path::PathBuf;
pub use std::str::FromStr;
pub use vector::*;

pub type Rand = rand_xoshiro::Xoshiro256PlusPlus;
