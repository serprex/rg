use fnv::FnvHashSet;
use specs::World;
use rand::Rng;
use super::util::R;

mod forest;
mod greedy;
mod bezier;

pub use self::forest::ForestRoomGen;
pub use self::greedy::GreedyRoomGen;
pub use self::bezier::BezierRoomGen;

pub trait RoomGen {
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World);
}

impl<T, U> RoomGen for (T, U)
	where T: RoomGen, U: RoomGen
{
	fn generate(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
		self.0.generate(rng, xyz, w, h, exits, room);
		self.1.generate(rng, xyz, w, h, exits, room);
	}
}
