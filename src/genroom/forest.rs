use fnv::FnvHashSet;
use rand::Rng;
use rand::distributions::{Range, IndependentSample};
use specs::World;

use super::RoomGen;
use super::super::actions::Action;
use super::super::components::*;
use super::super::tick::Ticker;
use super::super::position::Possy;
use super::super::util::{R, Char};
use super::super::x1b::RGB4;

#[derive(Copy, Clone)]
pub struct Forest {
	pub trees: usize,
	pub raff: usize,
	pub null: usize,
}

impl Default for Forest {
	fn default() -> Self {
		Forest {
			trees: 4,
			raff: 1,
			null: 9,
		}
	}
}

impl RoomGen for Forest {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &mut FnvHashSet<[i16; 3]>, room: &mut World) {
		let range = Range::new(0, self.trees + self.raff + self.null);
		let raffspeed = Range::new(8, 14);
		let raffclaw = room.create_now()
			.with(Chr(Char::from('x')))
			.with(Weight(2))
			.with(Atk::<Weapon>::new(1, 2, 2))
			.build();

		for x in xyz[0]..xyz[0]+w {
			for y in xyz[1]..xyz[1]+h {
				if exits.contains(&[x, y, xyz[2]]) { continue }
				{
					let possy = room.read_resource::<Possy>();
					if !possy.get_ents([x, y, xyz[2]]).is_empty() {
						continue
					}
				}
				let r = range.ind_sample(rng);
				if r < self.trees {
					let Walls(ref mut walls) = *room.write_resource::<Walls>();
					let (fg, bg) =
						if rng.gen() { (RGB4::LightGreen, RGB4::Green) }
						else { (RGB4::Green, RGB4::LightGreen) };
					walls.insert([x, y, xyz[2]], Char::new_with_color('*', fg, bg));
				} else if r - self.trees < self.raff {
					let speed = raffspeed.ind_sample(rng);
					let e = room.create_now()
						.with(Chr(Char::from('r')))
						.with(Ai::new(AiState::Random, speed))
						.with(Solid)
						.with(Race::Raffbarf)
						.with(Mortal(4))
						.with(Weight(10))
						.with(Weapon(raffclaw))
						.build();
					let mut ticker = room.write_resource::<Ticker>();
					ticker.push(speed, Action::Ai { src: e });
					let mut possy = room.write_resource::<Possy>();
					possy.set_pos(e, [x, y, xyz[2]]);
				}
			}
		}
	}
}
