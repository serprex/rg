use std::mem;
use rand::Rng;
use specs::*;
use components::*;
use util::*;
use actions;
use position::Possy;

pub fn ailoop<R: Rng>(rng: &mut R, w: &mut World) {
	let (cpos, mut cnpos, mut cai, crace) =
		(w.read::<Pos>(), w.write::<NPos>(), w.write::<Ai>(), w.read::<Race>());
	let possy = w.read_resource::<Possy>();
	let ents = w.entities();
	let Todo(ref mut todos) = *w.write_resource::<Todo>();
	for (_, mut ai, &race, ent) in (&cpos, &mut cai, &crace, &ents).iter() {
		if let Some(pos) = possy.get_pos(ent) {
			if ai.tick == 0 {
				let mut npos = pos;
				ai.tick = ai.speed;
				match ai.state {
					AiState::PlayerInventory(invp) => {
						ai.tick = 1;
						let mut bag = w.write::<Bag>();
						let mut weapons = w.write::<Weapon>();
						if let Some(&mut Bag(ref mut ebag)) = bag.get_mut(ent) {
							'invput: loop {
								match getch() {
									'i' => {
										ai.state = AiState::Player;
										ai.tick = 10;
									},
									'j' if ebag.len() > 0 =>
										ai.state = AiState::PlayerInventory(if invp == ebag.len()-1 { 0 } else { invp + 1 }),
									'k' if ebag.len() > 0 =>
										ai.state = AiState::PlayerInventory(if invp == 0 { ebag.len()-1 } else { invp - 1 }),
									'w' => {
										match weapons.insert(ent, Weapon(ebag.remove(invp))) {
											InsertResult::Updated(Weapon(oldw)) => ebag.push(oldw),
											_ => (),
										}
										if invp == ebag.len() {
											ai.state = AiState::PlayerInventory(0);
										}
									},
									'W' => {
										weapons.remove(ent);
									},
									c if c >= '0' && c <= '9' => {
										let v = (c as u32 as u8 - b'0') as usize;
										if v < ebag.len() {
											ai.state = AiState::PlayerInventory(v);
										}
									},
									'\x1b' => (),
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
									todos.push(Box::new(move|w| actions::blink(ent, w)));
								} else {
									continue;
								}
							}
							*casting = AiState::Player;
						}
					},
					AiState::Player => 'playerinput: loop {
						let ch = getch();
						match char_as_dir(ch) {
							Ok(d) => xy_incr_dir(&mut npos, d),
							Err('p') => {
								match char_as_dir(getch()) {
									Ok(d) => {
										let gp = xyz_plus_dir(pos, d);
										todos.push(Box::new(move|w| actions::grab(gp, ent, w)));
									},
									_ => continue 'playerinput,
								}
							},
							Err('i') => {
								ai.state = AiState::PlayerInventory(0);
							},
							Err('a') => {
								match char_as_dir(getch()) {
									Ok(d) => todos.push(Box::new(move|w| actions::attack(d, ent, w))),
									_ => continue 'playerinput,
								}
							},
							Err('s') => {
								match char_as_dir(getch()) {
									Ok(d) => todos.push(Box::new(move|w| actions::shoot(d, ent, w))),
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
						let mut choices: [[i16; 3]; 6] = unsafe { mem::uninitialized() };
						choices[0] = pos;
						choices[1] = pos;
						let mut chs = 2;
						for choice in &[[pos[0]-1, pos[1], pos[2]],
						[pos[0]+1, pos[1], pos[2]],
						[pos[0], pos[1]-1, pos[2]],
						[pos[0], pos[1]+1, pos[2]]] {
							if !possy.contains(*choice) {
								choices[chs] = *choice;
								chs += 1;
							}
						}
						npos = *rng.choose(&choices[..chs]).unwrap();
						let near = possy.get_within(npos, 5);
						for (e2, _) in near {
							if ent != e2 {
								if let Some(&race2) = crace.get(e2) {
									if is_aggro(race, race2) {
										ai.state = AiState::Aggro(e2)
									}
								}
							}
						}
					},
					AiState::Scared(foe) => {
						match possy.get_pos(foe) {
							None => ai.state = AiState::Random,
							Some(fxy) => {
								let mut choices: [[i16; 3]; 4] = unsafe { mem::uninitialized() };
								let mut chs = 0;
								let dist = (pos[0] - fxy[0]).abs() + (pos[1] - fxy[1]).abs();
								for choice in &[[pos[0]-1, pos[1], pos[2]],
								[pos[0]+1, pos[1], pos[2]],
								[pos[0], pos[1]-1, pos[2]],
								[pos[0], pos[1]+1, pos[2]]] {
									if (pos[0] - fxy[0]).abs() + (pos[1] - fxy[1]).abs() > dist && !possy.contains(*choice) {
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
						match possy.get_pos(foe) {
							None => ai.state = AiState::Random,
							Some(fxy) => {
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
										if let Some(&fdir) = rng.choose(&dirs[..dnum]) {
											todos.push(Box::new(move|w| actions::shoot(fdir, ent, w)));
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
												npos = nxy
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
												todos.push(Box::new(move|w| actions::attack(dir, ent, w)));
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
	}
}
