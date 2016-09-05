use rand::{self, Rng};
use specs::World;

use roomgen::RoomGen;
use components::*;
use util::Char;

#[derive(Copy, Clone)]
pub struct ForestRoomGen(pub f64);

impl Default for ForestRoomGen {
	fn default() -> Self {
		ForestRoomGen(0.3)
	}
}

impl RoomGen for ForestRoomGen {
	fn generate(&self, xyz: [i16; 3], w: i16, h: i16, room: &mut World) {
		let rc = self.0;
		let mut rng = rand::thread_rng();
		let Walls(ref mut walls) = *room.write_resource::<Walls>();
		for x in xyz[0]..xyz[0]+w {
			for y in xyz[1]..xyz[1]+h {
				if rng.gen::<f64>() < rc {
					walls.insert([x, y, xyz[2]], Char::from('*'));
				}
			}
		}
	}
}
