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
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Instant, Duration};
use specs::*;

use ailoop::ailoop;
use components::*;
use util::*;

macro_rules! w_register {
	($w: expr, $($comp: ty),*) => {
		$($w.register::<$comp>();)*
	}
}

macro_rules! fetch {
	(
		$arg:expr;
		$(let $rid:ident = $rty:ty;)*
		$(let mut $wid:ident = $wty:ty;)*
	) => {
		let ($($rid,)*$(mut $wid,)*) = $arg.fetch(|w| ($(w.read::<$rty>(),)* $(w.write::<$wty>(),)*));
	}
}

fn main(){
	let player;
	let mut planner = {
		let mut w = World::new();
		w_register!(w, Pos, NPos, Mortal, Ai, Portal, Race, Chr, Weight,
			Bag, Armor, Weapon, Head, Shield, AiStasis, Inventory,
			Def<Armor>, Def<Weapon>, Def<Head>, Def<Shield>,
			Atk<Armor>, Atk<Weapon>, Atk<Head>, Atk<Shield>);
		let dagger = w.create_now()
			.with(Chr(Char::from_char('x')))
			.with(Weight(2))
			.with(Atk::<Weapon>::new(1))
			.build();
		player = w.create_now()
			.with(Ai::new(AiState::Player, 10))
			.with(Bag(Vec::new()))
			.with(Chr(Char::from_char('@')))
			.with(Pos([4, 4, 0]))
			.with(Mortal(8))
			.with(Race::Wazzlefu)
			.with(Weapon(dagger))
			.build();
		w.create_now()
			.with(Chr(Char::from_char('r')))
			.with(Pos([6, 6, 0]))
			.with(Ai::new(AiState::Random, 12))
			.with(Mortal(4))
			.with(Race::Raffbarf)
			.build();
		w.create_now()
			.with(Chr(Char::from_char('k')))
			.with(Pos([20, 8, 0]))
			.with(Ai::new(AiState::Random, 8))
			.with(Mortal(2))
			.with(Race::Leylapan)
			.build();
		w.create_now()
			.with(Chr(Char::from_char('!')))
			.with(Pos([8, 8, 0]))
			.with(Atk::<Weapon>::new(2))
			.with(Weight(5))
			.build();
		w.create_now()
			.with(Chr(Char::from_char('#')))
			.with(Pos([8, 10, 0]))
			.with(Def::<Armor>::new(2))
			.with(Weight(5))
			.build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify([0, 0, 0], 40, 40, &mut w);
		rrg.modify([-10, -10, 1], 60, 60, &mut w);
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::<()>::new(80, 60)));
	let _lock = TermJuggler::new();
	let mut now = Instant::now();
	while !EXITGAME.load(Ordering::Relaxed) {
		{
			let curselop = curse.clone();
			planner.run_custom(move |arg|{
				fetch!{arg;
					let pos = Pos;
					let chr = Chr;
					let inventory = Inventory;
					let weapons = Weapon;
					let cbag = Bag;
				};
				if let Some(&Pos(plpos)) = pos.get(player) {
					let mut curseloplock = curselop.lock().unwrap();
					let pxy = plpos;
					for (&Pos(a), &Chr(ch)) in (&pos, &chr).iter() {
						let x = a[0] - pxy[0] + 6;
						let y = a[1] - pxy[1] + 6;
						if a[2] == pxy[2] && x >= 0 && x <= 12 && y >= 0 && y <= 12 {
							curseloplock.set(x as u16, y as u16, ch);
						}
					}
					for &Inventory(inve, invp) in inventory.iter() {
						if let Some(&Bag(ref bag)) = cbag.get(inve) {
							for (idx, &item) in bag.iter().enumerate() {
								let ch = chr.get(item).unwrap_or(&Chr(Char::from_char(' '))).0.gch();
								curseloplock.printnows(40, 1 + idx as u16,
									&format!("{}{:2} {}", if idx == invp { '>' } else { ' ' }, idx, ch),
									x1b::TextAttr::empty(), (), ());
							}
						}
						if let Some(&Weapon(item)) = weapons.get(inve) {
							let ch = chr.get(item).unwrap_or(&Chr(Char::from_char(' '))).0.gch();
							curseloplock.printnows(60, 1, &format!("Weapon: {}", ch),
								x1b::TextAttr::empty(), (), ());
						}
					}
				} else {
					EXITGAME.store(true, Ordering::Relaxed)
				}
			});
		}
		planner.wait();
		let newnow = Instant::now();
		let dur = newnow - now;
		now = if dur < Duration::from_millis(16) {
			let sleepdur = Duration::from_millis(16) - dur;
			thread::sleep(sleepdur);
			newnow + sleepdur
		} else {
			newnow
		};
		{
			let mut curselock = curse.lock().unwrap();
			curselock.printnows(40, 0, &dur_as_f64(dur).to_string()[..6], x1b::TextAttr::empty(), (), ());
			curselock.perframe_refresh_then_clear(Char::from_char(' ')).unwrap();
		}
		planner.run_custom(ailoop);
		planner.wait();
		planner.run_custom(move|arg|{
			let (mut pos, npos, mut mort, portal, ai, ents) = arg.fetch(|w|
				(w.write::<Pos>(), w.read::<NPos>(), w.write::<Mortal>(), w.read::<Portal>(), w.read::<Ai>(), w.entities())
			);
			let mut collisions: FnvHashMap<[i16; 3], Vec<Entity>> = Default::default();

			for (&Pos(p), e) in (&pos, &ents).iter() {
				let xy = npos.get(e).map(|&NPos(np)| np).unwrap_or(p);
				match collisions.entry(xy) {
					Entry::Occupied(mut ent) => {ent.get_mut().push(e);},
					Entry::Vacant(ent) => {ent.insert(vec![e]);},
				}
			}

			for (&mut Pos(ref mut p), &NPos(n)) in (&mut pos, &npos).iter() {
				let col = collisions.get(&n).unwrap();
				if col.len() == 1 {
					*p = n;
				}
			}

			for (_xyz, col) in collisions.into_iter() {
				if col.len() < 2 { continue }
				for &e in col.iter() {
					for &ce in col.iter() {
						if ce != e {
							if let Some(&Portal(porx)) = portal.get(e) {
								if let Some(&mut Pos(ref mut posxy)) = pos.get_mut(ce) {
									// maybe require a non-None race to portal?
									*posxy = porx
								}
							}
							if let Some(aie) = ai.get(e) {
								match aie.state {
									AiState::Missile(_) => {
										if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
											if *mce == 0 {
												arg.delete(ce);
											} else {
												*mce -= 1;
											}
										}
										arg.delete(e)
									},
									AiState::Melee(_) => {
										if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
											if *mce == 0 {
												arg.delete(ce);
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
		x1b::Cursor::<()>::dropclear().ok();
		tcsetattr(0, TCSAFLUSH, &self.0).ok();
	}
}
