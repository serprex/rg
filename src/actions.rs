use rand::Rng;
use specs::*;

use components::*;
use position::Possy;
use tick::Ticker;
use util::*;

#[allow(dead_code)]
pub enum Action {
	Action(Box<Fn(&mut World) + Send + Sync>),
	Ai {
		src: Entity,
	},
	Movedir {
		dir: Dir,
		src: Entity,
	},
	Colcheck {
		src: Entity,
	},
	Moveto {
		np: [i16; 3],
		src: Entity,
	},
	Melee {
		dur: u8,
		src: Entity,
		ent: Entity,
	},
	Missile {
		spd: u32,
		dir: Dir,
		dur: u8,
		ent: Entity,
	},
	Attack {
		dir: Dir,
		src: Entity,
	},
	Lunge {
		dir: Dir,
		src: Entity,
	},
	Shoot {
		dir: Dir,
		src: Entity,
	},
	Throw {
		dir: Dir,
		psrc: Entity,
		tsrc: Entity,
		obj: Entity,
	},
	Heal {
		src: Entity,
		amt: i16,
	},
	Blink {
		src: Entity,
	},
	Grab {
		xyz: [i16; 3],
		src: Entity,
	},
	Pickup {
		src: Entity,
		ent: Entity,
	},
	Render {
		src: Entity,
	},
	Say(String),
	Seek(u32, Entity),
	PossyGc,
}

impl Action {
	pub fn call(self, w: &mut World) {
		match self {
			Action::Action(a) => a(w),
			Action::Ai { src } => super::ailogic::ailogic(src, w),
			Action::Movedir { dir, src } => movedir(dir, src, w),
			Action::Colcheck { src } => colcheck(src, w),
			Action::Moveto { np, src } => moveto(np, src, w),
			Action::Melee { dur, src, ent } => melee(dur, src, ent, w),
			Action::Missile { spd, dir, dur, ent } => missile(spd, dir, dur, ent, w),
			Action::Attack { dir, src } => attack(dir, src, w),
			Action::Lunge { dir, src } => lunge(dir, src, w),
			Action::Shoot { dir, src } => shoot(dir, src, w),
			Action::Throw {
				dir,
				psrc,
				tsrc,
				obj,
			} => throw(dir, psrc, tsrc, obj, w),
			Action::Heal { src, amt } => heal(src, amt, w),
			Action::Blink { src } => blink(src, w),
			Action::Grab { xyz, src } => grab(xyz, src, w),
			Action::Pickup { src, ent } => pickup(src, ent, w),
			Action::Render { src } => super::render::render(src, w),
			Action::Say(s) => say(s, w),
			Action::Seek(n, src) => seek(n, src, w),
			Action::PossyGc => possygc(w),
		}
	}
}

fn seek(n: u32, src: Entity, w: &mut World) {
	let (seeking, act) = {
		let mut cseek = w.write_storage::<Seeking>();
		let possy = w.read_resource::<Possy>();
		let seeking = if let Some(&Seeking(ref seeking, _)) = cseek.get(src) {
			match seeking {
				&Seek::Entity(seekent) => if let Some(pos) = possy.get_pos(src) {
					if let Some(objpos) = possy.get_pos(seekent) {
						if objpos == [pos[0], pos[1] + 1, pos[2]] {
							Some(seekent)
						} else {
							None
						}
					} else {
						None
					}
				} else {
					None
				},
				&Seek::Race(race) => if let Some(pos) = possy.get_pos(src) {
					let ents = possy.get_ents([pos[0], pos[1] + 1, pos[2]]);
					let crace = w.read_storage::<Race>();
					let mut seeking = None;
					for &e in ents {
						match crace.get(e) {
							Some(&r) if r == race => {
								seeking = Some(e);
								break;
							}
							_ => (),
						}
					}
					seeking
				} else {
					None
				},
			}
		} else {
			return;
		};
		if let Some(seeking) = seeking {
			if let Some(Seeking(_, act)) = cseek.remove(src) {
				(seeking, act)
			} else {
				return;
			}
		} else {
			let mut ticker = w.write_resource::<Ticker>();
			ticker.push(n, Action::Seek(n, src));
			return;
		}
	};
	pickup(src, seeking, w);
	act.call(w);
}

