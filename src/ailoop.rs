use std::mem;
use rand::{self, Rng};
use specs::*;
use super::components::*;
use super::util::*;

pub fn ailoop(arg: RunArg) {
	let (mut cpos, mut cnpos, mut cai, mut crace, ents) = arg.fetch(|w|
		(w.write::<Pos>(), w.write::<NPos>(), w.write::<Ai>(), w.write::<Race>(), w.entities())
	);
	let collisions: FnvHashSet<[i16; 2]> = cpos.iter().map(|pos| pos.xy).collect();
	let mut rng = rand::thread_rng();
	let mut newent: Vec<(Entity, Ai, Pos)> = Vec::new();
	for (pos, mut ai, &race, ent) in (&cpos, &mut cai, &crace, &ents).iter() {
		if ai.tick == 0 {
			let mut npos = NPos(pos.xy);
			ai.tick = ai.speed;
			match ai.state {
				AiState::Player => 'playerinput: loop {
					let ch = getch();
					match ch {
						'h' => npos.0[0] -= 1,
						'l' => npos.0[0] += 1,
						'k' => npos.0[1] -= 1,
						'j' => npos.0[1] += 1,
						'a' => {
							let ach = getch();
							let mut crmis = |d, xy| {
								newent.push((arg.create(), Ai::new(AiState::Melee(d), 1), Pos::new('x', xy)));
							};
							match ach {
								'h' => crmis(3, [pos.xy[0]-1, pos.xy[1]]),
								'l' => crmis(3, [pos.xy[0]+1, pos.xy[1]]),
								'k' => crmis(3, [pos.xy[0], pos.xy[1]-1]),
								'j' => crmis(3, [pos.xy[0], pos.xy[1]+1]),
								_ => continue 'playerinput
							}
						},
						's' => {
							let sch = getch();
							let mut crmis = |d, xy| {
								newent.push((arg.create(), Ai::new(AiState::Missile(d), 4), Pos::new('j', xy)));
							};
							match sch {
								'h' => crmis(Dir::H, [pos.xy[0]-1, pos.xy[1]]),
								'l' => crmis(Dir::L, [pos.xy[0]+1, pos.xy[1]]),
								'k' => crmis(Dir::K, [pos.xy[0], pos.xy[1]-1]),
								'j' => crmis(Dir::J, [pos.xy[0], pos.xy[1]+1]),
								_ => continue 'playerinput
							}
						},
						_ => (),
					}
					break
				},
				AiState::Random => {
					let mut choices: [[i16; 2]; 6] = unsafe { mem::uninitialized() };
					choices[0] = pos.xy;
					choices[1] = pos.xy;
					let mut chs = 2;
					for choice in &[[pos.xy[0]-1, pos.xy[1]],
					[pos.xy[0]+1, pos.xy[1]],
					[pos.xy[0], pos.xy[1]-1],
					[pos.xy[0], pos.xy[1]+1]] {
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
							let mut choices: [[i16; 2]; 4] = unsafe { mem::uninitialized() };
							let mut chs = 0;
							let dist = (pos.xy[0] - fxy[0]).abs() + (pos.xy[1] - fxy[1]).abs();
							for choice in &[[pos.xy[0]-1, pos.xy[1]],
							[pos.xy[0]+1, pos.xy[1]],
							[pos.xy[0], pos.xy[1]-1],
							[pos.xy[0], pos.xy[1]+1]] {
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
							let mut xxyy = pos.xy;
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
				},
				AiState::Missile(dir) => {
					match dir {
						Dir::H => npos.0[0] -= 1,
						Dir::L => npos.0[0] += 1,
						Dir::K => npos.0[1] -= 1,
						Dir::J => npos.0[1] += 1,
					}
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
	for (ent, newai, newpos) in newent {
		cai.insert(ent, newai);
		cpos.insert(ent, newpos);
		crace.insert(ent, Race::None);
	}
}
