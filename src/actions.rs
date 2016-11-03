use std::mem;

use rand::Rng;
use specs::*;
use smallvec::SmallVec;

use components::*;
use position::Possy;
use tick::Ticker;
use util::*;

#[allow(dead_code)]
pub enum Action {
	Action(Box<Fn(&mut R, &mut World) + Send + Sync>),
	Ai { src: Entity },
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
	Render { src: Entity },
}

impl Action {
	pub fn call(self, rng: &mut R, w: &mut World) {
		match self {
			Action::Action(a) => a(rng, w),
			Action::Ai { src } => aievent(src, rng, w),
			Action::Movedir { dir, src } => movedir(dir, src, rng, w),
			Action::Colcheck { src } => colcheck(src, rng, w),
			Action::Moveto { np, src } => moveto(np, src, rng, w),
			Action::Melee { dur, src, ent } => melee(dur, src, ent, rng, w),
			Action::Missile { spd, dir, dur, ent} => missile(spd, dir, dur, ent, rng, w),
			Action::Attack { dir, src } => attack(dir, src, rng, w),
			Action::Lunge { dir, src } => lunge(dir, src, rng, w),
			Action::Shoot { dir, src } => shoot(dir, src, rng, w),
			Action::Throw { dir, psrc, tsrc, obj } => throw(dir, psrc, tsrc, obj, rng, w),
			Action::Heal { src, amt } => heal(src, amt, rng, w),
			Action::Blink { src } => blink(src, rng, w),
			Action::Grab { xyz, src } => grab(xyz, src, rng, w),
			Action::Pickup { src, ent } => pickup(src, ent, rng, w),
			Action::Render { src } => super::render::render(src, rng, w),
		}
	}
}

