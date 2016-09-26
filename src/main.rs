extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;
extern crate fnv;
extern crate smallvec;

mod ailoop;
mod roomgen;
mod greedgrow;
mod genroom_greedy;
mod genroom_forest;
mod util;
mod components;
mod actions;
mod super_sparse_storage;
mod position;

use std::collections::hash_map::Entry;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Instant, Duration};

use rand::{Rng, Rand, XorShiftRng};
use specs::*;
use smallvec::SmallVec;
use x1b::RGB4;

use ailoop::ailoop;
use components::*;
use util::*;
use roomgen::RoomGen;
use position::Possy;

macro_rules! w_register {
	($w: expr, $($comp: ty),*) => {
		$($w.register::<$comp>();)*
	}
}

fn main(){
	let mut rng = XorShiftRng::rand(&mut rand::thread_rng());
	let mut w = World::new();
	w_register!(w, Pos, NPos, Mortal, Ai, Portal, Race, Chr, Weight, Strength,
		WDirection, Bow, Heal, Casting,
		Bag, Armor, Weapon, Head, Shield, AiStasis, Inventory, Solid, Spell,
		Def<Armor>, Def<Weapon>, Def<Head>, Def<Shield>,
		Atk<Armor>, Atk<Weapon>, Atk<Head>, Atk<Shield>);
	w.add_resource(Walls::default());
	w.add_resource(Todo::default());
	w.create_now()
		.with(Chr(Char::from('x')))
		.with(Weight(3))
		.with(Atk::<Weapon>::new(1, 1, -2))
		.with(NPos([4, 8, 0]))
		.with(Pos)
		.build();
	w.create_now()
		.with(Chr(Char::from('b')))
		.with(Weight(2))
		.with(Bow(4, 1))
		.with(NPos([2, 5, 0]))
		.with(Pos)
		.build();
	let player = w.create_now()
		.with(Ai::new(AiState::Player, 10))
		.with(Bag(Vec::new()))
		.with(Chr(Char::from('@')))
		.with(NPos([4, 4, 0]))
		.with(Pos)
		.with(Solid)
		.with(Mortal(20))
		.with(Race::Wazzlefu)
		.with(Strength(10))
		.with(Weight(30))
		.build();
	w.create_now()
		.with(Chr(Char::from('r')))
		.with(NPos([10, 6, 0]))
		.with(Pos)
		.with(Solid)
		.with(Ai::new(AiState::Random, 12))
		.with(Mortal(4))
		.with(Weight(10))
		.with(Race::Raffbarf)
		.build();
	w.create_now()
		.with(Chr(Char::from('k')))
		.with(NPos([20, 8, 0]))
		.with(Pos)
		.with(Solid)
		.with(Ai::new(AiState::Random, 8))
		.with(Mortal(2))
		.with(Weight(20))
		.with(Race::Leylapan)
		.build();
	w.create_now()
		.with(Chr(Char::from('!')))
		.with(NPos([8, 8, 0]))
		.with(Pos)
		.with(Atk::<Weapon>::new(2, 3, 2))
		.with(Weight(5))
		.build();
	w.create_now()
		.with(Chr(Char::from('#')))
		.with(NPos([8, 10, 0]))
		.with(Pos)
		.with(Solid)
		.with(Def::<Armor>::new(2))
		.with(Weight(5))
		.build();
	{
		let possy = Possy::new(&mut w);
		w.add_resource(possy);
	}
	{
	let rrg = genroom_greedy::GreedyRoomGen::default();
	let frg = genroom_forest::ForestRoomGen::default();
	let mut f1 = [[20, 10, 22, 12], [30, 32, 20, 22], [20, 30, 24, 36], [50, 50, 55, 55], [60, 50, 62, 52], [80, 60, 82, 70], [90, 90, 95, 105]];
	let fadj = greedgrow::grow(&mut rng, &mut f1, 0, 0, 120, 120);
	/*for fxy in f1.iter() {
		if rng.gen() {
			rrg.generate(&mut rng, [fxy[0], fxy[1], 1], fxy[2]-fxy[0], fxy[3]-fxy[1], &mut w)
		} else {
			frg.generate(&mut rng, [fxy[0], fxy[1], 1], fxy[2]-fxy[0], fxy[3]-fxy[1], &mut w)
		}
	}*/
	rrg.generate(&mut rng, [0, 0, 0], 40, 40, &mut w);
	rrg.generate(&mut rng, [-10, -10, 1], 60, 60, &mut w);
	frg.generate(&mut rng, [0, 0, 2], 40, 80, &mut w);
	frg.generate(&mut rng, [0, 0, 3], 40, 80, &mut w);
	}
	let mut curse = x1b::Curse::<RGB4>::new(80, 60);
	let _lock = TermJuggler::new();
	let mut now = Instant::now();
	while !EXITGAME.load(Ordering::Relaxed) {
		{
			let possy = w.read_resource::<Possy>();
			if let Some(plpos) = possy.get_pos(player) {
				let pos = w.read::<Pos>();
				let (chr, inventory, weapons, cbag) =
					(w.read::<Chr>(), w.read::<Inventory>(), w.read::<Weapon>(), w.read::<Bag>());
				let pxy = plpos;
				{
				let Walls(ref walls) = *w.read_resource::<Walls>();
				let mut xyz = pxy;
				for x in 0..12 {
					xyz[0] = pxy[0] + x - 6;
					for y in 0..12 {
						xyz[1] = pxy[1] + y - 6;
						if let Some(&ch) = walls.get(&xyz) {
							curse.set(x as u16, y as u16, ch);
						}
					}
				}
				}
				for (_, &Chr(ch), e) in (&pos, &chr, &w.entities()).iter() {
					if let Some(a) = possy.get_pos(e) {
						let x = a[0] - pxy[0] + 6;
						let y = a[1] - pxy[1] + 6;
						if a[2] == pxy[2] && x >= 0 && x <= 12 && y >= 0 && y <= 12 {
							curse.set(x as u16, y as u16, ch);
						}
					}
				}
				for &Inventory(inve, invp) in inventory.iter() {
					if let Some(&Bag(ref bag)) = cbag.get(inve) {
						if bag.is_empty() {
							curse.printnows(40, 1, "Empty", x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
						} else {
							for (idx, &item) in bag.iter().enumerate() {
								let ch = chr.get(item).unwrap_or(&Chr(Char::from(' '))).0.get_char();
								curse.printnows(40, 1 + idx as u16,
									&format!("{}{:2} {}", if idx == invp { '>' } else { ' ' }, idx, ch),
									x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
							}
						}
					}
					if let Some(&Weapon(item)) = weapons.get(inve) {
						let ch = chr.get(item).unwrap_or(&Chr(Char::from(' '))).0.get_char();
						curse.printnows(60, 1, &format!("Weapon: {}", ch),
							x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
					}
				}
			} else {
				EXITGAME.store(true, Ordering::Relaxed)
			}
		}
		let newnow = Instant::now();
		let dur = newnow - now;
		now = if dur < Duration::from_millis(16) {
			let sleepdur = Duration::from_millis(16) - dur;
			thread::sleep(sleepdur);
			newnow + sleepdur
		} else {
			newnow
		};
		curse.printnows(40, 0, &dur_as_string(dur), x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
		curse.perframe_refresh_then_clear(Char::from(' ')).unwrap();
		ailoop(&mut rng, &mut w);
		loop {
			w.maintain();
			let todo = {
				let Todo(ref mut todos) = *w.write_resource::<Todo>();
				if todos.is_empty() { break }
				let todo = todos.drain(..).collect::<Vec<_>>();
				todo // stupid lifetime inference
			};
			for (ent, action) in todo {
				action(ent, &mut w)
			}
		}
		{
			let (pos, npos, mut mort, portal, mut ai, mut solid, ents) =
				(w.read::<Pos>(), w.read::<NPos>(), w.write::<Mortal>(), w.read::<Portal>(), w.write::<Ai>(), w.write::<Solid>(), w.entities());
			let Walls(ref walls) = *w.read_resource::<Walls>();
			let mut possy = w.write_resource::<Possy>();

			'newposloop:
			for (_, &NPos(n), ent) in (&pos, &npos, &ents).iter() {
				if walls.contains_key(&n) {
					continue 'newposloop
				}
				for &e in possy.npos_map(&npos, &ents).get_ents(n).into_iter() {
					if e != ent && solid.get(e).is_some() {
						continue 'newposloop
					}
				}
				possy.set_pos(ent, n);
			}

			let mut rmai = SmallVec::<[Entity; 2]>::new();
			let mut spos = SmallVec::<[(Entity, [i16; 3]); 2]>::new();
			for (_xyz, col) in possy.npos_map(&npos, &ents).collisions().into_iter() {
				if col.len() < 2 { continue }
				for &e in col.iter() {
					for &ce in col.iter() {
						if ce != e {
							if let Some(&Portal(porx)) = portal.get(e) {
								spos.push((ce, porx));
							}
							if let Some(aie) = ai.get(e) {
								match aie.state {
									AiState::Missile(_, dmg) => {
										if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
											if *mce <= dmg {
												*mce = 0;
												solid.remove(ce);
												rmai.push(ce);
											} else {
												*mce -= dmg;
											}
										}
										if let Some(_) = solid.get(ce) {
											w.delete_later(e)
										}
									},
									AiState::Melee(_, dmg) => {
										if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
											if *mce <= dmg {
												*mce = 0;
												solid.remove(ce);
												rmai.push(ce);
											} else {
												*mce -= dmg;
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
			for e in rmai {
				ai.remove(e);
			}
			for (e, p) in spos {
				possy.set_pos(e, p);
			}
		}
		w.write::<NPos>().clear();
		w.maintain();
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
