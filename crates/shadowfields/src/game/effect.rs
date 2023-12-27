use super::internal::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Effect {
	pub ttl: f32, // <<<< 🪲 BAD design, remove or add total_ttl, duplicate with PARTICLE_BEAM_TTL etc. used inconsistently.
	pub typ: EffectType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectType {
	ParticleExplosion { pos: vec3, texture: Handle },
	Debris { pos: vec3, normal: vec3, texture: Handle },
	ParticleBeam { start: vec3, orientation: Orientation, len: f32, texture: Handle },
}

pub const PARTICLE_BEAM_TTL: f32 = 1.8; // seconds
pub const PARTICLE_EXPLOSION_TTL: f32 = 1.8; // seconds
pub const DEBRIS_TTL: f32 = 3.0; // seconds
pub const RESPAWN_TTL: f32 = 1.5; // seconds

impl Effect {
	pub fn particle_explosion(pos: vec3, texture: Handle) -> Self {
		Self {
			ttl: PARTICLE_EXPLOSION_TTL,
			typ: EffectType::ParticleExplosion { pos, texture },
		}
	}

	pub fn debris(pos: vec3, normal: vec3, texture: Handle) -> Self {
		Self {
			ttl: DEBRIS_TTL,
			typ: EffectType::Debris { pos, normal, texture },
		}
	}

	pub fn particle_beam(start: vec3, orientation: Orientation, len: f32, texture: Handle) -> Self {
		Self {
			ttl: PARTICLE_BEAM_TTL,
			typ: EffectType::ParticleBeam { start, orientation, len, texture },
		}
	}
}
