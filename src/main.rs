extern crate rand;
extern crate termios;
#[macro_use]
extern crate bitflags;

mod x1b;
mod GreedyRoomGen;
mod obj;
mod math;

use obj::{Obj, RoomPhase};

pub fn raw(fd: i32) -> std::io::Result<()> {
	use termios::*;
	let mut term = try!(Termios::from_fd(fd));
	cfmakeraw(&mut term);
	term.c_lflag &= !ECHO;
	tcsetattr(fd, TCSANOW, &term);
	Ok(())
}

fn main(){
	raw(0);
	let mut room = obj::Room {
		w: 60,
		h: 40,
		.. obj::Room::new(obj::Player::new((3, 3)))
	};
	let rrg = GreedyRoomGen::GreedyRoomGen::default();
	rrg.modify(&mut room);
	while room.tock() {}
}
