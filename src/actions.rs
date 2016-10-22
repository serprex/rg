use rand::Rng;
use specs::*;
use smallvec::SmallVec;

use components::*;
use util::*;
use position::Possy;

pub fn movedir(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	if let Some(np) = {
		let possy = w.read_resource::<Possy>();
		possy.get_pos(src).map(|pos| xyz_plus_dir(pos, dir))
	} {
		moveto(np, src, rng, w)
	}
}

pub fn colcheck(src: Entity, rng: &mut R, w: &mut World) {
	if let Some(np) = {
		let possy = w.read_resource::<Possy>();
		possy.get_pos(src)
	} {
		moveto(np, src, rng, w)
	}
}

pub fn moveto(np: [i16; 3], src: Entity, _rng: &mut R, w: &mut World) {
	let mut possy = w.write_resource::<Possy>();
	let mut mort = w.write::<Mortal>();
	let portal = w.read::<Portal>();
	let fragile = w.read::<Fragile>();
	let mut ai = w.write::<Ai>();
	let mut solid = w.write::<Solid>();
	let Walls(ref walls) = *w.read_resource::<Walls>();
	if walls.contains_key(&np) {
		if fragile.get(src).is_some() {
			w.delete_later(src)
		}
		return
	}
	if solid.get(src).is_some() {
		for &e in possy.get_ents(np).iter() {
			if solid.get(e).is_some() {
				return
			}
		}
	}
	possy.set_pos(src, np);
	let mut rmai = SmallVec::<[Entity; 2]>::new();
	let mut spos = SmallVec::<[(Entity, [i16; 3]); 2]>::new();
	for ce in possy.get_ents(np).iter().cloned().filter(|&ce| ce != src) {
		if let Some(&Portal(porx)) = portal.get(ce) {
			spos.push((src, porx));
		}
		if let Some(aie) = ai.get(src) {
			match aie.state {
				AiState::Missile(_, dmg, _) => {
					if solid.get(ce).is_some() {
						if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
							if *mce <= dmg {
								*mce = 0;
								solid.remove(ce);
								rmai.push(ce);
							} else {
								*mce -= dmg;
							}
						}
						if fragile.get(src).is_some() {
							w.delete_later(src);
						} else {
							rmai.push(src);
						}
					}
				},
				AiState::Melee(_, dmg, _) => {
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
	for e in rmai {
		ai.remove(e);
	}
	for (e, p) in spos {
		possy.set_pos(e, p);
	}
}

pub fn attack(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	let (bp, newent) = {
		let weapons = w.read::<Weapon>();
		let mut cch = w.write::<Chr>();
		let mut cai = w.write::<Ai>();
		let cpos = w.read_resource::<Possy>();
		let watk = w.read::<Atk<Weapon>>();
		if let Some(&Weapon(went)) = weapons.get(src) {
			if let Some(&wch) = cch.get(went) {
				if let Some(pos) = cpos.get_pos(src) {
					let cstr = w.read::<Strength>();
					let &Strength(srcstr) = cstr.get(went).unwrap_or(&Strength(1));
					let bp = xyz_plus_dir(pos, dir);
					let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 1, 0));
					cai.insert(went, Ai::new(AiState::Melee(wstats.dur, (srcstr / (wstats.dur as i16 * 4)) + wstats.dmg, src), 1));
					if wstats.spd != 0 {
						if let Some(mut ai) = cai.get_mut(src) {
							ai.tick = if wstats.spd < 0 {
								let spd = (-wstats.spd) as u8;
								if spd < ai.tick { ai.tick - spd } else { 0 }
							} else {
								ai.tick + wstats.spd as u8
							};
						}
					}
					(bp, went)
				} else { return }
			} else { return }
		} else { return }
	};
	moveto(bp, newent, rng, w)
}

pub fn lunge(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	movedir(dir, src, rng, w);
	attack(dir, src, rng, w);
}

pub fn shoot(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	let (went, sent) = {
		let weapons = w.read::<Weapon>();
		let mut shields = w.write::<Shield>();
		if let Some(&Weapon(went)) = weapons.get(src) {
			if let Some(Shield(sent)) = shields.remove(src) {
				(went, sent)
			} else { return }
		} else { return }
	};
	throw(dir, src, went, sent, rng, w)
}

pub fn throw(dir: Dir, psrc: Entity, tsrc: Entity, obj: Entity, rng: &mut R, w: &mut World) {
	let (bp, ai, ch) = {
		let possy = w.read_resource::<Possy>();
		if let Some(pos) = possy.get_pos(psrc) {
			let cstr = w.read::<Strength>();
			let cwei = w.read::<Weight>();
			let cchr = w.read::<Chr>();
			let mut cai = w.write::<Ai>();
			let bp = xyz_plus_dir(pos, dir);
			let &Strength(srcstr) = cstr.get(tsrc).unwrap_or(&Strength(1));
			let &Weight(objwei) = cwei.get(obj).unwrap_or(&Weight(1));
			let dmg = srcstr as i16 + objwei as i16 / 2;
			let spd = 1 + (objwei as i16 * 8 / srcstr as i16) as u8;
			if let Some(&ch) = cchr.get(obj) {
				(bp, Ai::new(AiState::Missile(dir, dmg, 108/spd), spd), ch)
			} else {
				return
			}
		} else {
			return
		}
	};
	let newent = w.create_now()
		.with(ai)
		.with(ch)
		.build();
	moveto(bp, newent, rng, w)
}

pub fn heal(src: Entity, amt: i16, _rng: &mut R, w: &mut World) {
	let mut mortal = w.write::<Mortal>();
	if let Some(&mut Mortal(ref mut mo)) = mortal.get_mut(src) {
		*mo += amt
	}
}

pub fn blink(src: Entity, rng: &mut R, w: &mut World) {
	let np = if let Some(pxy) = w.write_resource::<Possy>().get_pos(src) {
		[rng.gen_range(0, 40), rng.gen_range(0, 40), pxy[2]]
	} else {
		return
	};
	moveto(np, src, rng, w);
}

pub fn grab(xyz: [i16; 3], src: Entity, _rng: &mut R, w: &mut World) {
	let strength = w.read::<Strength>();
	let mut bag = w.write::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) = (strength.get(src), bag.get_mut(src)) {
		let weight = w.read::<Weight>();
		let mut possy = w.write_resource::<Possy>();
		let mut rmpos = Vec::new();
		let mut totwei: i32 = ebag.iter().filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32)).sum();
		for &ent in possy.get_ents(xyz).iter() {
			if let Some(&Weight(wei)) = weight.get(ent) {
				if totwei + wei as i32 <= strg as i32 {
					ebag.push(ent);
					rmpos.push(ent);
					totwei += wei as i32;
				}
			}
		}
		for ent in rmpos {
			possy.remove(ent);
		}
	}
}

pub fn pickup(src: Entity, ent: Entity, _rng: &mut R, w: &mut World) {
	let strength = w.read::<Strength>();
	let mut bag = w.write::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) = (strength.get(src), bag.get_mut(src)) {
		let weight = w.read::<Weight>();
		let mut possy = w.write_resource::<Possy>();
		let mut rmpos = Vec::new();
		let mut totwei: i32 = ebag.iter().filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32)).sum();
		if let Some(&Weight(wei)) = weight.get(ent) {
			if totwei + wei as i32 <= strg as i32 {
				ebag.push(ent);
				rmpos.push(ent);
				totwei += wei as i32;
			}
		}
		for ent in rmpos {
			possy.remove(ent);
		}
	}
}
