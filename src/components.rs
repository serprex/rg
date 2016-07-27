use specs::{Component, VecStorage, NullStorage};

macro_rules! impl_storage {
	($storage: ident, $($comp: ident),*) => {
		$(impl Component for $comp {
			type Storage = $storage<Self>;
		})*
	}
}

#[derive(Copy, Clone)]
pub struct PosComp {
	pub xy: [i16; 2],
	pub nx: [i16; 2],
	pub ch: char,
}
impl PosComp {
	pub fn new(ch: char, xy: [i16; 2]) -> PosComp {
		PosComp {
			xy: xy,
			nx: xy,
			ch: ch,
		}
	}
}

#[derive(Copy, Clone)]
pub enum AiComp {
	Random,
	Aggro,
	Scared,
}

#[derive(Copy, Clone)]
pub struct MortalComp(pub i16);
#[derive(Clone, Default)]
pub struct PlayerComp;
#[derive(Clone, Default)]
pub struct AggroComp;
#[derive(Clone, Default)]
pub struct WallComp;

impl_storage!(VecStorage, PosComp, MortalComp, AiComp);
impl_storage!(NullStorage, PlayerComp, AggroComp, WallComp);
