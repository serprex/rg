use std::marker::PhantomData;
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
	pub xy: [i16; 3],
	pub ch: char,
}
impl Pos {
	pub fn new(ch: char, xy: [i16; 3]) -> Pos {
		Pos {
			xy: xy,
			ch: ch,
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NPos(pub [i16; 3]);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Race {
	Wazzlefu,
	Raffbarf,
	Leylapan,
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Weight(pub i8);

pub struct Armor;
pub struct Weapon;
pub struct Shield;
pub struct Head;

pub struct Def<T: 'static> {
	pub val: i8,
	phantom: &'static PhantomData<T>,
}
pub struct Atk<T: 'static> {
	pub val: i8,
	phantom: &'static PhantomData<T>,
}
pub struct Bow(pub u8, pub i8);
pub enum Usable {
	Mortal(i8),
	CastBolt,
	CastForce,
	CastBlink,
}

#[derive(Clone)]
pub struct Bag(pub Vec<Entity>);

#[derive(Copy, Clone)]
pub struct Portal(pub [i16; 3]);

impl_storage!(VecStorage, Pos, Mortal, Ai, Race);
impl_storage!(HashMapStorage, NPos, Portal);
//impl_storage!(NullStorage, Pit);

pub fn is_aggro(r1: Race, r2: Race) -> bool {
	match (r1, r2) {
		(Race::Wazzlefu, _) => true,
		(_, Race::Wazzlefu) => true,
		_ => false,
	}
}
