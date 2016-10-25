use std::mem;
use rand::Rng;
use specs::*;
use components::*;
use util::*;
use actions;
use position::Possy;

pub fn ailoop(rng: &mut R, w: &mut World) {
	let crace = w.read::<Race>();
	let mut cai = w.write::<Ai>();
	let possy = w.read_resource::<Possy>();
	let ents = w.entities();
	let Todo(ref mut todos) = *w.write_resource::<Todo>();
	let mut rmai = Vec::new();
	for (mut ai, ent) in (&mut cai, &ents).iter() {
		if ai.tick == 0 {
			if let Some(pos) = possy.get_pos(ent) {
				ai.tick = ai.speed;
				match ai.state {
					AiState::PlayerInventory(invp) => {
						ai.tick = 1;
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
										todos.push(Box::new(move|r, w| actions::moveto(pos, drop, r, w)));
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
											todos.push(Box::new(move|r, w| actions::throw(d, ent, ent, went, r, w)));
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
							if let AiState::PlayerCasting(ref mut cast) = *casting {
								cast.push(ch);
								if cast == "blink" {
									todos.push(Box::new(move|r, w| actions::blink(ent, r, w)));
								} else {
									continue;
								}
							}
							*casting = AiState::Player;
						}
					},
					AiState::Player => 'playerinput: loop {
						match char_as_dir(getch()) {
							Ok(d) => {
								todos.push(Box::new(move|r, w| actions::movedir(d, ent, r, w)));
							},
							Err('p') => {
								match char_as_dir(getch()) {
									Ok(d) => {
										let gp = xyz_plus_dir(pos, d);
										todos.push(Box::new(move|r, w| actions::grab(gp, ent, r, w)));
									},
									_ => continue 'playerinput,
								}
							},
							Err('i') => {
								ai.state = AiState::PlayerInventory(0);
							},
							Err('a') => {
								match char_as_dir(getch()) {
									Ok(d) => todos.push(Box::new(move|r, w| actions::attack(d, ent, r, w))),
									_ => continue 'playerinput,
								}
							},
							Err('q') => {
								match char_as_dir(getch()) {
									Ok(d) => todos.push(Box::new(move|r, w| actions::lunge(d, ent, r, w))),
									_ => continue 'playerinput,
								}
							},
							Err('s') => {
								match char_as_dir(getch()) {
									Ok(d) => todos.push(Box::new(move|r, w| actions::shoot(d, ent, r, w))),
									_ => continue 'playerinput,
								}
							},
							Err('d') => {
								ai.state = AiState::PlayerCasting(String::new());
							},
							Err(c) if c >= '0' && c <= '9' => ai.tick = c as u32 as u8 - b'0',
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
							todos.push(Box::new(move|r, w| actions::movedir(dir, ent, r, w)));
						}
						let near = possy.get_within(pos, 5);
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
									todos.push(Box::new(move|r, w| actions::movedir(dir, ent, r, w)));
								}
							}
						}
					},
					AiState::Aggro(foe) => {
						match possy.get_pos(foe) {
							None => ai.state = AiState::Random,
							Some(fxy) => {
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
											let mut shields = w.write::<Shield>();
											let mut weight = w.write::<Weight>();
											let mut fragile = w.write::<Fragile>();
											let mut cch = w.write::<Chr>();
											let shot = w.create_later();
											weight.insert(shot, Weight(1));
											fragile.insert(shot, Fragile);
											cch.insert(shot, Chr(Char::from('j')));
											todos.push(Box::new(move|r, w| actions::throw(fdir, ent, ent, shot, r, w)));
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
												todos.push(Box::new(move|r, w| actions::movedir(mdir, ent, r, w)));
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
												todos.push(Box::new(move|r, w| actions::attack(dir, ent, r, w)));
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
													todos.push(Box::new(move|r, w| actions::movedir(dir, ent, r, w)));
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
					AiState::Missile(dir, _, ref mut dur) => {
						if *dur == 0 {
							rmai.push(ent);
						} else {
							todos.push(Box::new(move|r, w| actions::movedir(dir, ent, r, w)));
							*dur -= 1;
						}
					},
					AiState::Melee(ref mut dur, _, src) => {
						if *dur == 0 {
							rmai.push(ent);
							todos.push(Box::new(move|r, w| actions::pickup(src, ent, r, w)));
						} else {
							todos.push(Box::new(move|r, w| actions::colcheck(ent, r, w)));
							*dur -= 1;
						}
					},
					//_ => (),
				}
			}
		} else {
			ai.tick -= 1
		}
	}
	for e in rmai {
		cai.remove(e);
	}
}
