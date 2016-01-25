use std::cell::RefCell;
use math::*;

pub trait Obj{
	fn xy(&self) -> (i32, i32);
	fn mv(&mut self, (i32, i32)) -> (i32, i32);
	fn step(&mut self, d: Dir) -> (i32, i32){
		let (x, y) = self.xy();
		self.mv(match d {
			Dir::E => (x+1, y),
			Dir::NE => (x+1, y-1),
			Dir::N => (x, y-1),
			Dir::NW => (x-1, y-1),
			Dir::W => (x-1, y),
			Dir::SW => (x-1, y+1),
			Dir::S => (x, y+1),
			Dir::SE => (x+1, y+1),
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
	pub xy: (i32, i32),
	pub ticks: i32
}
impl Obj for Player{
	fn xy(&self) -> (i32, i32){
		self.xy
	}
	fn mv(&mut self, xy: (i32, i32)) -> (i32, i32) {
		self.xy = xy;
		xy
	}
	fn ch(&self) -> char{ '@' }
	fn tock(&mut self) -> bool {
		if self.ticks > 0 { self.ticks -= 1 }
		self.ticks == 0
	}
}
pub struct Wall{
	pub xy: (i32, i32)
}
impl Wall{
	fn new(xy: (i32, i32)) -> Self {
		Wall { xy: xy }
	}
}
impl Obj for Wall{
	fn xy(&self) -> (i32, i32){ self.xy }
	fn mv(&mut self, xy: (i32, i32)) -> (i32, i32) { self.xy = xy; xy }
	fn ch(&self) -> char { '#' }
}
pub struct Room<'a>{
	pub p: Player,
	pub o: Vec<RefCell<Box<Obj + 'a>>>,
	pub w: i32,
	pub h: i32,
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
