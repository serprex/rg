extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;
extern crate fnv;

mod genroom_greedy;
mod math;
mod components;

use std::collections::hash_map::*;
use std::collections::HashSet;
use std::cmp::{self, Ord};
use std::env;
use std::hash::BuildHasherDefault;
use std::io::{self, Read};
use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use std::thread;
use std::time::{Instant, Duration};
use rand::*;
use specs::*;
use fnv::FnvHasher;

use components::*;

pub type FnvHashSet<T> = HashSet<T, BuildHasherDefault<FnvHasher>>;
pub type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

macro_rules! w_register {
	($w: expr, $($comp: ident),*) => {
		$($w.register::<$comp>();)*
	}
}

static EXITGAME: AtomicBool = ATOMIC_BOOL_INIT;

fn dur_as_f64(dur: Duration) -> f64 {
	dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1e9
}

fn cmpi<T, U>(a: T, b: T, lt: U, eq: U, gt: U) -> U
	where T: Ord
{
	match a.cmp(&b) {
		cmp::Ordering::Less => lt,
		cmp::Ordering::Equal => eq,
		cmp::Ordering::Greater => gt,
	}
}

fn getch() -> char {
	let stdin = io::stdin();
	let sin = stdin.lock();
	let mut sinchars = sin.bytes();
	let ch = sinchars.next().map(|next| next.unwrap_or(0x1b) as char).unwrap_or('\x1b');
	if ch == '\x1b' { EXITGAME.store(true, Ordering::Relaxed) }
	ch
}

fn is_aggro(r1: Race, r2: Race) -> bool {
	match (r1, r2) {
		(Race::Wazzlefu, Race::Rat) => true,
		(Race::Rat, Race::Wazzlefu) => true,
		_ => false,
	}
}

