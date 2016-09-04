use rand::{self, Rng};
use specs::*;
use super::components::*;
use super::util::*;

pub fn attack(src: Entity, w: &mut World) {
	let weapons = w.read::<Weapon>();
	let mut cch = w.write::<Chr>();
	let mut cai = w.write::<Ai>();
	let mut cpos = w.write::<Pos>();
	let watk = w.read::<Atk<Weapon>>();
	let wdirection = w.read::<WDirection>();
	if let Some(&Weapon(went)) = weapons.get(src) {
		if let Some(&wch) = cch.get(went) {
			if let Some(&Pos(pos)) = cpos.get(src) {
				if let Some(&WDirection(wdir)) =  wdirection.get(src) {
					let bp = xyz_plus_dir(pos, wdir);
					let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 0, 0));
					let newent = w.create_later();
					cch.insert(newent, wch);
					cai.insert(newent, Ai::new(AiState::Melee(wstats.dur, wstats.dmg), 1));
					cpos.insert(newent, Pos(bp));
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

pub fn shoot(src: Entity, w: &mut World) {
	let weapons = w.read::<Weapon>();
	let bows = w.read::<Bow>();
	let mut cch = w.write::<Chr>();
	let mut cpos = w.write::<Pos>();
	let wdirection = w.read::<WDirection>();
	let mut cai = w.write::<Ai>();
	if let Some(&Weapon(went)) = weapons.get(src) {
		if let Some(&Bow(spd, dmg)) = bows.get(went) {
			if let Some(&wch) = cch.get(went) {
				if let Some(&Pos(pos)) = cpos.get(src) {
					if let Some(&WDirection(wdir)) =  wdirection.get(src) {
						let bp = xyz_plus_dir(pos, wdir);
						let newent = w.create_later();
						cch.insert(newent, wch);
						cai.insert(newent, Ai::new(AiState::Missile(wdir, dmg), spd));
						cpos.insert(newent, Pos(bp));
					}
				}
			}
		}
	}
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

pub fn blink(src: Entity, w: &mut World) {
	let mut npos = w.write::<NPos>();
	let pos = w.read::<Pos>();
	if let Some(&Pos(pxy)) = pos.get(src) {
		let mut rng = rand::thread_rng();
		npos.insert(src, NPos([rng.gen_range(0, 40), rng.gen_range(0, 40), pxy[2]]));
	}
}
