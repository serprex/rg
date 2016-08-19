use std::mem;
use rand::{self, Rng};
use specs::*;
use super::components::*;
use super::util::*;

pub fn ailoop(arg: RunArg) {
	let (mut cpos, mut cnpos, mut cai, mut crace, mut cch, mut bag, mut weight, ents) = arg.fetch(|w|
		(w.write::<Pos>(), w.write::<NPos>(), w.write::<Ai>(), w.write::<Race>(), w.write::<Chr>(), w.write::<Bag>(), w.write::<Weight>(), w.entities())
	);
	let collisions: FnvHashSet<[i16; 3]> = cpos.iter().map(|pos| pos.xy).collect();
	let mut rng = rand::thread_rng();
	let mut newent: Vec<(Entity, Chr, Ai, Pos)> = Vec::new();
	let mut grab: Vec<(Entity, [i16; 3])> = Vec::new();
	for (pos, mut ai, &race, ent) in (&cpos, &mut cai, &crace, &ents).iter() {
		if ai.tick == 0 {
			let mut npos = NPos(pos.xy);
			ai.tick = ai.speed;
			match ai.state {
				AiState::Player => 'playerinput: loop {
					let ch = getch();
					match char_as_dir(ch) {
						Ok(d) => xy_incr_dir(&mut npos.0, d),
						Err('p') => {
							let ach = getch();
							match char_as_dir(ach) {
								Ok(d) => grab.push((ent, xyz_plus_dir(pos.xy, d))),
								_ => continue 'playerinput,
							}
						},
						Err('a') => {
							let ach = getch();
							let bp = match char_as_dir(ach) {
								Ok(d) => xyz_plus_dir(pos.xy, d),
								_ => continue 'playerinput,
							};
							newent.push((arg.create(), Chr(Char::from_char('x')), Ai::new(AiState::Melee(3), 1), Pos::new(bp)));
						},
						Err('s') => {
							let sch = getch();
							let (dir, bp) = match char_as_dir(sch) {
								Ok(d) => {
									(d, xyz_plus_dir(pos.xy, d))
								},
								_ => continue 'playerinput,
							};
							newent.push((arg.create(), Chr(Char::from_char('j')), Ai::new(AiState::Missile(dir), 4), Pos::new(bp)));
						},
						_ => (),
					}
					break
				},
				AiState::Random => {
					let mut choices: [[i16; 3]; 6] = unsafe { mem::uninitialized() };
					choices[0] = pos.xy;
					choices[1] = pos.xy;
					let mut chs = 2;
					for choice in &[[pos.xy[0]-1, pos.xy[1], pos.xy[2]],
					[pos.xy[0]+1, pos.xy[1], pos.xy[2]],
					[pos.xy[0], pos.xy[1]-1, pos.xy[2]],
					[pos.xy[0], pos.xy[1]+1, pos.xy[2]]] {
						if !collisions.contains(choice) {
							choices[chs] = *choice;
							chs += 1;
						}
					}
					npos.0 = *rng.choose(&choices[..chs]).unwrap();
					for (pos2, &race2, e2) in (&cpos, &crace, &ents).iter() {
						if ent != e2 && is_aggro(race, race2) &&
							(npos.0[0] - pos2.xy[0]).abs() < 5 &&
							(npos.0[1] - pos2.xy[1]).abs() < 5 {
							ai.state = AiState::Aggro(e2)
						}
					}
				},
				AiState::Scared(foe) => {
					match cpos.get(foe) {
						None => ai.state = AiState::Random,
						Some(fxy) => {
							let fxy = fxy.xy;
							let mut choices: [[i16; 3]; 4] = unsafe { mem::uninitialized() };
							let mut chs = 0;
							let dist = (pos.xy[0] - fxy[0]).abs() + (pos.xy[1] - fxy[1]).abs();
							for choice in &[[pos.xy[0]-1, pos.xy[1], pos.xy[2]],
							[pos.xy[0]+1, pos.xy[1], pos.xy[2]],
							[pos.xy[0], pos.xy[1]-1, pos.xy[2]],
							[pos.xy[0], pos.xy[1]+1, pos.xy[2]]] {
								if (pos.xy[0] - fxy[0]).abs() + (pos.xy[1] - fxy[1]).abs() > dist && !collisions.contains(choice) {
									choices[chs] = *choice;
									chs += 1;
								}
							}
							if chs == 0 {
								ai.state = AiState::Aggro(foe)
							} else {
								npos.0 = *rng.choose(&choices[..chs]).unwrap()
							}
						}
					}
				},
				AiState::Aggro(foe) => {
					match cpos.get(foe) {
						None => ai.state = AiState::Random,
						Some(fxy) => {
							let fxy = fxy.xy;
							match race {
								Race::Leylapan => {
									let mut dirs: [Dir; 2] = unsafe { mem::uninitialized() };
									let mut dnum = 0;
									if pos.xy[0] != fxy[0] {
										dirs[0] = if pos.xy[0] < fxy[0] {
											Dir::L
										} else {
											Dir::H
										};
										dnum = 1
									}
									if pos.xy[1] != fxy[1] {
										dirs[dnum] = if pos.xy[1] < fxy[1] {
											Dir::J
										} else {
											Dir::K
										};
										dnum += 1
									}
									if dnum > 0 {
										let fdir = *rng.choose(&dirs[..dnum]).unwrap();
										let bp = xyz_plus_dir(pos.xy, fdir);
										newent.push((arg.create(), Chr(Char::from_char('j')), Ai::new(AiState::Missile(fdir), 2), Pos::new(bp)));
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
										let nxy = xyz_plus_dir(pos.xy, mdir);
										if collisions.contains(&nxy) {
											ai.state = AiState::Scared(foe)
										} else {
											npos.0 = nxy
										}
									} else {
										ai.state = AiState::Scared(foe)
									}
								},
								_ => {
									let mut xxyy = pos.xy;
									let mut attacking = false;
									for &choice in &[[pos.xy[0]-1, pos.xy[1], pos.xy[2]],
									[pos.xy[0]+1, pos.xy[1], pos.xy[2]],
									[pos.xy[0], pos.xy[1]-1, pos.xy[2]],
									[pos.xy[0], pos.xy[1]+1, pos.xy[2]]] {
										if choice == fxy {
											newent.push((arg.create(), Chr(Char::from_char('x')), Ai::new(AiState::Melee(2), 1), Pos::new(choice)));
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
											let co = if pos.xy[0] != fxy[0] && rng.gen() { 0 } else { 1 };
											npos.0[co] += cmpi(pos.xy[co], fxy[co], 1, 0, -1);
										} else {
											ai.state = AiState::Random
										}
									}
								}
							}
						}
					}
				},
				AiState::Missile(dir) => {
					xy_incr_dir(&mut npos.0, dir);
				},
				AiState::Melee(ref mut dur) => {
					if *dur == 0 {
						arg.delete(ent)
					} else {
						*dur -= 1
					}
				},
				//_ => (),
			}
			if npos.0 != pos.xy {
				cnpos.insert(ent, npos);
			}
		} else {
			ai.tick -= 1
		}
	}
	let mut rments = Vec::new();
	for (ent, xyz) in grab {
		if let Some(ebag) = bag.get_mut(ent) {
			for (pos, wei, ent) in (&cpos, &weight, &ents).iter() {
				if pos.xy == xyz {
					ebag.0.push(ent);
					rments.push(ent);
				}
			}
		}
	}
	for ent in rments {
		cpos.remove(ent);
	}
	for (ent, newch, newai, newpos) in newent {
		cch.insert(ent, newch);
		cai.insert(ent, newai);
		cpos.insert(ent, newpos);
		crace.insert(ent, Race::None);
	}
}
