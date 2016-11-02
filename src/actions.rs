use rand::Rng;
use specs::*;
use smallvec::SmallVec;

use components::*;
use position::Possy;
use tick::Ticker;
use util::*;

pub enum Action {
	Action(Box<Fn(&mut R, &mut World) + Send + Sync>),
	Movedir{ dir: Dir, src: Entity },
	Colcheck { src: Entity },
	Moveto { np: [i16; 3], src: Entity },
	Melee { dur: u8, src: Entity, ent: Entity },
	Missile { spd: u32, dir: Dir, dur: u8, ent: Entity },
	Attack { dir: Dir, src: Entity },
	Lunge { dir: Dir, src: Entity },
	Shoot { dir: Dir, src: Entity },
	Throw { dir: Dir, psrc: Entity, tsrc: Entity, obj: Entity },
	Heal { src: Entity, amt: i16 },
	Blink { src: Entity },
	Grab { xyz: [i16; 3], src: Entity },
	Pickup { src: Entity, ent: Entity },
}

impl Action {
	pub fn call(self, rng: &mut R, w: &mut World) {
		match self {
			Action::Action(a) => a(rng, w),
			Action::Movedir { dir: dir, src: src } => movedir(dir, src, rng, w),
			Action::Colcheck { src: src } => colcheck(src, rng, w),
			Action::Moveto { np: np, src: src } => moveto(np, src, rng, w),
			Action::Melee { dur: dur, src: src, ent: ent } => melee(dur, src, ent, rng, w),
			Action::Missile { spd: spd, dir: dir, dur: dur, ent: ent} => missile(spd, dir, dur, ent, rng, w),
			Action::Attack { dir: dir, src: src } => attack(dir, src, rng, w),
			Action::Lunge { dir: dir, src: src } => lunge(dir, src, rng, w),
			Action::Shoot { dir: dir, src: src } => shoot(dir, src, rng, w),
			Action::Throw { dir: dir, psrc: psrc, tsrc: tsrc, obj: obj } => throw(dir, psrc, tsrc, obj, rng, w),
			Action::Heal { src: src, amt: amt } => heal(src, amt, rng, w),
			Action::Blink { src: src } => blink(src, rng, w),
			Action::Grab { xyz: xyz, src: src } => grab(xyz, src, rng, w),
			Action::Pickup { src: src, ent: ent } => pickup(src, ent, rng, w),
		}
	}
}

fn movedir(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	if let Some(np) = {
		let possy = w.read_resource::<Possy>();
		possy.get_pos(src).map(|pos| xyz_plus_dir(pos, dir))
	} {
		moveto(np, src, rng, w)
	}
}

fn colcheck(src: Entity, rng: &mut R, w: &mut World) {
	if let Some(np) = {
		let possy = w.read_resource::<Possy>();
		possy.get_pos(src)
	} {
		moveto(np, src, rng, w)
	}
}

fn moveto(np: [i16; 3], src: Entity, _rng: &mut R, w: &mut World) {
	let mut possy = w.write_resource::<Possy>();
	let mut mort = w.write::<Mortal>();
	let portal = w.read::<Portal>();
	let fragile = w.read::<Fragile>();
	let mut solid = w.write::<Solid>();
	let Walls(ref walls) = *w.read_resource::<Walls>();
	if walls.contains_key(&np) {
		if fragile.get(src).is_some() {
			w.delete_later(src);
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
	let mut ai = w.write::<Ai>();
	let mut misl = w.write::<Dmg>();
	let mut rmai = SmallVec::<[Entity; 1]>::new();
	let mut rmisl = SmallVec::<[Entity; 1]>::new();
	let mut spos = Vec::new();
	for ce in possy.get_ents(np).iter().cloned().filter(|&ce| ce != src) {
		if let Some(&Portal(porx)) = portal.get(ce) {
			spos.push((src, porx));
		}
		if let Some(&Dmg(dmg)) =  misl.get(src) {
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
				} else { // TODO enable persisting attack
					rmisl.push(src);
				}
			}
		}
	}
	for e in rmai {
		ai.remove(e);
	}
	for e in rmisl {
		misl.remove(e);
	}
	for (e, p) in spos {
		possy.set_pos(e, p);
	}
}

fn melee(dur: u8, src: Entity, ent: Entity, rng: &mut R, w: &mut World) {
	if dur == 0 {
		let mut weapons = w.write::<Weapon>();
		let mut possy = w.write_resource::<Possy>();
		weapons.insert(src, Weapon(ent));
		possy.remove(ent);
	} else {
		{
		let mut ticker = w.write_resource::<Ticker>();
		ticker.push(1, Action::Melee { dur: dur - 1, src: src, ent: ent });
		}
		colcheck(ent, rng, w);
	}
}

