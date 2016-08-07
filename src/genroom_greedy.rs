use std::cmp;
use std::collections::HashSet;
use rand::*;
use rand::distributions::{IndependentSample, Range};
use math::*;
use specs::World;

use components::*;
use super::FnvHashSet;

pub struct GreedyRoomGen(usize);
impl Default for GreedyRoomGen {
	fn default() -> Self {
		GreedyRoomGen(6)
	}
}
impl GreedyRoomGen {
	pub fn new(rc: usize) -> Self {
		GreedyRoomGen(rc)
	}

	pub fn modify(&self, w: i16, h: i16, pxy: [i16; 2], room: &mut World) {
		let rc = self.0;
		let betwh = Range::new(0, (w-2)*(h-2));
		let bet4 = Range::new(0, 4);
		let mut rng = thread_rng();
		let mut rxy =
			vec![[cmp::max(pxy[0]-1, 0),
				cmp::max(pxy[1]-1, 0), pxy[0]+1, pxy[1]+1]];
		let done = &mut [false; 4];
		let mut adjacent = vec![false; rc*rc];
		while rxy.len() < rc {
			let xy = betwh.ind_sample(&mut rng);
			let (rx, ry) = (xy % (w-2), xy / (h-2));
			let candy = [rx, ry, rx+2, ry+2];
			if !rxy.iter().any(|&a| rectover(candy, a))
				{ rxy.push(candy) }
		}
		loop {
			let mut cangrow: Vec<bool> = Vec::with_capacity(rc);
			let b4 = bet4.ind_sample(&mut rng);
			for (i, rect) in rxy.iter().enumerate() {
				let mut newrect = *rect;
				let oob = match b4 {
					0|1 => {
						newrect[b4] -= 1;
						newrect[b4] < 0
					},
					2|3 => {
						newrect[b4] += 1;
						newrect[b4] >= if b4 == 2 { w } else { h }
					},
					_ => unreachable!(),
				};
				if oob {
					cangrow.push(false);
					continue
				}
				let mut grow = true;
				for (j, rect2) in rxy.iter().enumerate() {
					if i != j && rectoverinc(newrect, *rect2){
						grow = false;
						adjacent[i+j*rc] = true;
						adjacent[i*rc+j] = true;
					}
				}
				cangrow.push(grow)
			}
			if cangrow.iter().all(|&x| !x) {
				done[b4] = true;
				if done.iter().all(|&x| x) { break }
			}
			for (ref mut xywh, &grow) in rxy.iter_mut().zip(&cangrow) {
				if !grow { continue }
				xywh[b4] += match b4 {
					0|1 => -1,
					2|3 => 1,
					_ => unreachable!(),
				}
			}
		}
		let mut doors: FnvHashSet<[i16; 2]> = Default::default();
		let mut nzgrps: FnvHashSet<usize> = (1..rc).into_iter().collect();
		let mut zgrps: FnvHashSet<usize> = HashSet::with_capacity_and_hasher(rc, Default::default());
		zgrps.insert(0);
		'doorloop:
		loop {
			let nthzi = rng.gen_range(0, zgrps.len());
			let &iszi = zgrps.iter().skip(nthzi).next().unwrap();
			let adjs = (0..rc).into_iter()
				.filter(|i| adjacent[i+iszi*rc])
				.collect::<Vec<_>>();
			if adjs.is_empty() { break }
			let &aidx = rng.choose(&adjs).unwrap();
			let r1 = rxy[iszi];
			let r2 = rxy[aidx];
			for &(r1i, r2i, mxi, mni) in &[(0, 2, 1, 3), (2, 0, 1, 3), (1, 3, 0, 2), (3, 1, 0, 2)] {
				if r1[r1i] == r2[r2i] {
					let mn = cmp::max(r1[mxi], r2[mxi])+1;
					let mx = cmp::min(r1[mni], r2[mni]);
					if mn == mx { continue 'doorloop }
					let mnx = rng.gen_range(mn, mx);
					doors.insert(if mxi == 1 { [r1[r1i],mnx] } else { [mnx,r1[r1i]] });
					break
				}
			}
			zgrps.insert(aidx);
			nzgrps.remove(&aidx);
			if nzgrps.is_empty() {
				let r = rxy[aidx];
				let x = rng.gen_range(r[0]+1, r[2]);
				let y = rng.gen_range(r[1]+1, r[3]);
				room.create_now()
					.with(Pos::new('\\', [x,y]))
					.with(Portal)
					.build();
				break
			}
		}
		fn add_wall(room: &mut World, doors: &mut FnvHashSet<[i16; 2]>, xy: [i16; 2], ch: char) {
			if !doors.contains(&xy) {
				doors.insert(xy);
				room.create_now()
					.with(Pos::new(ch, xy))
					.build();
			}
		}
		for xywh in rxy {
			for x in xywh[0]..xywh[2]+1 {
				for &i in [1usize, 3].into_iter() {
					add_wall(room, &mut doors, [x, xywh[i]], '\u{2550}')
				}
			}
			for y in xywh[1]..xywh[3]+1 {
				for &i in [0usize, 2].into_iter() {
					add_wall(room, &mut doors, [xywh[i], y], '\u{2551}')
				}
			}
		}
	}
}
