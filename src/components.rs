use specs::{Component, VecStorage, NullStorage};

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
	pub nx: [i16; 2],
	pub ch: char,
}
impl Pos {
	pub fn new(ch: char, xy: [i16; 2]) -> Pos {
		Pos {
			xy: xy,
			nx: xy,
			ch: ch,
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AiState {
	Random,
	Aggro,
	Scared,
	Player,
	Melee(u8),
	Missile(u8),
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

impl_storage!(VecStorage, Pos, Mortal, Ai);
impl_storage!(NullStorage, Portal);
