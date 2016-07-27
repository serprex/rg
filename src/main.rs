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
		w_register!(w, PosComp, MortalComp, PlayerComp,
			WallComp, AiComp);
		w.create_now()
			.with(PlayerComp)
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
	let cursefresh = curse.clone();
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
		cursefresh.lock().unwrap().perframe_refresh_then_clear(x1b::TCell::from_char(' ')).unwrap();
		let stdin = io::stdin();
		let sin = stdin.lock();
		let mut sinchars = sin.bytes();
		let ch = sinchars.next().unwrap().unwrap() as char;
		if ch == '\x1b' { break }
		planner.run1w1r(move|a: &mut PosComp, _: &PlayerComp|{
			a.nx = a.xy;
			match ch {
				'h' => a.nx[0] -= 1,
				'l' => a.nx[0] += 1,
				'k' => a.nx[1] -= 1,
				'j' => a.nx[1] += 1,
				_ => (),
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
				if n.xy != n.nx && !*collisions.get(&n.nx).unwrap_or(&false) {
					n.xy = n.nx;
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
