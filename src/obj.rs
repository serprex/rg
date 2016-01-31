use std;
use std::io::Read;
use std::collections::{HashSet, HashMap};
use std::cell::Cell;
use std::borrow::Borrow;
use math::*;
use x1b;

pub fn stepmath((x, y): (u16, u16), d: Dir) -> (u16, u16) {
	match d {
		Dir::E => (x+1, y),
		Dir::NE if y>0 => (x+1, y-1),
		Dir::N if y>0 => (x, y-1),
		Dir::NW if x>0 && y>0 => (x-1, y-1),
		Dir::W if x>0 => (x-1, y),
		Dir::SW if x>0 => (x-1, y+1),
		Dir::S => (x, y+1),
		Dir::SE => (x+1, y+1),
		_ => (x, y)
	}
}

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
			vec![(Action::Reqchar, Cell::new(0))]
		}else if self.ticks > 0 {
			self.ticks -= 1;
			Vec::new()
		}else {
			let mut retvec: Vec<(Action, Cell<u8>)> = Vec::new();
			if c == '\0' {
				retvec.push((Action::Reqchar, Cell::new(0)));
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

#[derive(Debug)]
pub enum Action {
	Step(u64, Dir),
	Reqchar,
	Clearchar,
	ExitGame,
}

pub struct Room<'a>{
	pub oi: u64,
	pub t: u64,
	pub o: HashMap<u64, Box<Obj + 'a>>,
	pub a: Vec<(Action, Cell<u8>)>,
	pub ch: char,
	pub w: u16,
	pub h: u16,
	curse: std::cell::RefCell<x1b::Curse>,
}

impl<'a> Room<'a>{
	fn prscr(&self) -> std::io::Result<()> {
		let mut chs = HashSet::new();
		let mut curse = self.curse.borrow_mut();
		curse.clear(x1b::TCell::from_char(' '));
		for (_, o) in self.o.iter() {
			let xy = o.xy();
			let ch = o.ch();
			curse.setxy(xy.0, xy.1, x1b::TCell::from_char(ch));
			chs.insert(ch);
		}
		let mut y = 0;
		for ch in chs {
			curse.setxy(self.w+2, y, x1b::TCell::from_char(ch));
			curse.printxy(self.w+4, y, match ch {
				'#' => "Wall",
				'@' => "Rogue",
				_ => "??",
			}, x1b::TextAttr::empty());
			y += 1
		}
		curse.printxy(1, self.h+2, &self.t.to_string(), x1b::TextAttr::empty());
		curse.refresh()
	}

	pub fn tock(&mut self) -> bool {
		self.t += 1;
		let mut newacts = Vec::new();
		for (&oid, o) in self.o.iter_mut() {
			let a = o.tock(oid, self.ch);
			newacts.extend(a)
		}
		self.a.extend(newacts);
		let mut rmacts = Vec::new();
		for (aidx, &(ref a, ref t)) in self.a.iter().enumerate() {
			let tval = t.get();
			if tval == 0 {
				rmacts.push(aidx);
				match a {
					&Action::Step(src, dir) => {
						let xy: (u16, u16);
						if let Some(o) = self.o.get(&src) {
							xy = stepmath(o.xy(), dir);
						}else { continue }
						let canmove = self.o.iter().all(|(oid, o)| o.xy() != xy);
						if canmove {
							let om = self.o.get_mut(&src).unwrap();
							om.mv(xy);
							om.delay(if isdiag(dir) { 141 } else { 100 })
						}
					},
					&Action::Reqchar => {
						if self.ch == '\0' {
							self.prscr();
							let stdin = std::io::stdin();
							let sin = stdin.lock();
							let mut sinchars = sin.bytes();
							self.ch = sinchars.next().unwrap().unwrap() as char;
						}
					},
					&Action::Clearchar => {
						self.ch = '\0'
					},
					&Action::ExitGame => return false
				}
			} else { t.set(tval-1) }
		}
		for aidx in rmacts.into_iter().rev() {
			self.a.remove(aidx);
		}
		true
	}

	pub fn insert(&mut self, o: Box<Obj + 'a>) -> u64 {
		while self.o.contains_key(&self.oi) { self.oi += 1; }
		self.o.insert(self.oi, o);
		self.oi
	}

	pub fn new(p: Player, w: u16, h: u16) -> Room<'a> {
		let mut o: HashMap<u64, Box<Obj + 'a>> = HashMap::new();
		o.insert(0, Box::new(p));
		Room {
			oi: 1,
			t: 0,
			o: o,
			a: Vec::new(),
			ch: '\0',
			w: w,
			h: h,
			curse: std::cell::RefCell::new(x1b::Curse::new(w, h)),
		}
	}
}

pub trait RoomPhase {
	fn modify(&self, &mut Room);
}
