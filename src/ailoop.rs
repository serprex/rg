use std::mem;
use rand::{self, Rng};
use specs::*;
use components::*;
use util::*;
use actions;

pub fn ailoop(w: &mut World) {
	let (mut stasis, mut inventory, mut cpos, mut cnpos, mut cai, mut crace, mut cch, mut bag, mut weight, mut strength, mut weapons, mut casting, ents) =
		(w.write::<AiStasis>(), w.write::<Inventory>(), w.write::<Pos>(), w.write::<NPos>(), w.write::<Ai>(), w.write::<Race>(), w.write::<Chr>(), w.write::<Bag>(), w.write::<Weight>(), w.write::<Strength>(), w.write::<Weapon>(), w.write::<Casting>(), w.entities());
	let collisions: FnvHashSet<[i16; 3]> = cpos.iter().map(|pos| pos.0).collect();
	let mut rng = rand::thread_rng();
	let mut newent: Vec<(Entity, Chr, Ai, Pos)> = Vec::new();
	let mut grab: Vec<(Entity, [i16; 3])> = Vec::new();
	let Todo(ref mut todos) = *w.write_resource::<Todo>();
	for (&Pos(pos), mut ai, &race, ent) in (&cpos, &mut cai, &crace, &ents).iter() {
		if stasis.get(ent).is_none() && ai.tick == 0 {
			let mut npos = pos;
			ai.tick = ai.speed;
			match ai.state {
				AiState::Player => 'playerinput: loop {
					let ch = getch();
					let mut rmcast = false;
					let mut wassomecast = false; // Lexical lifetimes I condemn you
					if let Some(&mut Casting(ref mut cast)) = casting.get_mut(ent) {
						wassomecast = true;
						if ch == ';' || ch == '\x1b' {
							rmcast = true;
						} else {
							cast.push(ch);
							if cast == "blink" {
								todos.push((ent, Box::new(actions::blink)));
								rmcast = true;
							}
						}
					}
					if rmcast { casting.remove(ent); }
					if !wassomecast {
						match char_as_dir(ch) {
							Ok(d) => xy_incr_dir(&mut npos, d),
							Err('p') => {
								match char_as_dir(getch()) {
									Ok(d) => grab.push((ent, xyz_plus_dir(pos, d))),
									_ => continue 'playerinput,
								}
							},
							Err('i') => {
								let newinv = w.create_later();
								inventory.insert(newinv, Inventory(ent, 99999));
								stasis.insert(newinv, AiStasis(ent));
							},
							Err('a') => {
								match char_as_dir(getch()) {
									Ok(d) => {
										let mut wdir = w.write::<WDirection>();
										wdir.insert(ent, WDirection(d));
										todos.push((ent, Box::new(actions::attack)));
									},
									_ => continue 'playerinput,
								}
							},
							Err('s') => {
								match char_as_dir(getch()) {
									Ok(d) => {
										let mut wdir = w.write::<WDirection>();
										wdir.insert(ent, WDirection(d));
										todos.push((ent, Box::new(actions::shoot)));
									},
									_ => continue 'playerinput,
								}
							},
							Err('d') => {
								casting.insert(ent, Casting(String::new()));
							},
							Err(c) if c >= '0' && c <= '9' => ai.tick = c as u32 as u8 - b'0',
							_ => (),
						}
					}
					break
				},
				AiState::Random => {
					let mut choices: [[i16; 3]; 6] = unsafe { mem::uninitialized() };
					choices[0] = pos;
					choices[1] = pos;
					let mut chs = 2;
					for choice in &[[pos[0]-1, pos[1], pos[2]],
					[pos[0]+1, pos[1], pos[2]],
					[pos[0], pos[1]-1, pos[2]],
					[pos[0], pos[1]+1, pos[2]]] {
						if !collisions.contains(choice) {
							choices[chs] = *choice;
							chs += 1;
						}
					}
					npos = *rng.choose(&choices[..chs]).unwrap();
					for (&Pos(pos2), &race2, e2) in (&cpos, &crace, &ents).iter() {
						if ent != e2 && is_aggro(race, race2) &&
							(npos[0] - pos2[0]).abs() < 5 &&
							(npos[1] - pos2[1]).abs() < 5 {
							ai.state = AiState::Aggro(e2)
						}
					}
				},
				AiState::Scared(foe) => {
					match cpos.get(foe) {
						None => ai.state = AiState::Random,
						Some(fxy) => {
							let fxy = fxy.0;
							let mut choices: [[i16; 3]; 4] = unsafe { mem::uninitialized() };
							let mut chs = 0;
							let dist = (pos[0] - fxy[0]).abs() + (pos[1] - fxy[1]).abs();
							for choice in &[[pos[0]-1, pos[1], pos[2]],
							[pos[0]+1, pos[1], pos[2]],
							[pos[0], pos[1]-1, pos[2]],
							[pos[0], pos[1]+1, pos[2]]] {
								if (pos[0] - fxy[0]).abs() + (pos[1] - fxy[1]).abs() > dist && !collisions.contains(choice) {
									choices[chs] = *choice;
									chs += 1;
								}
							}
							if chs == 0 {
								ai.state = AiState::Aggro(foe)
							} else {
								npos = *rng.choose(&choices[..chs]).unwrap()
							}
						}
					}
				},
				AiState::Aggro(foe) => {
					match cpos.get(foe) {
						None => ai.state = AiState::Random,
						Some(fxy) => {
							let fxy = fxy.0;
							match race {
								Race::Leylapan => {
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
									if dnum > 0 {
										let fdir = *rng.choose(&dirs[..dnum]).unwrap();
										let bp = xyz_plus_dir(pos, fdir);
										newent.push((w.create_later(), Chr(Char::from('j')), Ai::new(AiState::Missile(fdir, 1), 2), Pos(bp)));
										let mdir = if dnum == 2 {
											if dirs[0] == fdir { dirs[1] }
											else { dirs[0] }
										} else {
											match fdir {
												Dir::L => Dir::H,
												Dir::H => Dir::L,
												Dir::J => Dir::K,
												Dir::K => Dir::J,
											}
										};
										let nxy = xyz_plus_dir(pos, mdir);
										if collisions.contains(&nxy) {
											ai.state = AiState::Scared(foe)
										} else {
											npos = nxy
										}
									} else {
										ai.state = AiState::Scared(foe)
									}
								},
								_ => {
									let mut xxyy = pos;
									let mut attacking = false;
									for &choice in &[[pos[0]-1, pos[1], pos[2]],
									[pos[0]+1, pos[1], pos[2]],
									[pos[0], pos[1]-1, pos[2]],
									[pos[0], pos[1]+1, pos[2]]] {
										if choice == fxy {
											newent.push((w.create_later(), Chr(Char::from('x')), Ai::new(AiState::Melee(2, 2), 1), Pos(choice)));
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
											if xy == xxyy || (xy != fxy && collisions.contains(&xy)) {
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
											npos[co] += cmpi(pos[co], fxy[co], 1, 0, -1);
										} else {
											ai.state = AiState::Random
										}
									}
								}
							}
						}
					}
				},
				AiState::Missile(dir, _) => {
					xy_incr_dir(&mut npos, dir);
				},
				AiState::Melee(ref mut dur, _) => {
					if *dur == 0 {
						w.delete_later(ent)
					} else {
						*dur -= 1
					}
				},
				//_ => (),
			}
			if npos != pos {
				cnpos.insert(ent, NPos(npos));
			}
		} else {
			ai.tick -= 1
		}
	}
	let mut rminv = Vec::new();
	for (&mut Inventory(inve, ref mut invp), ent) in (&mut inventory, &ents).iter() {
		if *invp == 99999 {
			*invp = 0;
			continue
		}
		if let Some(&mut Bag(ref mut ebag)) = bag.get_mut(inve) {
			'invput: loop {
				match getch() {
					'i' => rminv.push(ent),
					'j' if ebag.len() > 0 =>
						*invp = if *invp == ebag.len()-1 { 0 } else { *invp + 1 },
					'k' if ebag.len() > 0 =>
						*invp = if *invp == 0 { ebag.len()-1 } else { *invp - 1 },
					'w' => {
						match weapons.insert(inve, Weapon(ebag.remove(*invp))) {
							InsertResult::Updated(Weapon(oldw)) => ebag.push(oldw),
							_ => (),
						}
						if *invp == ebag.len() {
							*invp = 0
						}
					},
					'W' => {
						weapons.remove(inve);
					},
					c if c >= '0' && c <= '9' => {
						let v = (c as u32 as u8 - b'0') as usize;
						if v < ebag.len() {
							*invp = v
						}
					},
					'\x1b' => (),
					_ => continue 'invput,
				};
				break
			}
		} else {
			rminv.push(ent);
		}
	}
	for ent in rminv {
		if let Some(inv) = inventory.get(ent) {
			stasis.remove(inv.0);
		}
		inventory.remove(ent);
	}
	let mut rmpos = Vec::new();
	for (ent, xyz) in grab {
		if let (Some(&Strength(strg)), Some(&mut Bag(ref mut ebag))) = (strength.get(ent), bag.get_mut(ent)) {
			let mut totwei = 0;
			for &e in ebag.iter() {
				if let Some(&Weight(wei)) = weight.get(e) {
					totwei += wei as i32;
				}
			}
			for (&Pos(pos), &Weight(wei), ent) in (&cpos, &weight, &ents).iter() {
				if pos == xyz && totwei + wei as i32 <= strg as i32 {
					ebag.push(ent);
					rmpos.push(ent);
				}
			}
		}
	}
	for ent in rmpos {
		cpos.remove(ent);
	}
	for (ent, newch, newai, newpos) in newent {
		cch.insert(ent, newch);
		cai.insert(ent, newai);
		cpos.insert(ent, newpos);
		crace.insert(ent, Race::None);
	}
}
