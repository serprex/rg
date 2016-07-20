extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;

mod genroom_greedy;
mod math;

use specs::*;
use std::sync::{Arc, Mutex};
use std::io::{self, Read};
use std::collections::hash_map::*;

macro_rules! impl_storage {
	($storage: ident, $($comp: ident),*) => {
		$(impl Component for $comp {
			type Storage = $storage<Self>;
		})*
	}
}
struct PosComp([i16; 2]);
struct NewPosComp([i16; 2]);
struct RenderComp(char);
struct MortalComp(i16);
#[derive(Clone, Default)]
struct PlayerComp;
#[derive(Clone, Default)]
struct AggroComp;
#[derive(Clone, Default)]
pub struct WallComp;
impl_storage!(VecStorage, PosComp, NewPosComp, RenderComp, MortalComp);
impl_storage!(NullStorage, PlayerComp, AggroComp, WallComp);

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
		w.register::<AggroComp>();
		w.create_now().with(PlayerComp)
			.with(PosComp([4, 4]))
			.with(NewPosComp([4, 4]))
			.with(MortalComp(8))
			.with(RenderComp('@')).build();
		w.create_now().with(AggroComp)
			.with(PosComp([6, 6]))
			.with(NewPosComp([6, 6]))
			.with(MortalComp(2))
			.with(RenderComp('r')).build();
		let rrg = genroom_greedy::GreedyRoomGen::default();
		rrg.modify(40, 40, [4, 4], &mut w);
		Planner::<()>::new(w, 2)
	};
	let curse = Arc::new(Mutex::new(x1b::Curse::new(40, 40)));
	let cursefresh = curse.clone();
	loop {
		{
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
				'k' => a.0[1] -= 1,
				'j' => a.0[1] += 1,
				_ => (),
			}
		});
		planner.wait();
		planner.run_custom(|arg|{
			let (mut pos, newpos) = arg.fetch(|w|{
				(w.write::<PosComp>(), w.read::<NewPosComp>())
			});
			let mut collisions: HashMap<[i16; 2], u16> = HashMap::new();
			for n in newpos.iter() {
				let x = collisions.entry(n.0).or_insert(0);
				*x += 1;
			}

			'outer:
			for (p, n) in (&mut pos, &newpos).iter() {
				if *collisions.get(&n.0).unwrap_or(&0) < 2 {
					p.0 = n.0;
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
