use std::marker::PhantomData;
use std::mem;
use fnv::FnvHashMap;
use specs::{Entity, Component, VecStorage, HashMapStorage, NullStorage};

use actions::Action;
use util::Char;

macro_rules! impl_storage {
	($storage: ident, $($comp: ty),*) => {
		$(impl Component for $comp {
			type Storage = $storage<Self>;
		})*
	}
}

#[derive(Copy, Clone)]
pub struct Chr(pub Char);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Race {
	Wazzlefu,
	Raffbarf,
	Leylapan,
	Yerienel,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Dir {
	H, J, K, L
}

#[derive(Clone)]
pub enum AllyState {
	Random,
	Follow,
	Aggro(Entity),
	Scared(Entity),
	Wander([i16; 3]),
}

#[derive(Clone)]
pub enum AiState {
	Random,
	Aggro(Entity),
	Scared(Entity),
	Ally(Entity, AllyState),
	Player,
	PlayerInventory(usize),
	PlayerCasting(String),
}
#[derive(Clone)]
pub struct Ai(pub AiState, pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mortal(pub i16);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Weight(pub i16);

#[derive(Copy, Clone, Default)]
pub struct Solid;

#[derive(Copy, Clone, Default)]
pub struct Fragile;

#[derive(Copy, Clone)]
pub struct Strength(pub i16);

#[derive(Copy, Clone)]
pub struct Armor(pub Entity);

#[derive(Copy, Clone)]
pub struct Weapon(pub Entity);

#[derive(Copy, Clone)]
pub struct Shield(pub Entity);

#[derive(Copy, Clone)]
pub struct Head(pub Entity);

#[derive(Copy, Clone)]
pub struct Def<T> {
	pub val: i8,
	phantom: PhantomData<T>,
}

#[derive(Copy, Clone)]
pub struct Atk<T> {
	pub dmg: i16,
	pub dur: u8,
	pub spd: i8,
	phantom: PhantomData<T>,
}

impl<T> Def<T> {
	pub fn new(val: i8) -> Self {
		Def::<T> {
			val: val,
			phantom: PhantomData,
		}
	}
}

impl<T> Atk<T> {
	pub fn new(dmg: i16, dur: u8, spd: i8) -> Self {
		Atk::<T> {
			dmg: dmg,
			dur: dur,
			spd: spd,
			phantom: PhantomData,
		}
	}
}

#[derive(Clone)]
pub struct Casting(pub String);

#[derive(Clone)]
pub struct Bag(pub Vec<Entity>);

#[derive(Copy, Clone)]
pub struct Portal(pub [i16; 3]);

#[derive(Default)]
pub struct Walls(pub FnvHashMap<[i16; 3], Char>);

#[derive(Default)]
pub struct Log(pub [String; 12]);

impl Log {
	pub fn push(&mut self, mut s: String) {
		for l in self.0.iter_mut() {
			s = mem::replace(l, s);
		}
	}
}

#[derive(Copy, Clone)]
pub struct Dmg(pub i16);

pub enum Seek {
	Entity(Entity),
	Race(Race),
}

pub struct Seeking(pub Seek, pub Action);

pub struct Consume(pub Box<(Fn(Entity) -> Action) + Send + Sync>);

impl_storage!(VecStorage, Chr, Ai);
impl_storage!(HashMapStorage, Portal, Weight, Strength, Consume,
	Mortal, Armor, Weapon, Shield, Head, Bag, Race, Dmg, Seeking,
	Def<Armor>, Def<Weapon>, Def<Shield>, Def<Head>,
	Atk<Armor>, Atk<Weapon>, Atk<Shield>, Atk<Head>);
impl_storage!(NullStorage, Solid, Fragile);

pub fn is_aggro(r1: Race, r2: Race) -> bool {
	match (r1, r2) {
		(Race::Wazzlefu, _) => true,
		(_, Race::Wazzlefu) => true,
		_ => false,
	}
}
