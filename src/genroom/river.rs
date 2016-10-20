use fnv::FnvHashSet;
use rand::Rng;
use specs::World;
use x1b::RGB4;

use super::RoomGen;
use super::super::util::Char;

#[derive(Clone)]
pub struct BezierRoomGen {
	pub ch: Char,
	pub width: usize,
	pub pts: Vec<[i16; 2]>,
}

impl BezierRoomGen {
	fn new<R: Rng>(rng: &mut R, ch: Char, width: usize, pnum: usize, x: i16, y: i16, w: i16, h: i16) -> Self {
		let mut pts = Vec::new();
		pts.push(match rng.gen_range(0, 4){
			0 => [rng.gen_range(x, x+w), y],
			1 => [rng.gen_range(x, x+w), y+h-1],
			2 => [x, rng.gen_range(y, y+h)],
			3 => [x+w-1, rng.gen_range(y, y+h)],
			_ => unreachable!(),
		});
		for _ in 0..pnum {
			pts.push([rng.gen_range(x, x+w), rng.gen_range(y, y+h)]);
		}
		pts.push(match rng.gen_range(0, 4){
			0 => [rng.gen_range(x, x+w), y],
			1 => [rng.gen_range(x, x+w), y+h-1],
			2 => [x, rng.gen_range(y, y+h)],
			3 => [x+w-1, rng.gen_range(y, y+h)],
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
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
	}
}