fn say(s: String, w: &mut World) {
	let mut log = w.write_resource::<Log>();
	log.push(s);
}

fn possygc(w: &mut World) {
	let mut possy = w.write_resource::<Possy>();
	possy.gc(&w);
	let mut ticker = w.write_resource::<Ticker>();
	ticker.push(10, Action::PossyGc);
}

fn movedir(dir: Dir, src: Entity, w: &mut World) {
	if let Some(np) = {
		let possy = w.read_resource::<Possy>();
		possy.get_pos(src).map(|pos| xyz_plus_dir(pos, dir))
	} {
		moveto(np, src, w)
	}
}

fn colcheck(src: Entity, w: &mut World) {
	if let Some(np) = {
		let possy = w.read_resource::<Possy>();
		possy.get_pos(src)
	} {
		moveto(np, src, w)
	}
}

fn moveto(np: [i16; 3], src: Entity, w: &mut World) {
	let mut possy = w.write_resource::<Possy>();
	let mut mort = w.write_storage::<Mortal>();
	let portal = w.read_storage::<Portal>();
	let fragile = w.read_storage::<Fragile>();
	let mut solid = w.write_storage::<Solid>();
	let Walls(ref walls) = *w.read_resource::<Walls>();
	if walls.contains_key(&np) {
		if fragile.get(src).is_some() {
			w.entities().delete(src);
		}
		return;
	}
	if solid.get(src).is_some() {
		for &e in possy.get_ents(np).iter() {
			if solid.get(e).is_some() {
				return;
			}
		}
	}
	possy.set_pos(src, np);
	let mut ai = w.write_storage::<Ai>();
	let mut misl = w.write_storage::<Dmg>();
	let ents = w.entities();
	let mut spos = Vec::new();
	for ce in possy.get_ents(np).iter().cloned().filter(|&ce| ce != src) {
		if let Some(&Portal(porx)) = portal.get(ce) {
			spos.push((src, porx));
		}
		if let Some(&Dmg(dmg)) = misl.get(src) {
			if solid.get(ce).is_some() {
				if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
					if *mce <= dmg {
						*mce = 0;
						solid.remove(ce);
						ai.remove(ce);
					} else {
						*mce -= dmg;
					}
				}
				if fragile.get(src).is_some() {
					ents.delete(src);
				} else {
					// TODO enable persisting attack
					misl.remove(src);
				}
			}
		}
	}
	for (e, p) in spos {
		possy.set_pos(e, p);
	}
}

fn melee(dur: u8, src: Entity, ent: Entity, w: &mut World) {
	if dur == 0 {
		let mut weapons = w.write_storage::<Weapon>();
		let mut possy = w.write_resource::<Possy>();
		weapons.insert(src, Weapon(ent));
		possy.remove(ent);
	} else {
		{
			let mut ticker = w.write_resource::<Ticker>();
			ticker.push(
				1,
				Action::Melee {
					dur: dur - 1,
					src: src,
					ent: ent,
				},
			);
		}
		colcheck(ent, w);
	}
}

fn missile(spd: u32, dir: Dir, dur: u8, ent: Entity, w: &mut World) {
	if dur == 0 {
		let mut cm = w.write_storage::<Dmg>();
		cm.remove(ent);
	} else {
		{
			let mut ticker = w.write_resource::<Ticker>();
			ticker.push(
				spd,
				Action::Missile {
					spd: spd,
					dir: dir,
					dur: dur - 1,
					ent: ent,
				},
			);
		}
		movedir(dir, ent, w);
	}
}

