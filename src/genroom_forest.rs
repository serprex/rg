use rand::Rng;
use rand::distributions::{Range, IndependentSample};
use specs::World;

use roomgen::RoomGen;
use components::*;
use position::Possy;
use util::Char;
use x1b::RGB4;

#[derive(Copy, Clone)]
pub struct ForestRoomGen {
	pub trees: usize,
	pub raff: usize,
	pub null: usize,
}

impl Default for ForestRoomGen {
	fn default() -> Self {
		ForestRoomGen {
			trees: 3,
			raff: 1,
			null: 6,
		}
	}
}

impl RoomGen for ForestRoomGen {
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &[[i16; 2]], room: &mut World) {
		let range = Range::new(0, self.trees + self.raff + self.null);
		let raffspeed = Range::new(8, 14);
		let raffclaw = room.create_now()
			.with(Chr(Char::from('x')))
			.with(Weight(2))
			.with(Atk::<Weapon>::new(1, 2, 2))
			.build();
		for x in xyz[0]..xyz[0]+w {
			for y in xyz[1]..xyz[1]+h {
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
						.with(Pos)
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
	}
}