fn aievent(ent: Entity, rng: &mut R, w: &mut World) {
	let mut cai = w.write::<Ai>();
	if let Some(mut ai) = cai.get_mut(ent) {
		let mut tick = ai.speed;
		let possy = w.read_resource::<Possy>();
		let ref mut ticker = *w.write_resource::<Ticker>();
		if let Some(pos) = possy.get_pos(ent) {
			match ai.state {
				AiState::PlayerInventory(invp) => {
					tick = 1;
					let mut bag = w.write::<Bag>();
					let mut weapons = w.write::<Weapon>();
					let mut shields = w.write::<Shield>();
					let mut armors = w.write::<Armor>();
					if let Some(&mut Bag(ref mut ebag)) = bag.get_mut(ent) {
						'invput: loop {
							match (getch(), ebag.is_empty()) {
								('i', _) =>
									ai.state = AiState::Player,
								('j', false) =>
									ai.state = AiState::PlayerInventory(if invp == ebag.len()-1 { 0 } else { invp + 1 }),
								('k', false) =>
									ai.state = AiState::PlayerInventory(if invp == 0 { ebag.len()-1 } else { invp - 1 }),
								('d', false) => {
									let drop = ebag.remove(invp);
									ticker.push(0, Action::Moveto { np: pos, src: drop });
									if invp == ebag.len() {
										ai.state = AiState::PlayerInventory(0);
									}
								},
								('w', false) => {
									if let InsertResult::Updated(Weapon(oldw)) = weapons.insert(ent, Weapon(ebag.remove(invp))) {
										ebag.push(oldw);
									} else if invp == ebag.len() {
										ai.state = AiState::PlayerInventory(0);
									}
								},
								('W', _) => {
									if let Some(Weapon(went)) = weapons.remove(ent) {
										ebag.push(went);
									}
								},
								('s', false) => {
									if let InsertResult::Updated(Shield(oldw)) = shields.insert(ent, Shield(ebag.remove(invp))) {
										ebag.push(oldw);
									} else if invp == ebag.len() {
										ai.state = AiState::PlayerInventory(0);
									}
								},
								('S', _) => {
									if let Some(Shield(went)) = shields.remove(ent) {
										ebag.push(went);
									}
								},
								('a', false) => {
									if let InsertResult::Updated(Armor(oldw)) = armors.insert(ent, Armor(ebag.remove(invp))) {
										ebag.push(oldw);
									} else if invp == ebag.len() {
										ai.state = AiState::PlayerInventory(0);
									}
								},
								('A', _) => {
									if let Some(Armor(went)) = armors.remove(ent) {
										ebag.push(went);
									}
								},
								('t', false) => {
									if let Ok(d) = char_as_dir(getch()) {
										let went = ebag.remove(invp);
										ticker.push(0, Action::Throw{ dir: d, psrc: ent, tsrc: ent, obj: went });
										ai.state = AiState::Player;
									} else {
										continue 'invput
									}
								},
								(c, _) if c >= '0' && c <= '9' => {
									let v = (c as u32 as u8 - b'0') as usize;
									if v < ebag.len() {
										ai.state = AiState::PlayerInventory(v);
									}
								},
								('\x1b', _) => (),
								_ => continue 'invput,
							};
							break
						}
					} else {
						ai.state = AiState::Player;
					}
				},
				ref mut casting @ AiState::PlayerCasting(_) => {
					let ch = getch();
					if ch == ';' || ch == '\x1b' {
						*casting = AiState::Player;
					} else {
						loop {
							if let AiState::PlayerCasting(ref mut cast) = *casting {
								cast.push(ch);
								if cast == "blink" {
									ticker.push(0, Action::Blink{ src: ent });
								} else {
									break
								}
							}
							*casting = AiState::Player;
							break
						}
					}
				},
				AiState::Player => 'playerinput: loop {
					match char_as_dir(getch()) {
						Ok(d) => {
							ticker.push(0, Action::Movedir { dir: d, src: ent });
						},
						Err('p') => {
							match char_as_dir(getch()) {
								Ok(d) => {
									let gp = xyz_plus_dir(pos, d);
									ticker.push(0, Action::Grab { xyz: gp, src: ent });
								},
								_ => continue 'playerinput,
							}
						},
						Err('i') => {
							ai.state = AiState::PlayerInventory(0);
						},
						Err('a') => {
							match char_as_dir(getch()) {
								Ok(d) => ticker.push(0, Action::Attack { dir: d, src: ent }),
								_ => continue 'playerinput,
							}
						},
						Err('q') => {
							match char_as_dir(getch()) {
								Ok(d) => ticker.push(0, Action::Lunge { dir: d, src: ent }),
								_ => continue 'playerinput,
							}
						},
						Err('s') => {
							match char_as_dir(getch()) {
								Ok(d) => ticker.push(0, Action::Shoot{ dir: d, src: ent }),
								_ => continue 'playerinput,
							}
						},
						Err('d') => {
							ai.state = AiState::PlayerCasting(String::new());
						},
						Err(c) if c >= '0' && c <= '9' => tick = (c as u32 as u8 - b'0') as u32,
						_ => (),
					}
					break
				},
				AiState::Random => {
					let mut choices: [Option<Dir>; 6] = unsafe { mem::uninitialized() };
					choices[0] = None;
					choices[1] = None;
					let mut chs = 2;
					for &dir in &[Dir::L, Dir::H, Dir::J, Dir::K] {
						let choice = xyz_plus_dir(pos, dir);
						if !possy.contains(choice) {
							choices[chs] = Some(dir);
							chs += 1;
						}
					}
					if let Some(&Some(dir)) = rng.choose(&choices[..chs]) {
						ticker.push(0, Action::Movedir { dir: dir, src: ent });
					}
					let near = possy.get_within(pos, 5);
					let crace = w.read::<Race>();
					if let Some(&race) = crace.get(ent) {
						for (e2, _) in near {
							if ent != e2 {
								if let Some(&race2) = crace.get(e2) {
									if is_aggro(race, race2) {
										ai.state = AiState::Aggro(e2)
									}
								}
							}
						}
					}
				},
				AiState::Scared(foe) => {
					match possy.get_pos(foe) {
						None => ai.state = AiState::Random,
						Some(fxy) => {
							let mut choices: [Dir; 4] = unsafe { mem::uninitialized() };
							let mut chs = 0;
							let dist = (pos[0] - fxy[0]).abs() + (pos[1] - fxy[1]).abs();
							for &dir in &[Dir::L, Dir::H, Dir::J, Dir::K] {
								let choice = xyz_plus_dir(pos, dir);
								if (pos[0] - fxy[0]).abs() + (pos[1] - fxy[1]).abs() > dist && !possy.contains(choice) {
									choices[chs] = dir;
									chs += 1;
								}
							}
							if chs == 0 {
								ai.state = AiState::Aggro(foe)
							} else {
								let dir = *rng.choose(&choices[..chs]).unwrap();
								ticker.push(0, Action::Movedir { dir: dir, src: ent });
							}
						}
					}
				},
				AiState::Aggro(foe) => {
					match possy.get_pos(foe) {
						None => ai.state = AiState::Random,
						Some(fxy) => {
							let crace = w.read::<Race>();
							match crace.get(ent) {
								Some(&Race::Leylapan) => {
									let mut dirs: [Dir; 2] = unsafe { mem::uninitialized() };
									let mut dnum = 0;
									if pos[0] != fxy[0] {
										dirs[0] = if pos[0] < fxy[0] {
											Dir::L
										} else {
											Dir::H
										};
										dnum = 1
									}
									if pos[1] != fxy[1] {
										dirs[dnum] = if pos[1] < fxy[1] {
											Dir::J
										} else {
											Dir::K
										};
										dnum += 1
									}
									if let Some(&fdir) = rng.choose(&dirs[..dnum]) {
										let mut weight = w.write::<Weight>();
										let mut fragile = w.write::<Fragile>();
										let mut cch = w.write::<Chr>();
										let shot = w.create_later();
										weight.insert(shot, Weight(2));
										fragile.insert(shot, Fragile);
										cch.insert(shot, Chr(Char::from('j')));
										ticker.push(0, Action::Throw { dir: fdir, psrc: ent, tsrc: ent, obj: shot });
										let mdir = if dnum == 2 {
											dirs[if dirs[0] == fdir { 1 } else { 0 }]
										} else {
											match fdir {
												Dir::L => Dir::H,
												Dir::H => Dir::L,
												Dir::J => Dir::K,
												Dir::K => Dir::J,
											}
										};
										let nxy = xyz_plus_dir(pos, mdir);
										if possy.contains(nxy) {
											ai.state = AiState::Scared(foe)
										} else {
											ticker.push(0, Action::Movedir { dir: mdir, src: ent });
										}
									} else {
										ai.state = AiState::Scared(foe)
									}
								},
								_ => {
									let mut xxyy = pos;
									let mut attacking = false;
									for &dir in &[Dir::L, Dir::H, Dir::J, Dir::K] {
										if xyz_plus_dir(pos, dir) == fxy {
											ticker.push(0, Action::Attack { dir: dir, src: ent });
											attacking = true;
											break
										}
									}
									if !attacking {
										let mut tries = 3;
										loop {
											let mut xy = xxyy;
											let co = if tries == 1 || (tries == 3 && rng.gen()) { 0 } else { 1 };
											xy[co] += cmpi(xy[co], fxy[co], 1, 0, -1);
											if xy == xxyy || (xy != fxy && possy.contains(xy)) {
												tries -= 1;
												if tries == 0 { break }
											} else {
												xxyy = xy;
												if xy == fxy {
													break
												} else {
													tries = 3
												}
											}
										}
										if xxyy == fxy {
											let co = if pos[0] != fxy[0] && rng.gen() { 0 } else { 1 };
											if let Some(dir) = match (co, cmpi(pos[co], fxy[co], 1, 0, -1)) {
												(0, -1) => Some(Dir::H),
												(0, 1) => Some(Dir::L),
												(1, -1) => Some(Dir::K),
												(1, 1) => Some(Dir::J),
												 _ => None,
											} {
												ticker.push(0, Action::Movedir { dir: dir, src: ent });
											}
										} else {
											ai.state = AiState::Random
										}
									}
								}
							}
						}
					}
				},
				//_ => (),
			}
			ticker.push(tick, Action::Ai { src: ent });
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
		let cpos = w.read_resource::<Possy>();
		let watk = w.read::<Atk<Weapon>>();
		let mut misl = w.write::<Dmg>();
		if let Some(Weapon(went)) = weapons.remove(src) {
			if let Some(pos) = cpos.get_pos(src) {
				let cstr = w.read::<Strength>();
				let &Strength(srcstr) = cstr.get(went).unwrap_or(&Strength(1));
				let bp = xyz_plus_dir(pos, dir);
				let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 1, 0));
				let mut ticker = w.write_resource::<Ticker>();
				misl.insert(went, Dmg(srcstr / 4 + wstats.dmg));
				let dur = wstats.dur;
				ticker.push(1, Action::Melee { dur: dur, src: src, ent: went });
				/*let mut cai = w.write::<Ai>();
				if wstats.spd != 0 {
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
		let totwei: i32 = ebag.iter().filter_map(|&e| weight.get(e).map(|&Weight(x)| x as i32)).sum();
		if let Some(&Weight(wei)) = weight.get(ent) {
			if totwei + wei as i32 <= strg as i32 {
				ebag.push(ent);
				possy.remove(ent);
			}
		}
	}
}
