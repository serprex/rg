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

#[derive(Copy, Clone)]
pub struct PosComp(pub Pos);
#[derive(Copy, Clone)]
pub struct MortalComp(pub i16);
#[derive(Clone, Default)]
pub struct PlayerComp;
#[derive(Clone, Default)]
pub struct AggroComp;
#[derive(Clone, Default)]
pub struct WallComp;
impl_storage!(VecStorage, PosComp, MortalComp);
impl_storage!(NullStorage, PlayerComp, AggroComp, WallComp);
