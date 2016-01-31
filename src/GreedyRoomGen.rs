use std::cmp;
use std::collections::HashSet;
use rand::*;
use rand::distributions::{IndependentSample, Range};
use obj::*;
use math::*;

pub struct GreedyRoomGen{
	pub rc: u32,
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
		let mut rng = thread_rng();
		let pxy = room.o.get(&0).unwrap().xy();
		let mut rxy: Vec<(u16, u16, u16, u16)> = vec![(if pxy.0 > 0 { pxy.0-1 } else {0}, if pxy.1 > 0 { pxy.1-1 } else {0}, pxy.0+1, pxy.1+1)];
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
					0 if rect.0 > 0 => (rect.0-1, rect.1, rect.2, rect.3),
					1 if rect.1 > 0 => (rect.0, rect.1-1, rect.2, rect.3),
					2 => (rect.0, rect.1, rect.2+1, rect.3),
					3 => (rect.0, rect.1, rect.2, rect.3+1),
					_ => {
						cangrow.push(false);
						continue
					},
				};
				if newrect.2 >= w || newrect.3 >= h {
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
		let mut doors : HashSet<(u16, u16)> = HashSet::new();
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
			fn is1off(a: u16, b: u16) -> bool {
				cmp::max(a, b) - cmp::min(a, b) == 1
			}
			if is1off(r1.0, r2.2) {
				let mny = cmp::max(r1.1, r2.1);
				let mxy = cmp::min(r1.3, r2.3);
				let y = rng.gen_range(mny, mxy);
				doors.insert((r1.0,y));
				doors.insert((r2.2,y));
			}else if is1off(r1.2, r2.0) {
				let mny = cmp::max(r1.1, r2.1);
				let mxy = cmp::min(r1.3, r2.3);
				let y = rng.gen_range(mny, mxy);
				doors.insert((r1.2,y));
				doors.insert((r2.0,y));
			}else if is1off(r1.1, r2.3) {
				let mnx = cmp::max(r1.0, r2.0);
				let mxx = cmp::min(r1.2, r2.2);
				let x = rng.gen_range(mnx, mxx);
				doors.insert((x,r1.1));
				doors.insert((x,r2.3));
			}else if is1off(r1.3, r2.1) {
				let mnx = cmp::max(r1.0, r2.0);
				let mxx = cmp::min(r1.2, r2.2);
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
					room.insert(Box::new(Wall::new((x,xywh.1))));
				}
				if !doors.contains(&(x,xywh.3)) {
					room.insert(Box::new(Wall::new((x,xywh.3))));
				}
			}
			for y in xywh.1..xywh.3+1 {
				if !doors.contains(&(xywh.0,y)) {
					room.insert(Box::new(Wall::new((xywh.0,y))));
				}
				if !doors.contains(&(xywh.2,y)) {
					room.insert(Box::new(Wall::new((xywh.2,y))));
				}
			}
		}
	}
}
