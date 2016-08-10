use specs::{Entity, Component, VecStorage, HashMapStorage, NullStorage};

macro_rules! impl_storage {
	($storage: ident, $($comp: ident),*) => {
		$(impl Component for $comp {
			type Storage = $storage<Self>;
		})*
	}
}

#[derive(Copy, Clone)]
pub struct Pos {
	pub xy: [i16; 2],
	pub ch: char,
}
impl Pos {
	pub fn new(ch: char, xy: [i16; 2]) -> Pos {
		Pos {
			xy: xy,
			ch: ch,
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NPos(pub [i16; 2]);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Race {
	Wazzlefu,
	Raffbarf,
	Leylapan,
	Rat,
	None,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Dir {
	H, J, K, L
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AiState {
	Random,
	Aggro(Entity),
	Scared(Entity),
	Player,
	Melee(u8),
	Missile(Dir),
}
#[derive(Copy, Clone)]
pub struct Ai {
	pub state: AiState,
	pub speed: u8,
	pub tick: u8,
}
impl Ai {
	pub fn new(state: AiState, speed: u8) -> Ai {
		Ai {
			state: state,
			speed: speed,
			tick: speed,
		}
	}
}

#[derive(Copy, Clone)]
pub struct Mortal(pub i16);

#[derive(Copy, Clone, Default)]
pub struct Portal;

impl_storage!(VecStorage, Pos, Mortal, Ai, Race);
impl_storage!(HashMapStorage, NPos);
impl_storage!(NullStorage, Portal);