fn main(){
	let mut planner = {
		let mut w = World::new();
		w_register!(w, Pos, NPos, Mortal, Ai, Portal, Race);
		w.create_now()
			.with(Ai::new(AiState::Player, 10))
			.with(Pos::new('@', [4, 4]))
			.with(NPos([4, 4]))
			.with(Mortal(8))
			.with(Race::Wazzlefu)
			.build();
		w.create_now()
			.with(Pos::new('r', [6, 6]))
			.with(NPos([6, 6]))
			.with(Ai::new(AiState::Random, 15))
			.with(Mortal(2))
			.with(Race::Rat)
			.build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify(40, 40, [4, 4], &mut w);
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::new(80, 60)));
	let _lock = if env::args().len() < 2
		{ Some(TermJuggler::new()) } else { None };
	let newworld: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
	let mut now = Instant::now();
	while !EXITGAME.load(Ordering::Relaxed) {
		{
			let curselop = curse.clone();
			planner.run_custom(move |arg|{
				let mut curseloplock = curselop.lock().unwrap();
				let pos = arg.fetch(|w| w.read::<Pos>());
				for a in pos.iter() {
					if a.xy[0] >= 0 && a.xy[1] >= 0 {
						curseloplock.set(a.xy[0] as u16, a.xy[1] as u16, x1b::TCell::from_char(a.ch));
					}
				}
			});
		}
		planner.wait();
		{
			let mut curselock = curse.lock().unwrap();
			let newnow = Instant::now();
			let dur = newnow - now;
			if dur < Duration::from_millis(16) {
				let sleepdur = Duration::from_millis(16) - dur;
				thread::sleep(sleepdur);
				now = newnow + sleepdur;
			} else {
				now = newnow;
			}
			curselock.printnows(40, 0, &dur_as_f64(dur).to_string()[..6], x1b::TextAttr::empty());
			curselock.perframe_refresh_then_clear(x1b::TCell::from_char(' ')).unwrap();
		}
		planner.run_custom(move |arg|{
			let (mut cpos, mut cnpos, mut cai, mut crace, ents) = arg.fetch(|w|
				(w.write::<Pos>(), w.write::<NPos>(), w.write::<Ai>(), w.write::<Race>(), w.entities())
			);
			let collisions: FnvHashSet<[i16; 2]> = cpos.iter().map(|pos| pos.xy).collect();
			let mut rng = rand::thread_rng();
			let mut newent: Vec<(Entity, Ai, Pos, Option<NPos>)> = Vec::new();
			for (pos, mut npos, mut ai, &race, ent) in (&cpos, &mut cnpos, &mut cai, &crace, &ents).iter() {
				npos.0 = pos.xy;
				if ai.tick == 0 {
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
										newent.push((arg.create(), Ai::new(AiState::Melee(d), 1), Pos::new('x', xy), None));
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
										newent.push((arg.create(), Ai::new(AiState::Missile(d), 4), Pos::new('j', xy), Some(NPos(xy))));
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
				} else {
					ai.tick -= 1
				}
			}
			for (ent, newai, newpos, newnpos) in newent {
				cai.insert(ent, newai);
				cpos.insert(ent, newpos);
				crace.insert(ent, Race::None);
				if let Some(npos) = newnpos {
					cnpos.insert(ent, npos);
				}
			}
		});
		planner.wait();
		let newworldrc = newworld.clone();
		planner.run_custom(move|arg|{
			let (mut pos, npos, mut mort, portal, ai, ents) = arg.fetch(|w|
				(w.write::<Pos>(), w.read::<NPos>(), w.write::<Mortal>(), w.read::<Portal>(), w.read::<Ai>(), w.entities())
			);
			let mut collisions: FnvHashMap<[i16; 2], Vec<Entity>> = Default::default();
			for (n, _, e) in (&pos, !&npos, &ents).iter() {
				match collisions.entry(n.xy) {
					Entry::Occupied(mut ent) => {ent.get_mut().push(e);},
					Entry::Vacant(ent) => {ent.insert(vec![e]);},
				}
			}

			for (n, e) in (&npos, &ents).iter() {
				match collisions.entry(n.0) {
					Entry::Occupied(mut ent) => {ent.get_mut().push(e);},
					Entry::Vacant(ent) => {ent.insert(vec![e]);},
				}
			}

			for (mut p, n, e) in (&mut pos, &npos, &ents).iter() {
				let col = collisions.get(&n.0).unwrap();
				if col.len() == 1 {
					p.xy = n.0;
				} else {
					for ce in col {
						if *ce != e {
							if let Some(aie) = ai.get(e) {
								match aie.state {
									AiState::Player => {
										if let Some(_pore) = portal.get(*ce) {
											*newworldrc.lock().unwrap() = true;
										}
									},
									AiState::Missile(_) => {
										if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(*ce) {
											if *mce == 0 {
												arg.delete(*ce);
											} else {
												*mce -= 1;
											}
										}
										arg.delete(e)
									},
									AiState::Melee(_) => {
										if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(*ce) {
											if *mce == 0 {
												arg.delete(*ce);
											} else {
												*mce -= 1;
											}
										}
									},
									_ => (),
								}
							}
						}
					}
				}
			}
		});
		planner.wait();
		let mut newwo = newworld.lock().unwrap();
		if *newwo {
			*newwo = false;
			let mut rments = Vec::new();
			{
			let world = planner.mut_world();
			let ents = world.entities();
			let cai = world.read::<Ai>();
			for ent in ents.iter() {
				if let Some(ai) = cai.get(ent) {
					if AiState::Player == ai.state {
						continue
					}
				}
				rments.push(ent);
			}
			}
			let rments = Arc::new(Mutex::new(rments));
			let rmentsrc = rments.clone();
			planner.run_custom(move |arg|{
				arg.fetch(|w| {
					let rme = rmentsrc.lock().unwrap();
					for &e in &*rme {
						w.delete_later(e);
					}
				});
			});
			planner.wait();
			let rrg = genroom_greedy::GreedyRoomGen::default();
			rrg.modify(40, 40, [4, 4], planner.mut_world());
		}
	}
}
struct TermJuggler(termios::Termios);
impl TermJuggler {
	pub fn new() -> Self {
		use termios::*;
		let mut term = Termios::from_fd(0).unwrap();
		let oldterm = term;
		cfmakeraw(&mut term);
		tcsetattr(0, TCSANOW, &term).expect("tcsetattr failed");
		print!("\x1bc\x1b[?25l");
		TermJuggler(oldterm)
	}
}
impl Drop for TermJuggler {
	fn drop(&mut self){
		use termios::*;
		x1b::Cursor::dropclear().ok();
		tcsetattr(0, TCSAFLUSH, &self.0).ok();
	}
}
