use std::cell::{Cell, RefCell};
use std::collections::{HashSet, HashMap};
use std::io;
use std::io::Read;
use math::*;
use obj::*;
use x1b;

#[derive(Debug)]
pub enum Action {
	Step(u64, Dir),
	Reqchar(u16, u16),
	Clearchar,
	ExitGame,
	NextRoom,
	Slap(u64),
	Remove(u64),
}

pub struct Room<'a>{
	pub oi: u64,
	pub t: u64,
	pub o: HashMap<u64, Box<Obj + 'a>>,
	pub a: Vec<(Action, Cell<u8>)>,
	pub ch: char,
	pub w: u16,
	pub h: u16,
	curse: RefCell<x1b::Curse>,
}

impl<'a> Room<'a>{
	fn prscr(&self, px: u16, py: u16) -> io::Result<()> {
		let mut chs = HashSet::new();
		let mut curse = self.curse.borrow_mut();
		let mut walls = HashSet::new();
		let mut hasdrawn = HashMap::new();
		for (_, o) in self.o.iter() {
			let xy = o.xy();
			let ch = o.ch();
			if ch == '#' {
				walls.insert(xy);
			}else {
				hasdrawn.insert(xy, ch);
			}
		}
		let mut scan = Vec::new();
		for &d in [Dir::NW, Dir::N, Dir::NE, Dir::E, Dir::SE, Dir::S, Dir::SW, Dir::W].into_iter() {
			let (x, y) = step((px, py), d);
			scan.push(if (x != px || y != py) && x < self.w && y < self.h && !walls.contains(&(x,y)) {
				match hasdrawn.get(&(x,y)) {
					None => curse.set(x, y, x1b::TCell::from_char('.')),
					Some(&ch) => {
						curse.set(x, y, x1b::TCell::from_char(ch));
						chs.insert(ch);
					}
				}
				true
			} else {
				if walls.contains(&(x,y)) {
					curse.set(x, y, x1b::TCell::from_char('#'))
				}
				false
			});
		}
		fn findnxy(px: u16, py: u16, w: u16, h: u16, mut n: u16, r: u16) -> (u16, u16) {
			fn retsome(x: i32, y: i32, w: u16, h: u16) -> (u16, u16) {
				if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 { (65535, 65535) }
				else { (x as u16, y as u16) }
			}
			let mut ox = (px as i32)-(r as i32);
			let mut oy = (py as i32)-(r as i32);
			let l = r*2;
			for i in 0..4 {
				for _ in 0..l {
					if n == 0 { return retsome(ox, oy, w, h) }
					match i {
						0 => ox += 1,
						1 => oy += 1,
						2 => ox -= 1,
						3 => oy -= 1,
						_ => unreachable!()
					}
					n -= 1;
				}
			}
			retsome(ox, oy, w, h)
		}
		let mut n = 2;
		loop {
			let mut nextscan = Vec::new();
			{
			let scanfirst = scan.first().unwrap().clone();
			let scanlast = scan.last().unwrap().clone();
			let scanlastfirst = [scanlast, scanfirst];
			let mut scandows = scan.windows(2);
			let mut dow: &[bool] = &[];
			for i in 0..8*n {
				let d01 = if i%n != 1 {
					dow = match scandows.next() {
						Some(scandow) => scandow,
						None => &scanlastfirst,
					};
					dow[0] && (i%n == 0 || dow[1])
				} else { dow[0] && dow[1] };
				nextscan.push(d01 && match findnxy(px, py, self.w, self.h, i, n) {
					(65535, 65535) => false,
					nxy => {
						let visi = !walls.contains(&nxy);
						if visi {
							match hasdrawn.get(&nxy) {
								None => curse.set(nxy.0, nxy.1, x1b::TCell::from_char('.')),
								Some(&ch) => {
									curse.set(nxy.0, nxy.1, x1b::TCell::from_char(ch));
									chs.insert(ch);
								}
							}
						} else {
							curse.set(nxy.0, nxy.1, x1b::TCell::from_char('#'));
						}
						visi
					},
				});
			}
			}
			if !nextscan.iter().any(|&x| x) { break }
			scan = nextscan;
			n += 1;
		}
		curse.set(px, py, x1b::TCell::from_char('@'));
		let mut y = 0;
		for ch in chs {
			curse.set(self.w+1, y, x1b::TCell::from_char(ch));
			curse.printnows(self.w+3, y, match ch {
				'>' => "Stairs",
				_ => "??",
			}, x1b::TextAttr::empty());
			y += 1
		}
		curse.printnows(0, self.h+1, &self.t.to_string(), x1b::TextAttr::empty());
		curse.perframe_refresh_then_clear(x1b::TCell::from_char(' '))
	}

	pub fn tock(&mut self) -> bool {
		self.t += 1;
		let mut newacts = Vec::new();
		for (&oid, o) in self.o.iter_mut() {
			newacts.extend(o.tock(oid, self.ch))
		}
		self.a.extend(newacts);
		let mut rmacts = Vec::new();
		let mut grr = false;
		let mut slapvec = Vec::new();
		let mut rmobj = Vec::new();
		for (aidx, &(ref a, ref t)) in self.a.iter().enumerate() {
			let tval = t.get();
			if tval == 0 {
				rmacts.push(aidx);
				match a {
					&Action::Step(src, dir) => {
						let xy: (u16, u16);
						if let Some(o) = self.o.get(&src) {
							xy = step(o.xy(), dir);
						}else { continue }
						let canmove = self.o.iter().all(|(_, o)| o.ch() != '#' || o.xy() != xy);
						if canmove {
							let om = self.o.get_mut(&src).unwrap();
							om.mv(xy);
							om.delay(if isdiag(dir) { 141 } else { 100 })
						}
					},
					&Action::Reqchar(x, y) => {
						if self.ch == '\0' {
							self.prscr(x, y);
							let stdin = io::stdin();
							let sin = stdin.lock();
							let mut sinchars = sin.bytes();
							self.ch = sinchars.next().unwrap().unwrap() as char;
						}
					},
					&Action::Clearchar => self.ch = '\0',
					&Action::ExitGame => return false,
					&Action::Remove(oid) => rmobj.push(oid),
					&Action::NextRoom => grr = true,
					&Action::Slap(src) => {
						let xy: (u16, u16);
						if let Some(o) = self.o.get(&src) {
							xy = o.xy()
						} else { continue }
						for (&oid, o) in self.o.iter() {
							if o.xy() == xy && oid != src {
								slapvec.push(oid)
							}
						}
					}
				}
			} else { t.set(tval-1) }
		}
		for aidx in rmacts.into_iter().rev() {
			self.a.remove(aidx);
		}
		for dst in slapvec.into_iter() {
			let o = if let Some(o) = self.o.get_mut(&dst)
				{ o } else { continue };
			o.slap();
		}
		for oid in rmobj.into_iter() {
			self.o.remove(&oid);
		}
		if grr {
			use genroom_greedy::*;
			let mut rmvec = Vec::new();
			for (&oid, o) in self.o.iter() {
				if o.ch() == '#' { rmvec.push(oid) }
			}
			for oid in rmvec.into_iter() {
				self.o.remove(&oid);
			}
			let grr = GreedyRoomGen::default();
			grr.modify(self);
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
			curse: RefCell::new(x1b::Curse::new(w+12, h+2)),
		}
	}
}

pub trait RoomPhase {
	fn modify(&self, &mut Room);
}

