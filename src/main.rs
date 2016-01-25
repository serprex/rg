extern crate rand;
extern crate ncurses;
use std::char;
use std::cell::RefCell;
use std::collections::HashSet;
use rand::*;
use rand::distributions::{IndependentSample, Range};
use ncurses::*;

#[derive(Clone, Copy)]
enum Dir { E, NE, N, NW, W, SW, S, SE }
fn ch2dir(c: char) -> Option<Dir> {
	Some(match c {
		'l' => Dir::E,
		'i' => Dir::NE,
		'k' => Dir::N,
		'u' => Dir::NW,
		'h' => Dir::W,
		'n' => Dir::SW,
		'j' => Dir::S,
		'm' => Dir::SE,
		_ => return None,
	})
}
fn dir2i32(d: Dir) -> i32 {
	match d {
		Dir::E => 0,
		Dir::NE => 1,
		Dir::N => 2,
		Dir::NW => 3,
		Dir::W => 4,
		Dir::SW => 5,
		Dir::S => 6,
		Dir::SE => 7,
	}
}
fn isdiag(d: Dir) -> bool {
	match d {
		Dir::E | Dir::N | Dir::W | Dir::S => false,
		Dir::NE | Dir::NW | Dir::SW | Dir::SE => true,
	}
}
fn calcdist2(xy1: (i32, i32), xy2: (i32, i32)) -> i32{
	let (x1, y1) = xy1;
	let (x2, y2) = xy2;
	(x1-x2)*(y1-y2) // TODO calc with diag limitation
}
fn rectover(r1: &(i32,i32,i32,i32), r2: &(i32,i32,i32,i32)) -> bool {
	r1.0 <= r2.2 && r1.2 >= r2.0 && r1.1 <= r2.3 && r1.3 >= r2.1
}
//fn dir2rad(d: Dir) -> f64 {
//	(dir2i32(d) as f64)*3.141592
//}

