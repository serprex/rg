use fnv::FnvHashSet;
use specs::World;
use rand::Rng;

pub mod forest;
pub mod greedy;
pub mod river;

pub trait RoomGen {
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World);
}

impl<T, U> RoomGen for (T, U)
	where T: RoomGen, U: RoomGen
{
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
		self.0.generate(rng, xyz, w, h, exits, room);
		self.1.generate(rng, xyz, w, h, exits, room);
	}
}

/*impl RoomGen for (Box<RoomGen>, Box<RoomGen>)
{
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, exits: &FnvHashSet<[i16; 2]>, room: &mut World) {
		self.0.generate(self, rng, xyz, w, h, exits, room);
		self.1.generate(self, rng, xyz, w, h, exits, room);
	}
}*/
