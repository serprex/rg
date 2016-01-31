extern crate rand;
extern crate termios;
extern crate x1b;
#[macro_use]
extern crate bitflags;

mod GreedyRoomGen;
mod obj;
mod math;

use obj::{Obj, RoomPhase};

pub fn raw(fd: i32) -> std::io::Result<()> {
	use termios::*;
	let mut term = try!(Termios::from_fd(fd));
	cfmakeraw(&mut term);
	term.c_lflag &= !ECHO;
	tcsetattr(fd, TCSANOW, &term)
}

fn main(){
	raw(0);
	print!("\x1bc\x1b[?25l");
	let mut room = obj::Room::new(obj::Player::new((3, 3)), 60, 40);
	let rrg = GreedyRoomGen::GreedyRoomGen::default();
	rrg.modify(&mut room);
	while room.tock() {}
}
