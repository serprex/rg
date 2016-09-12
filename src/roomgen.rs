use specs::World;

pub trait RoomGen {
	fn generate(&self, xyz: [i16; 3], w: i16, h: i16, room: &mut World);
}
