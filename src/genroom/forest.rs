use fnv::FnvHashSet;
use rand::Rng;
use rand::distributions::{Range, IndependentSample};
use specs::World;

use super::RoomGen;
use super::super::components::*;
use super::super::flood;
use super::super::position::Possy;
use super::super::util::{R, Char};
use super::super::x1b::RGB4;

#[derive(Copy, Clone)]
pub struct ForestRoomGen {
	pub trees: usize,
	pub raff: usize,
	pub null: usize,
}

impl Default for ForestRoomGen {
	fn default() -> Self {
		ForestRoomGen {
			trees: 4,
			raff: 1,
			null: 9,
		}
	}
}

impl RoomGen for ForestRoomGen {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
		let range = Range::new(0, self.trees + self.raff + self.null);
		let raffspeed = Range::new(8, 14);
		let raffclaw = room.create_now()
			.with(Chr(Char::from('x')))
			.with(Weight(2))
			.with(Atk::<Weapon>::new(1, 2, 2))
			.build();

		for x in xyz[0]..xyz[0]+w {
			for y in xyz[1]..xyz[1]+h {
				if exits.contains(&[x, y]) { continue }
				let r = range.ind_sample(rng);
				if r < self.trees {
					let Walls(ref mut walls) = *room.write_resource::<Walls>();
					let (fg, bg) =
						if rng.gen() { (RGB4::LightGreen, RGB4::Green) }
						else { (RGB4::Green, RGB4::LightGreen) };
					walls.insert([x, y, xyz[2]], Char::new_with_color('*', fg, bg));
				} else if r - self.trees < self.raff {
					let e = room.create_now()
						.with(Chr(Char::from('r')))
						.with(Solid)
						.with(Ai::new(AiState::Random, raffspeed.ind_sample(rng)))
						.with(Race::Raffbarf)
						.with(Mortal(4))
						.with(Weight(10))
						.with(Weapon(raffclaw))
						.build();
					let mut possy = room.write_resource::<Possy>();
					possy.set_pos(e, [x, y, xyz[2]]);
				}
			}
		}

		let (mut fx, mut fy) = (0, 0);
		let Walls(ref mut walls) = *room.write_resource::<Walls>();
		'xyloop:
		for x in xyz[0]..xyz[0]+w {
			for y in xyz[1]..xyz[1]+h {
				if !walls.contains_key(&[x, y, xyz[2]]) {
					fx = x;
					fy = y;
					break 'xyloop
				}
			}
		}
		let mut xys: FnvHashSet<[i16; 2]> = Default::default();
		while let Some((x, y)) = {
			flood::fill(&mut xys, fx, fy, xyz[0], xyz[1], xyz[0] + w, xyz[1] + h, |x, y| walls.contains_key(&[x, y, xyz[2]]));
			let candy = flood::holecandy(&xys, xyz[0], xyz[1], xyz[0] + w, xyz[1] + h, |x, y| walls.contains_key(&[x, y, xyz[2]]));
			rng.choose(&candy).map(|&xy| xy)
		} {
			walls.remove(&[x, y, xyz[2]]);
			fx = x;
			fy = y;
		}
	}
}
