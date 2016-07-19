use std::cmp;
use std::collections::HashSet;
use rand::*;
use rand::distributions::{IndependentSample, Range};
use math::*;
use specs::World;
use ::WallComp;

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
impl GreedyRoomGen {
	pub fn modify(&self, w: u16, h: u16, pxy: [u16; 2], room: &mut World) {
		let betw = Range::new(0, w-2);
		let beth = Range::new(0, h-2);
		let bet4 = Range::new(0, 4);
		let mut rng = thread_rng();
		let mut rxy: Vec<(u16, u16, u16, u16)> =
			vec![(if pxy[0] > 0 { pxy[0]-1 } else {0},
				  if pxy[1] > 0 { pxy[1]-1 } else {0},
				  pxy[0]+1, pxy[1]+1)];
		let done = &mut [false; 4];
		let mut adjacent = vec![false; (self.rc*self.rc) as usize];
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
					if i != j && rectoverinc(&newrect, rect2){
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
		let mut doors: HashSet<(u16, u16)> = HashSet::new();
		let mut nzgrps: HashSet<u32> = (1..self.rc).into_iter().collect();
		let mut zgrps: HashSet<u32> = HashSet::with_capacity(self.rc as usize);
		zgrps.insert(0);
		loop {
			let nthzi = rng.gen_range(0, zgrps.len());
			let &iszi = zgrps.iter().skip(nthzi).next().unwrap();
			let mut adjs = Vec::with_capacity(self.rc as usize);
			for i in 0..self.rc{
				if adjacent[(i+iszi*self.rc) as usize] { adjs.push(i) }
			}
			if adjs.is_empty() { break }
			let &aidx = rng.choose(&adjs).unwrap();
			let r1 = rxy[iszi as usize];
			let r2 = rxy[aidx as usize];
			if r1.0 == r2.2 {
				let mn = cmp::max(r1.1, r2.1)+1;
				let mx = cmp::min(r1.3, r2.3);
				if mn == mx { continue }
				let y = rng.gen_range(mn, mx);
				doors.insert((r1.0,y));
			}else if r1.2 == r2.0 {
				let mn = cmp::max(r1.1, r2.1)+1;
				let mx = cmp::min(r1.3, r2.3);
				if mn == mx { continue }
				let y = rng.gen_range(mn, mx);
				doors.insert((r1.2,y));
			}else if r1.1 == r2.3 {
				let mn = cmp::max(r1.0, r2.0)+1;
				let mx = cmp::min(r1.2, r2.2);
				if mn == mx { continue }
				let x = rng.gen_range(mn, mx);
				doors.insert((x,r1.1));
			}else if r1.3 == r2.1 {
				let mn = cmp::max(r1.0, r2.0)+1;
				let mx = cmp::min(r1.2, r2.2);
				if mn == mx { continue }
				let x = rng.gen_range(mn, mx);
				doors.insert((x,r1.3));
			}else { unreachable!() }
			zgrps.insert(aidx);
			nzgrps.remove(&aidx);
			if nzgrps.is_empty() {
				/*let r = rxy[aidx as usize];
				let x = rng.gen_range(r.0+1, r.2);
				let y = rng.gen_range(r.1+1, r.3);
				room.insert(Obj::new_portal((x,y)));*/
				break
			}
		}
		for xywh in rxy {
			for x in xywh.0..xywh.2+1 {
				if !doors.contains(&(x,xywh.1)) {
					room.create_now().with(WallComp([x as i16,xywh.1 as i16]));
				}
				if !doors.contains(&(x,xywh.3)) {
					room.create_now().with(WallComp([x as i16,xywh.3 as i16]));
				}
			}
			for y in xywh.1..xywh.3+1 {
				if !doors.contains(&(xywh.0,y)) {
					room.create_now().with(WallComp([xywh.0 as i16,y as i16]));
				}
				if !doors.contains(&(xywh.2,y)) {
					room.create_now().with(WallComp([xywh.2 as i16,y as i16]));
				}
			}
		}
	}
}
