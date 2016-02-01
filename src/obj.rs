use std::cell::Cell;
use math::*;
use room::{Action, RoomPhase};
use x1b;

pub trait Obj{
	fn xy(&self) -> (u16, u16);
	fn mv(&mut self, (u16, u16)) -> (u16, u16);
	fn ch(&self) -> char;
	fn tock(&mut self, _: u64, _: char) -> Vec<(Action, Cell<u8>)> { Vec::new() }
	fn delay(&mut self, _: u32) {}
}
pub trait Togo : Obj{
	fn set(&self, bool) -> bool;
	fn isset(&self) -> bool;
	fn toggle(&self) -> bool { self.set(self.isset()) }
}
pub trait Bio : Obj{
	fn dmg(&self) -> bool;
}
pub struct Player{
	pub xy: (u16, u16),
	pub ticks: u32,
}
impl Player {
	pub fn new(xy: (u16, u16)) -> Self{
		Player {
			xy: xy,
			ticks: 0,
		}
	}
}
impl Obj for Player{
	fn xy(&self) -> (u16, u16){
		self.xy
	}
	fn mv(&mut self, xy: (u16, u16)) -> (u16, u16) {
		self.xy = xy;
		xy
	}
	fn ch(&self) -> char{ '@' }
	fn delay(&mut self, t: u32) {
		self.ticks += t
	}
	fn tock(&mut self, id: u64, c: char) -> Vec<(Action, Cell<u8>)> {
		if self.ticks == 1 {
			self.ticks = 0;
			vec![(Action::Reqchar(self.xy.0, self.xy.1), Cell::new(0))]
		}else if self.ticks > 0 {
			self.ticks -= 1;
			Vec::new()
		}else {
			let mut retvec: Vec<(Action, Cell<u8>)> = Vec::new();
			if c == '\0' {
				retvec.push((Action::Reqchar(self.xy.0, self.xy.1), Cell::new(0)));
			}else {
				retvec.push((Action::Clearchar, Cell::new(0)));
				if let Some(d) = ch2dir(c) {
					retvec.push((Action::Step(id, d), Cell::new(0)));
				} else {
					match c {
						'1'...'9' => self.ticks += ((c as u32)-('0' as u32))*10,
						'\x1b' => retvec.push((Action::ExitGame, Cell::new(0))),
						_ => (),
					}
				}
			}
			retvec
		}
	}
}
impl Drop for Player {
	fn drop(&mut self){
		use termios::*;
		x1b::Cursor::dropclear();
		if let Ok(mut term) = Termios::from_fd(0) {
			term.c_lflag |= ECHO;
			tcsetattr(0, TCSAFLUSH, &term);
		}
	}
}

pub struct Wall{
	xy: (u16, u16)
}
impl Wall{
	pub fn new(xy: (u16, u16)) -> Self {
		Wall { xy: xy }
	}
}
impl Obj for Wall{
	fn xy(&self) -> (u16, u16){ self.xy }
	fn mv(&mut self, xy: (u16, u16)) -> (u16, u16) { self.xy = xy; xy }
	fn ch(&self) -> char { '#' }
}

pub struct Portal<T: RoomPhase> {
	xy: (u16, u16),
	rg: T,
}
impl<T: RoomPhase> Portal<T>{
	pub fn new(xy: (u16, u16), rg: T) -> Self {
		Portal{ xy: xy, rg: rg }
	}
}
