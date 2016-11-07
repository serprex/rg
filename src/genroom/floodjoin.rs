use fnv::FnvHashSet;
use rand::Rng;
use specs::World;

use super::RoomGen;
use super::super::components::*;
use super::super::flood;
use super::super::util::R;

pub struct Floodjoin;

impl RoomGen for Floodjoin {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, _exits: &mut FnvHashSet<[i16; 3]>, room: &mut World) {
		let mut fx;
		let mut fy;
		let Walls(ref mut walls) = *room.write_resource::<Walls>();
		'xyloop: loop {
			for x in xyz[0]..xyz[0]+w {
				for y in xyz[1]..xyz[1]+h {
					if !walls.contains_key(&[x, y, xyz[2]]) {
						fx = x;
						fy = y;
						break 'xyloop
					}
				}
			}
			return
		}
		let mut xys: FnvHashSet<[i16; 2]> = Default::default();
		while let Some((x, y)) = {
			let pred = &|x, y| walls.contains_key(&[x, y, xyz[2]]);
			flood::fill(&mut xys, fx, fy, xyz[0], xyz[1], xyz[0] + w, xyz[1] + h, &pred);
			let candy = flood::holecandy(&xys, xyz[0], xyz[1], xyz[0] + w, xyz[1] + h, &pred);
			rng.choose(&candy).map(|&xy| xy)
		} {
			walls.remove(&[x, y, xyz[2]]);
			fx = x;
			fy = y;
		}
	}
}
