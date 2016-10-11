use rand::{self, Rng};
use specs::*;
use smallvec::SmallVec;

use components::*;
use util::*;
use position::Possy;

/*
pub fn move(np: [i16; 2], src: Entity, w: &mut World) {

}*/

pub fn movedir(dir: Dir, src: Entity, w: &mut World) {
	let mut possy = w.write_resource::<Possy>();
	let crace = w.read::<Race>();
	let (mut mort, portal, mut ai, mut solid) =
		(w.write::<Mortal>(), w.read::<Portal>(), w.write::<Ai>(), w.write::<Solid>());
	if let Some(pos) = possy.get_pos(src) {
		let Walls(ref walls) = *w.read_resource::<Walls>();
		let np = xyz_plus_dir(pos, dir);
		if walls.contains_key(&np) {
			if let Some(&race) = crace.get(src) {
				if race == Race::None {
					w.delete_later(src)
				}
			}
			return
		}
		for &e in possy.get_ents(np).iter() {
			if solid.get(e).is_some() {
				return
			}
		}
		possy.set_pos(src, np);
	}
	let mut rmai = SmallVec::<[Entity; 2]>::new();
	let mut spos = SmallVec::<[(Entity, [i16; 3]); 2]>::new();
	for &xyz in possy.collisions.iter() {
		let col = possy.get_ents(xyz);
		for e in col.iter().cloned() {
			for ce in col.iter().cloned().filter(|&ce| ce != e) {
				if let Some(&Portal(porx)) = portal.get(e) {
					spos.push((ce, porx));
				}
				if let Some(aie) = ai.get(e) {
					match aie.state {
						AiState::Missile(_, dmg) => {
							if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
								if *mce <= dmg {
									*mce = 0;
									solid.remove(ce);
									rmai.push(ce);
								} else {
									*mce -= dmg;
								}
							}
							if let Some(_) = solid.get(ce) {
								w.delete_later(e)
							}
						},
						AiState::Melee(_, dmg) => {
							if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
								if *mce <= dmg {
									*mce = 0;
									solid.remove(ce);
									rmai.push(ce);
								} else {
									*mce -= dmg;
								}
							}
						},
						_ => (),
					}
				}
			}
		}
	}
	for e in rmai {
		ai.remove(e);
	}
	for (e, p) in spos {
		possy.set_pos(e, p);
	}
}

pub fn attack(dir: Dir, src: Entity, w: &mut World) {
	let weapons = w.read::<Weapon>();
	let mut cch = w.write::<Chr>();
	let mut cai = w.write::<Ai>();
	let mut cpos = w.write_resource::<Possy>();
	let mut cp = w.write::<Pos>();
	let mut crace = w.write::<Race>();
	let watk = w.read::<Atk<Weapon>>();
	if let Some(&Weapon(went)) = weapons.get(src) {
		if let Some(&wch) = cch.get(went) {
			if let Some(pos) = cpos.get_pos(src) {
				let bp = xyz_plus_dir(pos, dir);
				let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 0, 0));
				let newent = w.create_later();
				cch.insert(newent, wch);
				cai.insert(newent, Ai::new(AiState::Melee(wstats.dur, wstats.dmg), 1));
				cp.insert(newent, Pos);
				crace.insert(newent, Race::None);
				cpos.set_pos(newent, bp);
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

pub fn lunge(dir: Dir, src: Entity, w: &mut World) {
	movedir(dir, src, w);
	attack(dir, src, w);
}

pub fn shoot(dir: Dir, src: Entity, w: &mut World) {
	let weapons = w.read::<Weapon>();
	let bows = w.read::<Bow>();
	let mut cch = w.write::<Chr>();
	let mut cpos = w.write_resource::<Possy>();
	let mut cp = w.write::<Pos>();
	let mut crace = w.write::<Race>();
	let mut cai = w.write::<Ai>();
	if let Some(&Weapon(went)) = weapons.get(src) {
		if let Some(&Bow(spd, dmg)) = bows.get(went) {
			if let Some(&wch) = cch.get(went) {
				if let Some(pos) = cpos.get_pos(src) {
					let bp = xyz_plus_dir(pos, dir);
					let newent = w.create_later();
					cch.insert(newent, wch);
					cai.insert(newent, Ai::new(AiState::Missile(dir, dmg), spd));
					cp.insert(newent, Pos);
					crace.insert(newent, Race::None);
					cpos.set_pos(newent, bp);
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
	let mut possy = w.write_resource::<Possy>();
	if let Some(pxy) = possy.get_pos(src) {
		let mut rng = rand::thread_rng();
		possy.set_pos(src, [rng.gen_range(0, 40), rng.gen_range(0, 40), pxy[2]]);
	}
}

pub fn grab(xyz: [i16; 3], src: Entity, w: &mut World) {
	let strength = w.read::<Strength>();
	let mut bag = w.write::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) = (strength.get(src), bag.get_mut(src)) {
		let weight = w.read::<Weight>();
		let mut cpos = w.write::<Pos>();
		let possy = w.read_resource::<Possy>();
		let mut rmpos = Vec::new();
		let mut totwei = 0;
		for &Weight(wei) in ebag.iter().filter_map(|&e| weight.get(e)) {
			totwei += wei as i32;
		}
		for &ent in possy.get_ents(xyz).iter() {
			if let (Some(&Weight(wei)), Some(_)) = (weight.get(ent), cpos.get(ent)) {
				if totwei + wei as i32 <= strg as i32 {
					ebag.push(ent);
					rmpos.push(ent);
					totwei += wei as i32;
				}
			}
		}
		for ent in rmpos {
			cpos.remove(ent);
		}
	}
}
