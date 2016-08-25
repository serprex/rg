use specs::*;
use super::components::*;
use super::util::*;

fn create_aipos(w: &mut World, comps: &[(Chr, Ai, Pos)])
{
	for &(newch, newai, newpos) in comps {
		w.create_now()
			.with(newch)
			.with(newai)
			.with(newpos)
			.with(Race::None)
			.build();
	}
}

pub fn attack(src: Entity, w: &mut World) {
	let mut newent: Vec<(Chr, Ai, Pos)> = Vec::new();
	{
	let weapons = w.read::<Weapon>();
	let cch = w.read::<Chr>();
	let cpos = w.read::<Pos>();
	let watk = w.read::<Atk<Weapon>>();
	let wdirection = w.read::<WDirection>();
	if let Some(&Weapon(went)) = weapons.get(src) {
		if let Some(&wch) = cch.get(went) {
			if let Some(&Pos(pos)) = cpos.get(src) {
				if let Some(&WDirection(wdir)) =  wdirection.get(src) {
					let bp = xyz_plus_dir(pos, wdir);
					let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 0, 0));
					newent.push((wch, Ai::new(AiState::Melee(wstats.dur, wstats.dmg), 1), Pos(bp)));
					let mut cai = w.write::<Ai>();
					if let Some(mut ai) = cai.get_mut(src) {
						ai.tick = if wstats.spd < 0 {
							let spd = (-wstats.spd) as u8;
							if spd < ai.tick { ai.tick - spd } else { 0 }
						} else {
							ai.tick + wstats.spd as u8
						};
					}
				}
			}
		}
	}
	}
	create_aipos(w, &newent);
}

pub fn shoot(src: Entity, w: &mut World) {
	let mut newent: Vec<(Chr, Ai, Pos)> = Vec::new();
	{
	let weapons = w.read::<Weapon>();
	let bows = w.read::<Bow>();
	let cch = w.read::<Chr>();
	let cpos = w.read::<Pos>();
	let wdirection = w.read::<WDirection>();
	if let Some(&Weapon(went)) = weapons.get(src) {
		if let Some(&Bow(spd, dmg)) = bows.get(went) {
			if let Some(&wch) = cch.get(went) {
				if let Some(&Pos(pos)) = cpos.get(src) {
					if let Some(&WDirection(wdir)) =  wdirection.get(src) {
						let bp = xyz_plus_dir(pos, wdir);
						newent.push((wch, Ai::new(AiState::Missile(wdir, dmg), spd), Pos(bp)));
					}
				}
			}
		}
	}
	}
	create_aipos(w, &newent);
}

pub fn heal(src: Entity, w: &mut World) {
	let mut mortal = w.write::<Mortal>();
	let heal = w.read::<Heal>();
	if let Some(&mut Mortal(ref mut mo)) = mortal.get_mut(src) {
		if let Some(&Heal(amt)) = heal.get(src) {
			*mo += amt
		}
	}
}
