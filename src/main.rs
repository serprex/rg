extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;
extern crate fnv;

mod ailoop;
mod genroom_greedy;
mod util;
mod components;

use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{ATOMIC_BOOL_INIT, Ordering};
use std::thread;
use std::time::{Instant, Duration};
use specs::*;

use ailoop::ailoop;
use components::*;
use util::*;

macro_rules! w_register {
	($w: expr, $($comp: ident),*) => {
		$($w.register::<$comp>();)*
	}
}

fn main(){
	let mut planner = {
		let mut w = World::new();
		w_register!(w, Pos, NPos, Mortal, Ai, Portal, Race);
		w.create_now()
			.with(Ai::new(AiState::Player, 10))
			.with(Pos::new('@', [4, 4]))
			.with(Mortal(8))
			.with(Race::Wazzlefu)
			.build();
		w.create_now()
			.with(Pos::new('r', [6, 6]))
			.with(Ai::new(AiState::Random, 15))
			.with(Mortal(2))
			.with(Race::Rat)
			.build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify(40, 40, &mut w);
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::new(80, 60)));
	let _lock = TermJuggler::new();
	let newworld = Arc::new(ATOMIC_BOOL_INIT);
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
			now = if dur < Duration::from_millis(16) {
				let sleepdur = Duration::from_millis(16) - dur;
				thread::sleep(sleepdur);
				newnow + sleepdur
			} else {
				newnow
			};
			curselock.printnows(40, 0, &dur_as_f64(dur).to_string()[..6], x1b::TextAttr::empty());
			curselock.perframe_refresh_then_clear(x1b::TCell::from_char(' ')).unwrap();
		}
		planner.run_custom(ailoop);
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
											newworldrc.store(true, Ordering::Relaxed);
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
		planner.run_custom(|arg|{
			let mut cnpos = arg.fetch(|w| w.write::<NPos>());
			cnpos.clear();
		});
		let newwo = newworld.load(Ordering::Relaxed);
		if newwo {
			newworld.store(false, Ordering::Relaxed);
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
			rrg.modify(40, 40, planner.mut_world());
		}
	}
}
struct TermJuggler(termios::Termios);
impl TermJuggler {
	pub fn new() -> Option<Self> {
		use termios::*;
		if let Ok(ref mut term) = Termios::from_fd(0) {
			let oldterm = *term;
			cfmakeraw(term);
			tcsetattr(0, TCSANOW, term).expect("tcsetattr failed");
			print!("\x1bc\x1b[?25l");
			Some(TermJuggler(oldterm))
		} else {
			None
		}
	}
}
impl Drop for TermJuggler {
	fn drop(&mut self){
		use termios::*;
		x1b::Cursor::dropclear().ok();
		tcsetattr(0, TCSAFLUSH, &self.0).ok();
	}
}
