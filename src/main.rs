extern crate rand;
extern crate termios;
#[macro_use]
extern crate bitflags;

mod x1b;
mod GreedyRoomGen;
mod obj;
mod math;

use obj::{Obj, RoomPhase};

use std::collections::HashSet;
use std::io::Read;

fn prscr<'a>(curse: &'a mut x1b::Cursor, room: &'a obj::Room) -> std::io::Result<usize> {
	let mut chs = HashSet::new();
	curse.eraseall();
	curse.resetxy();
	for o in room.o.iter() {
		let ob = o.borrow();
		let xy = ob.xy();
		let ch = ob.ch();
		curse.mv(xy.0 as u16+1, xy.1 as u16+1);
		curse.prchr(ch);
		//mvaddch(xy.1, xy.0, ch as u64);
		chs.insert(ch);
	}
	let (px, py) = room.p.xy();
	curse.mv(px as u16+1, py as u16+1);
	curse.prchr('@');
	curse.sety(1);
	for ch in chs {
		curse.setx(room.w as u16+2);
		curse.print(&format!("{} {}", ch, match ch {
			'#' => "Wall",
			_ => "??",
		}));
		curse.down1();
	}
	curse.mv(1, room.h as u16+2);
	curse.print(&room.t.to_string());
	curse.flush()
}

#[derive(Default)]
pub struct NCurse{
	curse: x1b::Cursor,
}
impl NCurse {
	pub fn raw() -> std::io::Result<()> {
		use termios::*;
		let mut term = try!(Termios::from_fd(0));
		cfmakeraw(&mut term);
		term.c_lflag &= !ECHO;
		tcsetattr(0, TCSANOW, &term);
		Ok(())
	}

	pub fn rungame(&mut self){
		NCurse::raw();
		let stdin = std::io::stdin();
		let sin = stdin.lock();
		let mut sinchars = sin.bytes();
		let mut room = obj::Room {
			w: 60,
			h: 40,
			.. obj::Room::new(obj::Player { xy: (3, 3), ticks: 0 })
		};
		let rrg = GreedyRoomGen::GreedyRoomGen::default();
		rrg.modify(&mut room);
		loop{
			room.tock();
			if room.p.ticks == 0 {
				prscr(&mut self.curse, &room);
				let c = sinchars.next().unwrap().unwrap() as char;
				if let Some(d) = math::ch2dir(c) {
					room.p.step(d);
					room.p.ticks = if math::isdiag(d) { 141 }
						else { 100 };
				} else {
					match c {
						'1'...'9' => room.p.ticks = ((c as i32)-('0' as i32))*10,
						'\x1b' => return,
						_ => (),
					}
				}
			}
		}
	}
}
impl Drop for NCurse {
	fn drop(&mut self){
		use termios::*;
		x1b::Cursor::dropclear();
		if let Ok(mut term) = Termios::from_fd(0) {
			term.c_lflag |= ECHO;
			tcsetattr(0, TCSAFLUSH, &term);
		}
	}
}

fn main(){
	NCurse::default().rungame();
}
