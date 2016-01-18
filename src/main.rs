extern crate ncurses;
use std::char;
use std::cell::RefCell;
use std::collections::HashSet;
use ncurses::*;

#[derive(Clone, Copy)]
enum Dir { E, NE, N, NW, W, SW, S, SE }
enum Action {
	Move(Dir),
	Aim(Dir),
}
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
impl Obj for Wall{
	fn xy(&self) -> (i32, i32){ self.xy }
	fn mv(&mut self, xy: (i32, i32)) -> (i32, i32) { self.xy = xy; xy }
	fn ch(&self) -> char { '#' }
}
struct Room<'a>{
	p: &'a mut Player,
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
	pub fn tock(&mut self) -> () {
		self.t += 1;
		self.p.tock();
		for o in self.o.iter() {
			o.borrow_mut().tock();
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
	let mut p = Player { xy: (2, 2), ticks: 0 };
	let mut room = Room {
		p: &mut p,
		o: vec![
			RefCell::new(Box::new(Wall {xy: (3,3)})),
			RefCell::new(Box::new(Wall {xy: (3,4444)}))],
		w: 40,
		h: 40,
		t: 0,
	};
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
