use specs::World;
use rand::Rng;

pub trait RoomGen {
	fn generate<R: Rng>(&self, rng: &mut R, xyz: [i16; 3], w: i16, h: i16, room: &mut World);
}
