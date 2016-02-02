extern crate rand;
extern crate termios;
extern crate x1b;
#[macro_use]
extern crate bitflags;

mod genroom_greedy;
mod math;
mod obj;
mod room;

use room::RoomPhase;

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

fn main(){
	let rt = TermJuggler::new();
	let mut room = room::Room::new(obj::Player::new((3, 3)), 60, 40);
	let rrg = genroom_greedy::GreedyRoomGen::default();
	rrg.modify(&mut room);
	while room.tock() {}
	rt.end()
}
