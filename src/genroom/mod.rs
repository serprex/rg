use super::util::R;
use fnv::FnvHashSet;
use specs::World;

mod bezier;
mod floodjoin;
mod forest;
mod greedy;

pub use self::bezier::Bezier;
pub use self::floodjoin::Floodjoin;
pub use self::forest::Forest;
pub use self::greedy::Greedy;

pub trait RoomGen {
	fn generate(
		&self,
		rng: &mut R,
		xyz: [i16; 3],
		w: i16,
		h: i16,
		exits: &mut FnvHashSet<[i16; 3]>,
		room: &mut World,
	);
}

impl<T, U> RoomGen for (T, U)
where
	T: RoomGen,
	U: RoomGen,
{
	fn generate(
		&self,
		rng: &mut R,
		xyz: [i16; 3],
		w: i16,
		h: i16,
		exits: &mut FnvHashSet<[i16; 3]>,
		room: &mut World,
	) {
		self.0.generate(rng, xyz, w, h, exits, room);
		self.1.generate(rng, xyz, w, h, exits, room);
	}
}
