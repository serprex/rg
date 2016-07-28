extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;

mod genroom_greedy;
mod math;
mod components;

use std::sync::{Arc, Mutex};
use std::io::{self, Read};
use std::collections::hash_map::*;
use std::collections::HashSet;
use rand::*;
use specs::*;
use components::*;

macro_rules! w_register {
	($w: expr, $($comp: ident),*) => {
		$($w.register::<$comp>();)*
	}
}

fn main(){
	let rt = TermJuggler::new();
	let mut planner = {
		let mut w = World::new();
		w_register!(w, PosComp, MortalComp, AiComp);
		w.create_now()
			.with(AiComp::Player)
			.with(PosComp::new('@', [4, 4]))
			.with(MortalComp(8))
			.build();
		w.create_now()
			.with(PosComp::new('r', [6, 6]))
			.with(AiComp::Random)
			.with(MortalComp(2))
			.build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify(40, 40, [4, 4], &mut w);
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::new(40, 40)));
	loop {
		{
			let curselop = curse.clone();
			planner.run0w1r(move|a: &PosComp|{
				if a.xy[0] >= 0 && a.xy[1] >= 0 {
					curselop.lock().unwrap().set(a.xy[0] as u16, a.xy[1] as u16, x1b::TCell::from_char(a.ch));
				}
			});
		}
		planner.wait();
		curse.lock().unwrap().perframe_refresh_then_clear(x1b::TCell::from_char(' ')).unwrap();
		let stdin = io::stdin();
		let sin = stdin.lock();
		let mut sinchars = sin.bytes();
		let ch = sinchars.next().unwrap().unwrap() as char;
		if ch == '\x1b' { break }
		planner.run_custom(move |arg|{
			let (mut pos, mut ai) = arg.fetch(|w|{
				(w.write::<PosComp>(), w.write::<AiComp>())
			});
			let collisions: HashSet<[i16; 2]> = pos.iter().map(|pos| pos.xy).collect();
			let mut rng = rand::thread_rng();
			let mut pxy = [0, 0];
			for (pos, ai) in (&mut pos, &mut ai).iter() {
				match *ai {
					AiComp::Player => pxy = pos.xy,
					_ => (),
				}
			}
			for (mut pos, mut ai) in (&mut pos, &mut ai).iter() {
				match *ai {
					AiComp::Player => {
						match ch {
							'h' => pos.nx[0] -= 1,
							'l' => pos.nx[0] += 1,
							'k' => pos.nx[1] -= 1,
							'j' => pos.nx[1] += 1,
							_ => (),
						}
					},
					AiComp::Random => {
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
						pos.nx = *rng.choose(&choices[0..chs]).unwrap();
						if (pos.nx[0] - pxy[0]).abs() < 5 && (pos.nx[1] - pxy[1]).abs() < 5 {
							*ai = AiComp::Aggro
						}
					},
					AiComp::Scared => {
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
							*ai = AiComp::Aggro
						} else {
							pos.nx = *rng.choose(&choices[0..chs]).unwrap()
						}
					},
					AiComp::Aggro => {
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
							*ai = AiComp::Random
						}
					},
				}
			}
		});
		planner.wait();
		planner.run_custom(|arg|{
			let mut pos = arg.fetch(|w|{
				w.write::<PosComp>()
			});
			let mut collisions: HashMap<[i16; 2], bool> = HashMap::new();
			for n in pos.iter() {
				match collisions.entry(n.nx) {
					Entry::Occupied(mut ent) => {ent.insert(true);},
					Entry::Vacant(ent) => {ent.insert(false);},
				}
			}

			for n in (&mut pos).iter() {
				if n.xy != n.nx {
					if !*collisions.get(&n.nx).unwrap_or(&false) {
						n.xy = n.nx;
					} else {
						n.nx = n.xy;
					}
				}
			}
		});
		planner.wait();
	}
	std::mem::drop(rt);
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
