use specs::{Component, VecStorage};

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
	Player,
}

#[derive(Copy, Clone)]
pub struct MortalComp(pub i16);

impl_storage!(VecStorage, PosComp, MortalComp, AiComp);
