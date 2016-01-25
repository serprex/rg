extern crate rand;
extern crate ncurses;

mod GreedyRoomGen;
mod obj;
mod math;

use std::char;
use std::collections::HashSet;
use ncurses::*;
use obj::*;
use math::*;

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
		let mut room = obj::Room {
			w: 60,
			h: 40,
			.. Room::new(Player { xy: (3, 3), ticks: 0 })
		};
		let rrg = GreedyRoomGen::GreedyRoomGen::default();
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
