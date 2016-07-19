extern crate rand;
extern crate termios;
extern crate x1b;
#[macro_use] extern crate bitflags;
extern crate specs;

mod genroom_greedy;
mod math;

use specs::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic;
use std::io::{self, Read};

/*
macro_rules! impl_vec_storage {
	($x: ident) => {{
		impl Component for $x {
			let Storage = VecStorage<Self>;
		}
	}}
}*/
struct PosComp([i16; 2]);
impl Component for PosComp {
	type Storage = VecStorage<Self>;
}
struct NewPosComp([i16; 2]);
impl Component for NewPosComp {
	type Storage = VecStorage<Self>;
}
struct RenderComp(char);
impl Component for RenderComp {
	type Storage = VecStorage<Self>;
}
struct MortalComp(i16);
impl Component for MortalComp {
	type Storage = VecStorage<Self>;
}
#[derive(Clone, Default)]
struct PlayerComp;
impl Component for PlayerComp {
	type Storage = NullStorage<Self>;
}
#[derive(Clone, Default)]
pub struct WallComp([i16; 2]);
impl Component for WallComp {
	type Storage = VecStorage<Self>;
}

fn main(){
	let rt = TermJuggler::new();
	let mut planner = {
		let mut w = World::new();
		w.register::<PosComp>();
		w.register::<NewPosComp>();
		w.register::<RenderComp>();
		w.register::<MortalComp>();
		w.register::<PlayerComp>();
		w.register::<WallComp>();
		w.create_now().with(PlayerComp)
			.with(PosComp([4, 4]))
			.with(NewPosComp([4, 4]))
			.with(MortalComp(8))
			.with(RenderComp('@')).build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify(40, 40, [4, 4], &mut w);
		w.create_now().with(WallComp([5, 5])).build();
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::new(40, 40)));
	let cursefresh = curse.clone();
	loop {
		{
			let curselop = curse.clone();
			planner.run0w1r(move|b: &WallComp|{
				let curse = curselop.clone();
				curse.lock().unwrap().set(b.0[0] as u16, b.0[1] as u16, x1b::TCell::from_char('#'));
			});
			let curselop = curse.clone();
			planner.run0w2r(move|c: &RenderComp, b: &PosComp|{
				let curse = curselop.clone();
				curse.lock().unwrap().set(b.0[0] as u16, b.0[1] as u16, x1b::TCell::from_char(c.0));
			});
		}
		planner.wait();
		cursefresh.lock().unwrap().perframe_refresh_then_clear(x1b::TCell::from_char(' ')).unwrap();
		let stdin = io::stdin();
		let sin = stdin.lock();
		let mut sinchars = sin.bytes();
		let ch = sinchars.next().unwrap().unwrap() as char;
		if ch == '\x1b' { break }
		planner.run1w2r(move|a: &mut NewPosComp, b: &PosComp, _: &PlayerComp|{
			a.0 = b.0;
			match ch {
				'h' => a.0[0] -= 1,
				'l' => a.0[0] += 1,
				'j' => a.0[1] -= 1,
				'k' => a.0[1] += 1,
				_ => (),
			}
		});
		planner.wait();
		planner.run_custom(|arg|{
			let (mut pos, newpos, walls, ents) = arg.fetch(|w|{
				(w.write::<PosComp>(), w.read::<NewPosComp>(), w.read::<WallComp>(), w.entities())
			});

			'outer:
			for (p, n) in (&mut pos, &newpos).iter() {
				for wp in walls.iter() {
					if wp.0 == n.0 { break 'outer; }
				}
				p.0 = n.0;
			}
		});
		planner.wait();
	}
	rt.end();
}
struct TermJuggler;
impl TermJuggler {
	pub fn new() -> Self {
		use termios::*;
		let mut term = Termios::from_fd(0).unwrap();
		cfmakeraw(&mut term);
		term.c_lflag &= !ECHO;
		tcsetattr(0, TCSANOW, &term);
		print!("\x1bc\x1b[?25l");
		TermJuggler
	}
	pub fn end(self) {}
}
impl Drop for TermJuggler {
	fn drop(&mut self){
		use termios::*;
		x1b::Cursor::dropclear();
		if let Ok(mut term) = Termios::from_fd(0) {
			term.c_lflag |= ECHO;
			tcsetattr(0, TCSAFLUSH, &term);
		}
	}
}
