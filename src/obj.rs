use std::cell::Cell;
use math::*;
use room::Action;

pub struct Stats {
	pub mxhp: u16,
	pub hp: u16,
	pub atk: u16,
	pub dex: u16,
}

pub trait Obj{
	fn xy(&self) -> (u16, u16);
	fn z(&self) -> i16 { 0 }
	fn mv(&mut self, (u16, u16)) -> (u16, u16);
	fn ch(&self) -> char;
	fn tock(&mut self, _: u64, _: char) -> Vec<(Action, Cell<u8>)> { Vec::new() }
	fn delay(&mut self, _: u32) {}
	fn stats(&self) -> Option<&Stats> { None }
	fn stats_mut(&mut self) -> Option<&mut Stats> { None }
	fn slap(&mut self) {}
}
pub struct Player{
	xy: (u16, u16),
	stats: Stats,
	ticks: u32,
}
impl Player {
	pub fn new(xy: (u16, u16)) -> Self{
		Player {
			xy: xy,
			stats: Stats{
				mxhp: 4,
				hp: 2,
				atk: 4,
				dex: 4,
			},
			ticks: 0,
		}
	}
}
impl Obj for Player{
	fn xy(&self) -> (u16, u16){
		self.xy
	}
	fn z(&self) -> i16 {
		8192
	}
	fn mv(&mut self, xy: (u16, u16)) -> (u16, u16) {
		self.xy = xy;
		xy
	}
	fn ch(&self) -> char{ '@' }
	fn delay(&mut self, t: u32) {
		self.ticks += t
	}
	fn stats(&self) -> Option<&Stats> { Some(&self.stats) }
	fn stats_mut(&mut self) -> Option<&mut Stats> { Some(&mut self.stats) }
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
						',' => {
							retvec.push((Action::Slap(id), Cell::new(8)));
							self.ticks += 24;
						}
						'\x1b' => retvec.push((Action::ExitGame, Cell::new(0))),
						_ => (),
					}
				}
			}
			retvec
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

pub struct Portal {
	xy: (u16, u16),
	on: bool,
}
impl Portal{
	pub fn new(xy: (u16, u16)) -> Self {
		Portal{ xy: xy, on: false }
	}
}
impl Obj for Portal{
	fn xy(&self) -> (u16, u16) { self.xy }
	fn mv(&mut self, xy: (u16, u16)) -> (u16, u16) { self.xy = xy; xy }
	fn ch(&self) -> char { '>' }
	fn slap(&mut self) {
		self.on = true;
	}
	fn tock(&mut self, id: u64, _: char) -> Vec<(Action, Cell<u8>)> {
		if self.on {
			vec![(Action::Remove(id), Cell::new(0)), (Action::NextRoom, Cell::new(0))]
		} else { Vec::new() }
	}
}

pub struct Rat {
	xy: (u16, u16),
	stats: Stats,
	ticks: u32,
}
