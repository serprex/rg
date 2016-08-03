extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;
extern crate fnv;

mod genroom_greedy;
mod math;
mod components;

use std::sync::{Arc, Mutex};
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use std::collections::hash_map::*;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use rand::*;
use specs::*;
use components::*;
use fnv::FnvHasher;

macro_rules! w_register {
	($w: expr, $($comp: ident),*) => {
		$($w.register::<$comp>();)*
	}
}

static EXITGAME: AtomicBool = ATOMIC_BOOL_INIT;

fn getch() -> char {
	let stdin = io::stdin();
	let sin = stdin.lock();
	let mut sinchars = sin.bytes();
	let ch = sinchars.next().unwrap().unwrap() as char;
	if ch == '\x1b' { EXITGAME.store(true, Ordering::Relaxed) }
	ch
}

fn main(){
	let mut planner = {
		let mut w = World::new();
		w_register!(w, Pos, Mortal, Ai, Portal);
		w.create_now()
			.with(Ai::new(AiState::Player, 10))
			.with(Pos::new('@', [4, 4]))
			.with(Mortal(8))
			.build();
		w.create_now()
			.with(Pos::new('r', [6, 6]))
			.with(Ai::new(AiState::Random, 15))
			.with(Mortal(2))
			.build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify(40, 40, [4, 4], &mut w);
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::new(40, 40)));
	let _lock = TermJuggler::new();
	let newworld: Arc<Mutex<Option<World>>> = Arc::new(Mutex::new(None));
	while !EXITGAME.load(Ordering::Relaxed) {
		{
			let curselop = curse.clone();
			planner.run0w1r(move|a: &Pos|{
				if a.xy[0] >= 0 && a.xy[1] >= 0 {
					curselop.lock().unwrap().set(a.xy[0] as u16, a.xy[1] as u16, x1b::TCell::from_char(a.ch));
				}
			});
		}
		planner.wait();
		curse.lock().unwrap().perframe_refresh_then_clear(x1b::TCell::from_char(' ')).unwrap();
		planner.run_custom(move |arg|{
			let (mut pos, mut ai) = arg.fetch(|w|
				(w.write::<Pos>(), w.write::<Ai>())
			);
			let collisions: HashSet<[i16; 2], BuildHasherDefault<FnvHasher>> = pos.iter().map(|pos| pos.xy).collect();
			let mut rng = rand::thread_rng();
			let mut pxy = [0, 0];
			for (pos, ai) in (&mut pos, &mut ai).iter() {
				match ai.state {
					AiState::Player => pxy = pos.xy,
					_ => (),
				}
			}
			for (mut pos, mut ai) in (&mut pos, &mut ai).iter() {
				if ai.tick == 0 {
					ai.tick = ai.speed;
					match ai.state {
						AiState::Player => {
							let ch = getch();
							match ch {
								'h' => pos.nx[0] -= 1,
								'l' => pos.nx[0] += 1,
								'k' => pos.nx[1] -= 1,
								'j' => pos.nx[1] += 1,
								_ => (),
							}
						},
						AiState::Random => {
							let mut choices = [pos.xy; 6];
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
							pos.nx = *rng.choose(&choices[..chs]).unwrap();
							if (pos.nx[0] - pxy[0]).abs() < 5 && (pos.nx[1] - pxy[1]).abs() < 5 {
								ai.state = AiState::Aggro
							}
						},
						AiState::Scared => {
							let mut choices: [[i16; 2]; 4] = [[0, 0]; 4];
							let mut chs = 0;
							let dist = (pos.xy[0] - pxy[0]).abs() + (pos.xy[1] - pxy[1]).abs();
							for choice in &[[pos.xy[0]-1, pos.xy[1]],
							[pos.xy[0]+1, pos.xy[1]],
							[pos.xy[0], pos.xy[1]-1],
							[pos.xy[0], pos.xy[1]+1]] {
								if (pos.xy[0] - pxy[0]).abs() + (pos.xy[1] - pxy[1]).abs() > dist && !collisions.contains(choice) {
									choices[chs] = *choice;
									chs += 1;
								}
							}
							if chs == 0 {
								ai.state = AiState::Aggro
							} else {
								pos.nx = *rng.choose(&choices[..chs]).unwrap()
							}
						},
						AiState::Aggro => {
							let mut xxyy = pos.xy;
							let mut tries = 3;
							loop {
								let mut xy = xxyy;
								let co = if tries == 1 || (tries == 3 && rng.gen()) { 0 } else { 1 };
								xy[co] += if xy[co]<pxy[0] { 1 }
									else if xy[co]>pxy[0] { -1 }
									else { 0 };
								if xy == xxyy || collisions.contains(&xy) {
									tries -= 1;
									if tries == 0 { break }
								} else {
									xxyy = xy;
									if xy == pxy {
										break
									} else {
										tries = 3
									}
								}
							}
							if xxyy == pxy {
								let co = if pos.xy[0] != pxy[0] && rng.gen() { 0 } else { 1 };
								pos.nx[co] += if pos.xy[co]<pxy[co] { 1 }
									else if pos.xy[co]>pxy[co] { -1 }
									else { 0 };
							} else {
								ai.state = AiState::Random
							}
						},
					}
				} else {
					ai.tick -= 1
				}
			}
		});
		planner.wait();
		let newworldrc = newworld.clone();
		planner.run_custom(move|arg|{
			let (mut pos, portal, ai, ents) = arg.fetch(|w|
				(w.write::<Pos>(), w.read::<Portal>(), w.read::<Ai>(), w.entities())
			);
			let mut collisions: HashMap<[i16; 2], Vec<Entity>, BuildHasherDefault<FnvHasher>> = Default::default();
			for (n, e) in (&pos, &ents).iter() {
				match collisions.entry(n.nx) {
					Entry::Occupied(mut ent) => {ent.get_mut().push(e);},
					Entry::Vacant(ent) => {ent.insert(vec![e]);},
				}
			}

			for (n, e) in (&mut pos, &ents).iter() {
				if n.xy != n.nx {
					let col = collisions.get(&n.nx).unwrap();
					if col.len() == 1 {
						n.xy = n.nx;
					} else {
						for ce in col {
							if *ce != e {
								if let Some(aie) = ai.get(e) {
									if aie.state == AiState::Player {
										if let Some(_pore) = portal.get(*ce) {
											let mut world = World::new();
											w_register!(world, Pos, Mortal, Ai, Portal);
											world.create_now()
												.with(Ai::new(AiState::Player, 10))
												.with(Pos::new('@', n.nx))
												.with(Mortal(8))
												.build();
											let rrg = genroom_greedy::GreedyRoomGen::default();
											rrg.modify(40, 40, [4, 4], &mut world);
											let mut neww = newworldrc.lock().unwrap();
											*neww = Some(world);
										}
									}
								}
							}
						}
						n.nx = n.xy;
					}
				}
			}
		});
		planner.wait();
		let newwo = newworld.lock().unwrap().take();
		if newwo.is_some() {
			*planner.mut_world() = newwo.unwrap();
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