trait Obj{
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
trait Togo : Obj{
	fn set(&self, bool) -> bool;
	fn isset(&self) -> bool;
	fn toggle(&self) -> bool { self.set(self.isset()) }
}
trait Bio : Obj{
	fn dmg(&self) -> bool;
}
struct Player{
	xy: (i32, i32),
	ticks: i32
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
struct Wall{
	xy: (i32, i32)
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
struct Room<'a>{
	p: Player,
	o: Vec<RefCell<Box<Obj + 'a>>>,
	w: i32,
	h: i32,
	t: u64,
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

trait RoomPhase {
	fn modify(&self, &mut Room);
}
struct GreedyRoomGen{
	rc: u32,
}
impl Default for GreedyRoomGen {
	fn default() -> Self {
		GreedyRoomGen {
			rc: 6,
		}
	}
}
impl RoomPhase for GreedyRoomGen {
	fn modify(&self, room: &mut Room) {
		let w = room.w;
		let h = room.h;
		let betw = Range::new(0, w-2);
		let beth = Range::new(0, h-2);
		let bet4 = Range::new(0, 4);
		let mut rng = rand::thread_rng();
		let pxy = room.p.xy;
		let mut rxy: Vec<(i32, i32, i32, i32)> = vec![(pxy.0-1, pxy.1-1, pxy.0+1, pxy.1+1)];
		let done = &mut [false; 4];
		let mut adjacent = Vec::with_capacity((self.rc*self.rc) as usize);
		for _ in 0..self.rc*self.rc{
			adjacent.push(false)
		}
		while rxy.len() < self.rc as usize {
			let rx = betw.ind_sample(&mut rng);
			let ry = beth.ind_sample(&mut rng);
			let candy = (rx, ry, rx+2, ry+2);
			if !rxy.iter().any(|a| rectover(&candy, a))
				{ rxy.push(candy) }
		}
		loop {
			let mut cangrow: Vec<bool> = Vec::new();
			let b4 = bet4.ind_sample(&mut rng);
			for (i, rect) in rxy.iter().enumerate() {
				let newrect = match b4 {
					0 => (rect.0-1, rect.1, rect.2, rect.3),
					1 => (rect.0, rect.1-1, rect.2, rect.3),
					2 => (rect.0, rect.1, rect.2+1, rect.3),
					3 => (rect.0, rect.1, rect.2, rect.3+1),
					_ => unreachable!(),
				};
				if !(newrect.0 >= 0 && newrect.1 >= 0 && newrect.2 < w && newrect.3 < h) {
					cangrow.push(false);
					continue
				}
				let mut grow = true;
				for (j, rect2) in rxy.iter().enumerate() {
					if i != j && rectover(&newrect, rect2){
						grow = false;
						adjacent[i+j*(self.rc as usize)] = true;
						adjacent[i*(self.rc as usize)+j] = true;
					}
				}
				cangrow.push(grow)
			}
			if cangrow.iter().all(|&x| !x) {
				done[b4] = true;
				if done.iter().all(|&x| x) { break }
			}
			for (&mut (ref mut x1, ref mut y1, ref mut x2, ref mut y2), &grow) in rxy.iter_mut().zip(&cangrow) {
				if !grow { continue }
				match b4 {
					0 => *x1 -= 1,
					1 => *y1 -= 1,
					2 => *x2 += 1,
					3 => *y2 += 1,
					_ => unreachable!(),
				}
			}
		}
		let mut doors : HashSet<(i32, i32)> = HashSet::new();
		let mut iszgrp : Vec<bool> = Vec::with_capacity(self.rc as usize);
		let mut nzgrps : HashSet<u32> = HashSet::with_capacity((self.rc-1) as usize);
		let mut zgrps : HashSet<u32> = HashSet::with_capacity(self.rc as usize);
		zgrps.insert(0);
		iszgrp.push(true);
		for i in 1..self.rc {
			nzgrps.insert(i);
			iszgrp.push(false);
		}
		loop {
			let &iszi = rng.choose(&zgrps.iter().cloned().collect::<Vec<_>>()).unwrap();
			let mut adjs = Vec::new();
			for i in 0..self.rc{
				if adjacent[(i+iszi*self.rc) as usize] { adjs.push(i) }
			}
			if adjs.is_empty() { break }
			let &aidx = rng.choose(&adjs.iter().cloned().collect::<Vec<_>>()).unwrap();
			let r1 = rxy[iszi as usize];
			let r2 = rxy[aidx as usize];
			// TODO don't put doors in unreachable corners
			if (r1.0-r2.2).abs() == 1 {
				let mny = std::cmp::max(r1.1, r2.1);
				let mxy = std::cmp::min(r1.3, r2.3);
				let y = rng.gen_range(mny, mxy);
				doors.insert((r1.0,y));
				doors.insert((r2.2,y));
			}else if (r1.2-r2.0).abs() == 1 {
				let mny = std::cmp::max(r1.1, r2.1);
				let mxy = std::cmp::min(r1.3, r2.3);
				let y = rng.gen_range(mny, mxy);
				doors.insert((r1.2,y));
				doors.insert((r2.0,y));
			}else if (r1.1-r2.3).abs() == 1 {
				let mnx = std::cmp::max(r1.0, r2.0);
				let mxx = std::cmp::min(r1.2, r2.2);
				let x = rng.gen_range(mnx, mxx);
				doors.insert((x,r1.1));
				doors.insert((x,r2.3));
			}else if (r1.3-r2.1).abs() == 1 {
				let mnx = std::cmp::max(r1.0, r2.0);
				let mxx = std::cmp::min(r1.2, r2.2);
				let x = rng.gen_range(mnx, mxx);
				doors.insert((x,r1.3));
				doors.insert((x,r2.1));
			}else { unreachable!() }
			iszgrp[aidx as usize] = true;
			zgrps.insert(aidx);
			nzgrps.remove(&aidx);
			if nzgrps.is_empty() { break }
		}
		for xywh in rxy {
			room.o.reserve(((xywh.2-xywh.0+xywh.3-xywh.1+2)*2) as usize);
			for x in xywh.0..xywh.2+1 {
				if !doors.contains(&(x,xywh.1)) {
					room.o.push(RefCell::new(Box::new(Wall { xy: (x,xywh.1) })))
				}
				if !doors.contains(&(x,xywh.3)) {
					room.o.push(RefCell::new(Box::new(Wall { xy: (x,xywh.3) })))
				}
			}
			for y in xywh.1..xywh.3+1 {
				if !doors.contains(&(xywh.0,y)) {
					room.o.push(RefCell::new(Box::new(Wall { xy: (xywh.0,y) })));
				}
				if !doors.contains(&(xywh.2,y)) {
					room.o.push(RefCell::new(Box::new(Wall { xy: (xywh.2,y) })));
				}
			}
		}
	}
}

fn prscr<'a>(room: &'a Room){
	let mut chs = HashSet::new();
	clear();
	for o in room.o.iter() {
		let ob = o.borrow();
		let xy = ob.xy();
		let ch = ob.ch();
		mvaddch(xy.1, xy.0, ch as u64);
		chs.insert(ch);
	}
	mvaddch(room.p.xy().1, room.p.xy().0, '@' as u64);
	let mut y = 0;
	for ch in chs {
		mvaddch(y, room.w+1, ch as u64);
		mvaddstr(y, room.w+3, match ch {
			'#' => "Wall",
			_ => "??",
		});
		y += 1
	}
	mvaddstr(room.h+1, 0, &room.t.to_string());
}

struct NCurse;
impl NCurse {
	pub fn rungame(&self){
		initscr();
		raw();
		noecho();
		let mut room = Room {
			w: 60,
			h: 40,
			.. Room::new(Player { xy: (3, 3), ticks: 0 })
		};
		let rrg = GreedyRoomGen::default();
		rrg.modify(&mut room);
		loop{
			room.tock();
			if room.p.ticks == 0 {
				prscr(&room);
				refresh();
				let c: char = char::from_u32(getch() as u32).unwrap();
				if let Some(d) = ch2dir(c) {
					room.p.step(d);
					room.p.ticks = if isdiag(d) { 141 }
						else { 100 };
				} else {
					match c {
						'1'...'9' => room.p.ticks = ((c as i32)-('0' as i32))*10,
						'\x1b' => break,
						_ => (),
					}
				}
			}
		}
	}
}
impl Drop for NCurse {
	fn drop(&mut self){
		endwin();
	}
}

fn main(){
	NCurse.rungame();
}