fn attack(dir: Dir, src: Entity, w: &mut World) {
	let (bp, went) = {
		let mut weapons = w.write_storage::<Weapon>();
		let cpos = w.read_resource::<Possy>();
		let watk = w.read_storage::<Atk<Weapon>>();
		let mut misl = w.write_storage::<Dmg>();
		if let Some(Weapon(went)) = weapons.remove(src) {
			if let Some(pos) = cpos.get_pos(src) {
				let cstr = w.read_storage::<Strength>();
				let &Strength(srcstr) = cstr.get(went).unwrap_or(&Strength(1));
				let bp = xyz_plus_dir(pos, dir);
				let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 1, 0));
				let mut ticker = w.write_resource::<Ticker>();
				misl.insert(went, Dmg(srcstr / 4 + wstats.dmg));
				let dur = wstats.dur;
				ticker.push(
					1,
					Action::Melee {
						dur: dur,
						src: src,
						ent: went,
					},
				);
				/*
				if wstats.spd != 0 {
					let mut cai = w.write_storage::<Ai>();
					if let Some(mut ai) = cai.get_mut(src) {
						ai.tick = if wstats.spd < 0 {
							let spd = (-wstats.spd) as u8;
							if spd < ai.tick { ai.tick - spd } else { 0 }
						} else {
							ai.tick + wstats.spd as u8
						};
					}
				}*/
				(bp, went)
			} else {
				return;
			}
		} else {
			return;
		}
	};
	moveto(bp, went, w)
}

fn lunge(dir: Dir, src: Entity, w: &mut World) {
	movedir(dir, src, w);
	attack(dir, src, w);
}

fn shoot(dir: Dir, src: Entity, w: &mut World) {
	let (went, sent) = {
		let weapons = w.read_storage::<Weapon>();
		let mut shields = w.write_storage::<Shield>();
		if let Some(&Weapon(went)) = weapons.get(src) {
			if let Some(Shield(sent)) = shields.remove(src) {
				(went, sent)
			} else {
				return;
			}
		} else {
			return;
		}
	};
	throw(dir, src, went, sent, w)
}

fn throw(dir: Dir, psrc: Entity, tsrc: Entity, obj: Entity, w: &mut World) {
	let bp = {
		let possy = w.read_resource::<Possy>();
		if let Some(pos) = possy.get_pos(psrc) {
			let cstr = w.read_storage::<Strength>();
			let cwei = w.read_storage::<Weight>();
			let mut ticker = w.write_resource::<Ticker>();
			let mut misl = w.write_storage::<Dmg>();
			let bp = xyz_plus_dir(pos, dir);
			let &Strength(srcstr) = cstr.get(tsrc).unwrap_or(&Strength(1));
			let &Weight(objwei) = cwei.get(obj).unwrap_or(&Weight(1));
			let dmg = srcstr as i16 + objwei as i16 / 2;
			let spd = 1 + (objwei as i16 * 8 / srcstr as i16) as u32;
			misl.insert(obj, Dmg(dmg));
			ticker.push(
				spd,
				Action::Missile {
					spd: spd,
					dir: dir,
					dur: (108 / spd) as u8,
					ent: obj,
				},
			);
			bp
		} else {
			return;
		}
	};
	moveto(bp, obj, w)
}

fn heal(src: Entity, amt: i16, w: &mut World) {
	let mut mortal = w.write_storage::<Mortal>();
	if let Some(&mut Mortal(ref mut mo)) = mortal.get_mut(src) {
		*mo += amt
	}
}

fn blink(src: Entity, w: &mut World) {
	let pz = if let Some(pxy) = w.write_resource::<Possy>().get_pos(src) {
		pxy[2]
	} else {
		return;
	};
	let np = {
		let mut rng = w.write_resource::<R>();
		[rng.gen_range(0, 40), rng.gen_range(0, 40), pz]
	};
	moveto(np, src, w);
}

fn grab(xyz: [i16; 3], src: Entity, w: &mut World) {
	let strength = w.read_storage::<Strength>();
	let mut bag = w.write_storage::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) =
		(strength.get(src), bag.get_mut(src))
	{
		let weight = w.read_storage::<Weight>();
		let mut possy = w.write_resource::<Possy>();
		let mut totwei: i32 = ebag
			.iter()
			.filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32))
			.sum();
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

fn pickup(src: Entity, ent: Entity, w: &mut World) {
	let strength = w.read_storage::<Strength>();
	let mut bag = w.write_storage::<Bag>();
	if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) =
		(strength.get(src), bag.get_mut(src))
	{
		let weight = w.read_storage::<Weight>();
		let mut possy = w.write_resource::<Possy>();
		let totwei: i32 = ebag
			.iter()
			.filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32))
			.sum();
		if let Some(&Weight(wei)) = weight.get(ent) {
			if totwei + wei as i32 <= strg as i32 {
				ebag.push(ent);
				possy.remove(ent);
			}
		}
	}
}
