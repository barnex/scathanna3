//! Game logic specific to the client.

pub mod internal;

mod advantage_system;
mod client;
mod client_command_system;
mod console_system;
mod debug_overlay_system;
mod drawing_system;
mod effects_system;
mod extrapolation_system;
mod footstep_system;
mod join_system;
mod jump_pad_system;
mod loading_screen;
mod mapswitch_system;
mod player_movement_system;
mod respawn_system;
mod server_message_system;
mod sound_system;
mod volumetric_light;
mod weapon_system;

pub(crate) use advantage_system::*;
pub(crate) use client::*;
pub(crate) use client_command_system::*;
pub(crate) use debug_overlay_system::*;
pub(crate) use drawing_system::*;
pub(crate) use loading_screen::*;
pub use volumetric_light::*;
