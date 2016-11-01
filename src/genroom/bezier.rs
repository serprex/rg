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
	pub pnum: usize,
}

impl BezierRoomGen {
	fn new(ch: Char, width: usize, pnum: usize) -> Self {
		BezierRoomGen {
			ch: Char::new_with_color(' ', RGB4::Default, RGB4::Blue),
			width: 3,
			pnum: pnum,
		}
	}
}

impl RoomGen for BezierRoomGen {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
		let mut pts = Vec::new();
		let x = xyz[0] as f32;
		let y = xyz[1] as f32;
		let w = w as f32;
		let h = h as f32;
		let xrange = Range::new(x, x+w);
		let yrange = Range::new(y, y+h);
		let b4 = Range::new(0, 4);
		pts.push(match b4.ind_sample(rng) {
			0 => [xrange.ind_sample(rng), y],
			1 => [xrange.ind_sample(rng), y+h-1.0],
			2 => [x, yrange.ind_sample(rng)],
			3 => [x+w-1.0, yrange.ind_sample(rng)],
			_ => unreachable!(),
		});
		for _ in 0..self.pnum {
			pts.push([xrange.ind_sample(rng), yrange.ind_sample(rng)]);
		}
		pts.push(match b4.ind_sample(rng) {
			0 => [xrange.ind_sample(rng), y],
			1 => [xrange.ind_sample(rng), y+h-1.0],
			2 => [x, yrange.ind_sample(rng)],
			3 => [x+w-1.0, yrange.ind_sample(rng)],
			_ => unreachable!(),
		});
		let Walls(ref mut walls) = *room.write_resource::<Walls>();
		let mut n: f32 = 0.0;
		while n <= 1.0 {
			let mut pts = pts.clone();
			while pts.len() > 1 {
				for i in 0..pts.len()-1 {
					let px = pts[i][0] as f32 * n + pts[i+1][0] as f32 * (n - 1.0);
					let py = pts[i][1] as f32 * n + pts[i+1][1] as f32 * (n - 1.0);
					pts[i] = [px, py];
				}
			}
			let p0 = (pts[0][0] + 0.5) as i16;
			let p1 = (pts[0][1] + 0.5) as i16;
			if !exits.contains(&[p0, p1]) {
				walls.insert([p0, p1, xyz[2]], self.ch);
			}
			n += 1.0/512.0;
		}
	}
}
