use crate::prelude::*;

/// Validate assumptions that must hold for the GLTF file:
///   * lightcoords are in range [0,1].
pub(crate) fn validate_gltf(parsed_gltf: &ParsedGltf) -> Result<()> {
	let mut errors = vec![];

	errors.extend(validate_lightcoords(parsed_gltf));

	match errors.len() {
		0 => Ok(()),
		n => Err(anyhow!("{n} errors:\n{}", errors.into_iter().map(|e| e.to_string()).join("\n"))),
	}
}

// Check that lightcoords are valid. Return at most one error per object.
fn validate_lightcoords(parsed_gltf: &ParsedGltf) -> Vec<Error> {
	let mut errors = vec![];

	for obj in &parsed_gltf.objects {
		let name = &obj.name;
		'prim_loop: for prim in &obj.primitives {
			// no explicit lightcoords: texcoords are used instead,
			// in that case they must meet the same requirements as lightcoords.
			let lightcoords = prim.mesh.lightcoords.as_ref().unwrap_or(&prim.mesh.texcoords);
			for &lightcoord in lightcoords {
				if !is_valid_lightcoord(lightcoord) {
					errors.push(anyhow!("{name}: invalid lightcoord: {lightcoord}"));
					break 'prim_loop;
				}
			}
		}
	}

	errors
}

// Light coordinates outside [0, 1] are invalid because they map outside of the lightmap texture.
fn is_valid_lightcoord(lightcoord: vec2) -> bool {
	// leave a little leeway for round-off errors, which, unfortunately, happen in Blender.
	lightcoord.iter().all(|v| v >= -0.0000001 && v <= 1.0000001)
}
