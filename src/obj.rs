use std::cell::RefCell;
use math::*;

pub trait Obj{
	fn xy(&self) -> (u16, u16);
	fn mv(&mut self, (u16, u16)) -> (u16, u16);
	fn step(&mut self, d: Dir) -> (u16, u16){
		let (x, y) = self.xy();
		self.mv(match d {
			Dir::E => (x+1, y),
			Dir::NE if y>0 => (x+1, y-1),
			Dir::N if y>0 => (x, y-1),
			Dir::NW if x>0 && y>0 => (x-1, y-1),
			Dir::W if x>0 => (x-1, y),
			Dir::SW if x>0 => (x-1, y+1),
			Dir::S => (x, y+1),
			Dir::SE => (x+1, y+1),
			_ => (x, y)
		})
	}
	fn ch(&self) -> char;
	fn tock(&mut self) -> bool { false }
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
	pub ticks: u32
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
	fn tock(&mut self) -> bool {
		if self.ticks > 0 {
			self.ticks -= 1;
			false
		}else { true }
	}
}
pub struct Wall{
	pub xy: (u16, u16)
}
impl Wall{
	fn new(xy: (u16, u16)) -> Self {
		Wall { xy: xy }
	}
}
impl Obj for Wall{
	fn xy(&self) -> (u16, u16){ self.xy }
	fn mv(&mut self, xy: (u16, u16)) -> (u16, u16) { self.xy = xy; xy }
	fn ch(&self) -> char { '#' }
}
pub struct Room<'a>{
	pub p: Player,
	pub o: Vec<RefCell<Box<Obj + 'a>>>,
	pub w: u16,
	pub h: u16,
	pub t: u64,
}

impl<'a> Room<'a> {
	pub fn tock(&mut self) {
		self.t += 1;
		self.p.tock();
		for o in self.o.iter() {
			o.borrow_mut().tock();
		}
	}
	pub fn new(p: Player) -> Room<'a> {
		Room {
			p: p,
			o: Vec::new(),
			w: 0,
			h: 0,
			t: 0,
		}
	}
}

pub trait RoomPhase {
	fn modify(&self, &mut Room);
}
