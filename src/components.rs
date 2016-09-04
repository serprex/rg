use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use x1b::Char;
use specs::{World, Entity, Component, Storage, MaskedStorage, Allocator,
	VecStorage, HashMapStorage, NullStorage};
use super::super_sparse_storage::SuperSparseStorage;

macro_rules! impl_storage {
	($storage: ident, $($comp: ty),*) => {
		$(impl Component for $comp {
			type Storage = $storage<Self>;
		})*
	}
}

pub type Action = Box<Fn(Entity, &mut World) + Send + Sync>;

#[derive(Copy, Clone)]
pub struct Chr(pub Char<()>);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Pos(pub [i16; 3]);

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
	Melee(u8, i16),
	Missile(Dir, i16),
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
pub struct AiStasis(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mortal(pub i16);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Weight(pub i16);

#[derive(Copy, Clone, Default)]
pub struct Solid;

#[derive(Copy, Clone)]
pub struct Strength(pub u16);

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

#[derive(Copy, Clone)]
pub struct Bow(pub u8, pub i16);

pub struct Spell(pub Action);

pub struct Casting(pub String);

pub struct Todo(pub Vec<Action>);

impl Todo {
	pub fn add<A, D>(todos: &mut Storage<Todo, A, D>, ent: Entity, act: Action)
		where A: Deref<Target = Allocator>, D: DerefMut<Target = MaskedStorage<Todo>>
	{
		if let Some(&mut Todo(ref mut tv)) = todos.get_mut(ent) {
			tv.push(act);
			return
		}
		todos.insert(ent, Todo(vec![act]));
	}
}

#[derive(Copy, Clone)]
pub struct WDirection(pub Dir);

#[derive(Copy, Clone)]
pub struct Heal(pub i16);

#[derive(Clone)]
pub struct Bag(pub Vec<Entity>);

#[derive(Copy, Clone)]
pub struct Portal(pub [i16; 3]);

#[derive(Copy, Clone)]
pub struct Inventory(pub Entity, pub usize);

impl_storage!(VecStorage, Pos, Mortal, Ai, Race, Chr);
impl_storage!(HashMapStorage, Portal, Weight, Strength,
	Bow, Heal,
	Armor, Weapon, Shield, Head, Bag, AiStasis, Inventory, Spell, Todo,
	Def<Armor>, Def<Weapon>, Def<Shield>, Def<Head>,
	Atk<Armor>, Atk<Weapon>, Atk<Shield>, Atk<Head>);
impl_storage!(NullStorage, Solid);
impl_storage!(SuperSparseStorage, NPos, WDirection, Casting);

pub fn is_aggro(r1: Race, r2: Race) -> bool {
	match (r1, r2) {
		(Race::Wazzlefu, _) => true,
		(_, Race::Wazzlefu) => true,
		_ => false,
	}
}
