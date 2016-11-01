use fnv::FnvHashSet;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use specs::World;
use x1b::RGB4;

use super::RoomGen;
use super::super::util::{R, Char};
use super::super::components::*;

#[derive(Clone)]
pub struct BezierRoomGen {
	pub ch: Char,
	pub width: usize,
	pub pts: Vec<[i16; 2]>,
}

impl BezierRoomGen {
	fn new(rng: &mut R, ch: Char, width: usize, pnum: usize, x: i16, y: i16, w: i16, h: i16) -> Self {
		let mut pts = Vec::new();
		let xrange = Range::new(x, x+w);
		let yrange = Range::new(y, y+h);
		pts.push(match rng.gen_range(0, 4){
			0 => [xrange.ind_sample(rng), y],
			1 => [xrange.ind_sample(rng), y+h-1],
			2 => [x, yrange.ind_sample(rng)],
			3 => [x+w-1, yrange.ind_sample(rng)],
			_ => unreachable!(),
		});
		for _ in 0..pnum {
			pts.push([xrange.ind_sample(rng), yrange.ind_sample(rng)]);
		}
		pts.push(match rng.gen_range(0, 4){
			0 => [xrange.ind_sample(rng), y],
			1 => [xrange.ind_sample(rng), y+h-1],
			2 => [x, yrange.ind_sample(rng)],
			3 => [x+w-1, yrange.ind_sample(rng)],
			_ => unreachable!(),
		});
		BezierRoomGen {
			ch: Char::new_with_color(' ', RGB4::Default, RGB4::Blue),
			width: 3,
			pts: pts,
		}
	}
}

impl RoomGen for BezierRoomGen {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
		let Walls(ref mut walls) = *room.write_resource::<Walls>();
		let mut n: f32 = 0.0;
		while n <= 1.0 {
			let mut pts = self.pts.clone();
			while pts.len() > 1 {
				for i in 0..pts.len()-1 {
					let px = (pts[i][0] as f32 * n + pts[i+1][0] as f32 * (n - 1.0) + 0.5) as i16;
					let py = (pts[i][1] as f32 * n + pts[i+1][1] as f32 * (n - 1.0) + 0.5) as i16;
					pts[i] = [px, py];
				}
			}
			let pt = pts[0];
			if !exits.contains(&[pt[0], pt[1]]) {
				walls.insert([pt[0], pt[1], xyz[2]], self.ch);
			}
			n += 1.0/512.0;
		}
	}
}
