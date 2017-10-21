use fnv::FnvHashSet;
use rand::Rng;
use specs::{World};

use super::RoomGen;
use super::super::components::*;
use super::super::util::{R, Char};
use super::super::greedgrow;
use super::super::position::Possy;

#[derive(Copy, Clone)]
pub struct Greedy(pub usize);
impl Default for Greedy {
	fn default() -> Self {
		Greedy(6)
	}
}
impl RoomGen for Greedy {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &mut FnvHashSet<[i16; 3]>, room: &mut World) {
		let dim = [xyz[0], xyz[1], w, h];
		let mut rxy = greedgrow::init(rng, self.0, 3, dim, true);
		let adjacent = greedgrow::grow(rng, &mut rxy, dim, true);
		let rc = rxy.len();
		let connset = greedgrow::joinlist(rng, &adjacent, rc);
		let (mut doors, lastaidx) = greedgrow::doors(rng, connset.into_iter(), &rxy);
		for &xy in exits.iter().filter(|xy| xy[2] == xyz[2]) {
			doors.insert([xy[0], xy[1]]);
		}
		{
			let r = rxy[lastaidx];
			let x = rng.gen_range(r[0]+1, r[2]);
			let y = rng.gen_range(r[1]+1, r[3]);
			let e = room.create_entity()
				.with(Chr(Char::from('\\')))
				.with(Portal([xyz[0]+x,xyz[1]+y,xyz[2]+1]))
				.build();
			let e2 = room.create_entity()
				.with(Chr(Char::from('/')))
				.with(Portal([xyz[0]+x-1,xyz[1]+y,xyz[2]]))
				.build();
			let mut possy = room.write_resource::<Possy>();
			possy.set_pos(e, [xyz[0]+x,xyz[1]+y,xyz[2]]);
			possy.set_pos(e2, [xyz[0]+x-1,xyz[1]+y,xyz[2]+1]);
			exits.insert([xyz[0]+x-1,xyz[1]+y,xyz[2]]);
			exits.insert([xyz[0]+x,xyz[1]+y,xyz[2]+1]);
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
