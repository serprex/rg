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
	pub fn getchar(&self, x: i32, y: i32) -> char {
		let xy = (x, y);
		if xy == self.p.xy() { self.p.ch() }
		else { self.o.iter().find(|&o| o.borrow().xy() == xy).map(|o| o.borrow().ch()).unwrap_or(' ') }
	}
	pub fn tock(&mut self) {
		self.t += 1;
		self.p.tock();
		for o in self.o.iter() {
			o.borrow_mut().tock();
		}
	}
}

trait RoomGenerator{
	fn generate(&self, i32, i32) -> Room;
}
struct RogueRoomGen{
	pxy: (i32, i32),
	rc: u32,
}
impl Default for RogueRoomGen {
	fn default() -> Self {
		RogueRoomGen {
			pxy: (3, 3),
			rc: 6,
		}
	}
}
impl RoomGenerator for RogueRoomGen {
	fn generate(&self, w: i32, h: i32) -> Room {
		let betw = Range::new(0, w-2);
		let beth = Range::new(0, h-2);
		let bet4 = Range::new(0, 4);
		let mut rng = rand::thread_rng();
		let mut rxy: Vec<(i32, i32, i32, i32)> = Vec::new();
		while rxy.len() < (self.rc as usize) {
			let rx = betw.ind_sample(&mut rng);
			let ry = beth.ind_sample(&mut rng);
			let candy = (rx, ry, rx+2, ry+2);
			println!("\r{:?}\t{:?}", rxy, candy);
			if !rxy.iter().any(|a| rectover(&candy, a))
				{ rxy.push(candy) }
		}
		loop {
			let mut cangrow: Vec<bool> = Vec::new();
			let b4 = bet4.ind_sample(&mut rng);
			for rect in &rxy {
				let newrect = match b4 {
					0 => (rect.0-1, rect.1, rect.2, rect.3),
					1 => (rect.0, rect.1-1, rect.2, rect.3),
					2 => (rect.0, rect.1, rect.2+1, rect.3),
					3 => (rect.0, rect.1, rect.2, rect.3+1),
					_ => unreachable!(),
				};
				cangrow.push(newrect.0 >= 0 && newrect.1 >= 0 && newrect.2 < w && newrect.3 < h && rxy.iter().filter(|rect2| rectover(&newrect, rect2) ).take(2).count() == 1);
			}
			println!("\r{:?}\t{:?}", cangrow, rxy);
			if cangrow.iter().all(|&x| !x) { break }
			for (&mut (ref mut x1, ref mut y1, ref mut x2, ref mut y2), &grow) in rxy.iter_mut().zip(&cangrow) {
				println!("\r{},{},{},{},{}",x1,y1,x2,y2,grow);
				if !grow { continue }
				match b4 {
					0 => *x1 -= 1,
					1 => *y1 -= 1,
					2 => *x2 += 1,
					3 => *y2 += 1,
					_ => unreachable!(),
				}
				println!("\r{},{},{},{},{}",x1,y1,x2,y2,grow);
			}
		}
		let mut ovec = Vec::<RefCell<Box<Obj>>>::new();
		for xywh in rxy {
			for x in xywh.0..xywh.2+1 {
				ovec.push(RefCell::new(Box::new(Wall { xy: (x,xywh.1) })));
				ovec.push(RefCell::new(Box::new(Wall { xy: (x,xywh.3) })));
			}
			for y in xywh.1..xywh.3+1 {
				ovec.push(RefCell::new(Box::new(Wall { xy: (xywh.0,y) })));
				ovec.push(RefCell::new(Box::new(Wall { xy: (xywh.2,y) })));
			}
		}
		Room {
			p: Player { xy: self.pxy, ticks: 0 },
			o: ovec,
			w: w,
			h: h,
			t: 0,
		}
	}
}

fn prscr<'a>(room: &'a Room){
	let mut chs = HashSet::new();
	clear();
	for y in 0..room.h{
		for x in 0..room.w{
			let ch = room.getchar(x, y);
			addch(ch as u64);
			chs.insert(ch);
		}
		addch('\n' as u64);
	}
	printw(&room.t.to_string());
	let mut y = 0;
	for ch in chs {
		if ch == ' ' { continue }
		mvaddch(y, room.w, ch as u64);
		printw("  ");
		printw(match ch {
			'@' => "You",
			'#' => "Wall",
			_ => "??",
		});
		y += 1
	}
}

fn main(){
	initscr();
	raw();
	noecho();
	let rrg = RogueRoomGen::default();
	let mut room = rrg.generate(40, 40);
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
			}else {
				match c {
					'\x1b' => break,
					_ => (),
				}
			}
		}
	}
	endwin();
}