fn missile(spd: u32, dir: Dir, dur: u8, ent: Entity, rng: &mut R, w: &mut World) {
	if dur == 0 {
		let mut cm = w.write::<Dmg>();
		cm.remove(ent);
	} else {
		{
		let mut ticker = w.write_resource::<Ticker>();
		ticker.push(spd, Action::Missile { spd: spd, dir: dir, dur: dur - 1, ent: ent });
		}
		movedir(dir, ent, rng, w);
	}
}

fn attack(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	let (bp, went) = {
		let mut weapons = w.write::<Weapon>();
		let mut cch = w.write::<Chr>();
		let mut cai = w.write::<Ai>();
		let cpos = w.read_resource::<Possy>();
		let watk = w.read::<Atk<Weapon>>();
		let mut misl = w.write::<Dmg>();
		if let Some(Weapon(went)) = weapons.remove(src) {
			if let Some(&wch) = cch.get(went) {
				if let Some(pos) = cpos.get_pos(src) {
					let cstr = w.read::<Strength>();
					let &Strength(srcstr) = cstr.get(went).unwrap_or(&Strength(1));
					let bp = xyz_plus_dir(pos, dir);
					let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 1, 0));
					let mut ticker = w.write_resource::<Ticker>();
					misl.insert(went, Dmg(srcstr / 4 + wstats.dmg));
					let dur = wstats.dur;
					ticker.push(1, Action::Melee { dur: dur, src: src, ent: went });
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
	moveto(bp, went, rng, w)
}

fn lunge(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
	movedir(dir, src, rng, w);
	attack(dir, src, rng, w);
}

fn shoot(dir: Dir, src: Entity, rng: &mut R, w: &mut World) {
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

fn throw(dir: Dir, psrc: Entity, tsrc: Entity, obj: Entity, rng: &mut R, w: &mut World) {
	let bp = {
		let possy = w.read_resource::<Possy>();
		if let Some(pos) = possy.get_pos(psrc) {
			let cstr = w.read::<Strength>();
			let cwei = w.read::<Weight>();
			let cchr = w.read::<Chr>();
			let mut ticker = w.write_resource::<Ticker>();
			let mut misl = w.write::<Dmg>();
			let bp = xyz_plus_dir(pos, dir);
			let &Strength(srcstr) = cstr.get(tsrc).unwrap_or(&Strength(1));
			let &Weight(objwei) = cwei.get(obj).unwrap_or(&Weight(1));
			let dmg = srcstr as i16 + objwei as i16 / 2;
			let spd = 1 + (objwei as i16 * 8 / srcstr as i16) as u32;
			misl.insert(obj, Dmg(dmg));
			ticker.push(spd, Action::Missile { spd: spd, dir: dir, dur: (108/spd) as u8, ent: obj });
			bp
		} else { return }
	};
	moveto(bp, obj, rng, w)
}

fn heal(src: Entity, amt: i16, _rng: &mut R, w: &mut World) {
	let mut mortal = w.write::<Mortal>();
	if let Some(&mut Mortal(ref mut mo)) = mortal.get_mut(src) {
		*mo += amt
	}
}

fn blink(src: Entity, rng: &mut R, w: &mut World) {
	let np = if let Some(pxy) = w.write_resource::<Possy>().get_pos(src) {
		[rng.gen_range(0, 40), rng.gen_range(0, 40), pxy[2]]
	} else {
		return
	};
	moveto(np, src, rng, w);
}

fn grab(xyz: [i16; 3], src: Entity, _rng: &mut R, w: &mut World) {
	let strength = w.read::<Strength>();
	let mut bag = w.write::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) = (strength.get(src), bag.get_mut(src)) {
		let weight = w.read::<Weight>();
		let mut possy = w.write_resource::<Possy>();
		let mut totwei: i32 = ebag.iter().filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32)).sum();
		let mut rmpos = Vec::new();
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

fn pickup(src: Entity, ent: Entity, _rng: &mut R, w: &mut World) {
	let strength = w.read::<Strength>();
	let mut bag = w.write::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) = (strength.get(src), bag.get_mut(src)) {
		let weight = w.read::<Weight>();
		let mut possy = w.write_resource::<Possy>();
		let mut totwei: i32 = ebag.iter().filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32)).sum();
		if let Some(&Weight(wei)) = weight.get(ent) {
			if totwei + wei as i32 <= strg as i32 {
				ebag.push(ent);
				possy.remove(ent);
				totwei += wei as i32;
			}
		}
	}
}
