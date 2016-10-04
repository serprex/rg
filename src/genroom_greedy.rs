use std::cmp;
use std::collections::HashSet;
use fnv::FnvHashSet;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use specs::World;

use roomgen::RoomGen;
use components::*;
use util::{rectover, Char};
use greedgrow;
use position::Possy;

#[derive(Copy, Clone)]
pub struct GreedyRoomGen(pub usize);
impl Default for GreedyRoomGen {
	fn default() -> Self {
		GreedyRoomGen(6)
	}
}
impl RoomGen for GreedyRoomGen {
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, room: &mut World) {
		let rc = self.0;
		if w<2 || h<2 { return }
		let betwh = Range::new(0, (w-2)*(h-2));
		let mut rxy = Vec::with_capacity(rc);
		while rxy.len() < rc {
			let xy = betwh.ind_sample(rng);
			let (rx, ry) = (xyz[0] + xy % (w-2), xyz[1] + xy / (h-2));
			let candy = [rx, ry, rx+2, ry+2];
			if !rxy.iter().any(|&a| rectover(candy, a))
				{ rxy.push(candy) }
		}
		let adjacent = greedgrow::grow(rng, &mut rxy, xyz[0], xyz[1], w, h);
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
					.with(Chr(Char::from('\\')))
					.with(NPos([xyz[0]+x,xyz[1]+y,xyz[2]]))
					.with(Pos)
					.with(Portal([xyz[0]+x,xyz[1],xyz[2]+1]))
					.build();
				break
			}
		}
		let possy = room.read_resource::<Possy>();
		if let Some(floor) = possy.floors.get(&xyz[2]) {
			for k in floor.keys() {
				doors.insert([k[0], k[1]]);
			}
		}
		let Walls(ref mut walls) = *room.write_resource::<Walls>();
		let mut add_wall = |xy: [i16; 2], ch: char| {
			if !doors.contains(&xy) {
				doors.insert(xy);
				walls.insert([xyz[0]+xy[0],xyz[1]+xy[1],xyz[2]], Char::from(ch));
			}
		};
		for xywh in rxy {
			for x in xywh[0]..xywh[2]+1 {
				for &i in &[1usize, 3] {
					add_wall([x, xywh[i]], '\u{2550}')
				}
			}
			for y in xywh[1]..xywh[3]+1 {
				for &i in &[0usize, 2] {
					add_wall([xywh[i], y], '\u{2551}')
				}
			}
		}
	}
}
