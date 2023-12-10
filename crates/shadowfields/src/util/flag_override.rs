// Utility to override settings via optional CLI flags. E.g.:
//   override_(&mut settings.player.team, flags.team);
pub fn flag_override<T>(dst: &mut T, src: Option<T>) {
	if let Some(v) = src {
		*dst = v;
	}
}
