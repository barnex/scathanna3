//!
//! System to keep scores
//!

use super::internal::*;

/// Keeps scores and achievements (e.g. double kill).
#[derive(Default)]
pub(crate) struct Scores {
	by_player: HashMap<ID, Score>,
	pub by_team: [i32; NUM_TEAMS],
}

#[derive(Default)]
pub struct Score {
	pub total: i32,

	pub frags: u32,
	pub suicides: u32,

	pub multi_kills: u32,
	pub headshots: u32,
	pub deaths: u32,
}

impl Scores {
	pub fn join_new_player(&mut self, id: ID) {
		// make sure player is there with default (zero) score,
		// in case we format scores before the new player scores.
		self.by_player.entry(id).or_default();
	}

	pub fn by_player(&mut self, id: ID) -> &mut Score {
		self.by_player.entry(id).or_default()
	}

	pub fn by_team(&mut self, team: Team) -> &mut i32 {
		&mut self.by_team[team as usize]
	}

	pub fn iter(&self) -> impl Iterator<Item = (ID, &Score)> {
		self.by_player.iter().map(|(&id, score)| (id, score))
	}

	pub fn reset(&mut self, player_ids: impl Iterator<Item = ID>) {
		*self = default();
		for id in player_ids {
			self.join_new_player(id)
		}
	}

	pub fn max(&self) -> i32 {
		self.by_team.iter().copied().max().unwrap_or_default()
	}
}
/// Someone killed someone else
pub(crate) fn active_kill(state: &mut ServerState, actor: ID, victim: ID) -> Option<()> {
	let actor_team = player(state, actor)?.team;
	let vicitm_team = player(state, victim)?.team;

	if actor_team == vicitm_team {
		trace!("friendly fire {actor} -> {victim}");
		return None;
	}

	trace!("{actor} killed {victim}");

	//  "N frags remain gets announced when the leader makes progress"
	let remaining1 = state.scores.max() - state.autoswitch.frag_limit;

	*state.scores.by_team(actor_team) += 1;
	state.scores.by_player(actor).frags += 1;
	record_spree(state, actor);

	let remaining2 = state.scores.max() - state.autoswitch.frag_limit;
	if remaining1 != remaining2 {
		announce_remaining_frags(state)
	}

	log(state, format!("{} confettied {}", must_name(state, actor), must_name(state, victim)));
	hud_announce(state, Just(actor), format!("You confettied {}", must_name(state, victim)));
	hud_announce(state, Just(victim), format!("You got confettied by {}", must_name(state, actor)));

	kill(state, victim);

	Some(())
}

pub(crate) fn kill(state: &mut ServerState, victim: ID) -> Option<()> {
	despawn(state, victim)?;
	add_effect(state, Effect::particle_explosion(player(state, victim)?._center(), handle("star_blue"))); // << todo: color
	broadcast_scores(state);
	state.sprees.remove(&victim);
	Some(())
}

pub(crate) fn suicide(state: &mut ServerState, victim: ID, msg: &str) -> Option<()> {
	if player(state, victim)?.spawned {
		trace!("{victim} suicide");
		state.scores.by_player(victim).total -= 1;
		state.scores.by_player(victim).suicides += 1;
		kill(state, victim);
		log(state, format!("{} {}", must_name(state, victim), msg));
		hud_announce(state, Just(victim), format!("You {}", msg));
		sound_announce(state, Just(victim), handle("ann_be_careful"));
	}

	Some(())
}

pub(crate) fn broadcast_scores(state: &mut ServerState) {
	// Score delta:
	// 	`+N` against the second one if you're leading,
	//  `-N` against the leader if you're behind.
	let sorted = sorted(state.scores.by_team.to_vec()).with(|v| v.reverse());
	let top_score = sorted.get(0).copied().unwrap_or_default();
	let scnd_score = sorted.get(1).copied().unwrap_or_default();
	let delta = |score| if score == top_score { score - scnd_score } else { score - top_score };
	let max = state.autoswitch.frag_limit;

	let sec_remaining = f32::max(0.0, state.autoswitch.time_remaining()) as u32;
	let min = sec_remaining / 60;
	let sec = sec_remaining % 60;

	for (id, _score) in state.scores.iter() {
		let team = match player(state, id) {
			None => continue,
			Some(player) => player.team,
		};
		let score = state.scores.by_team[team as usize];
		let delta = delta(score);
		let text = format!("time: {min}:{sec:02}\n{team}: {score} / {max} ({delta:+})");

		state.diffs.push(
			UpdateHUD(HUDUpdate {
				pos: HUDPos::TopLeft,
				text,
				ttl_sec: 3.0,
			})
			.to_just(id),
		);
	}
}

fn announce_winner(state: &mut ServerState) {
	use Team::*;
	let top_score = state.scores.max();
	let winning_team = [Red, Green, Blue].into_iter().find(|&t| *state.scores.by_team(t) == top_score);
	if let Some(winning_team) = winning_team {
		hud_announce(state, All, format!("Team {winning_team} wins!"));
		sound_announce(
			state,
			All,
			match winning_team {
				Red => handle("ann_red_wins"),
				Green => handle("ann_green_wins"),
				Blue => handle("ann_blue_wins"),
			},
		);
	}

	let sorted_teams = vec![Red, Green, Blue].with(|v| v.sort_by_key(|&team| state.scores.by_team[team as usize])).with(|v| v.reverse());
	//let sorted_players = world.players().collect::<Vec<_>>().with(|v|v.sort_by_key(|id|world.player(id).map()))
	use std::fmt::Write;
	let mut scores = String::new();
	for team in sorted_teams {
		let _ = writeln!(&mut scores, "\n\nTeam {team}");
		let _ = writeln!(&mut scores, "___________________________________________\n");
		for id in ids(&state.entities.players) {
			if player(state, id).map(|p| p.team) == Some(team) {
				let score = state.scores.by_player(id);
				let frags = score.frags;
				let deaths = score.deaths;
				let _ = writeln!(&mut scores, "{:+20}: {:2} frags | {:2} deaths", must_name(state, id), frags, deaths);
			}
		}
	}
	println!("{}", &scores);
	hud_announce2(state, All, scores);
}
