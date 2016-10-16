extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;
extern crate fnv;
extern crate smallvec;

mod actions;
mod adjacency;
mod ailoop;
mod components;
mod genroom_forest;
mod genroom_greedy;
mod greedgrow;
mod position;
mod render;
mod roomgen;
mod termjuggler;
mod util;

use std::sync::atomic::Ordering;
use rand::{Rand, XorShiftRng};
use specs::*;

use components::*;
use position::Possy;
use roomgen::RoomGen;
use termjuggler::TermJuggler;
use util::{Char, EXITGAME};

macro_rules! w_register {
	($w: expr, $($comp: ty),*) => {
		$($w.register::<$comp>();)*
	}
}

fn main(){
	let mut rng = XorShiftRng::rand(&mut rand::thread_rng());
	let mut w = World::new();
	w_register!(w, Pos, Mortal, Ai, Portal, Race, Chr, Weight, Strength,
		Bow, Heal, Bag, Armor, Weapon, Head, Shield, Solid,
		Def<Armor>, Def<Weapon>, Def<Head>, Def<Shield>,
		Atk<Armor>, Atk<Weapon>, Atk<Head>, Atk<Shield>);
	w.add_resource(Walls::default());
	w.add_resource(Todo::default());
	let mut possy = Possy::new();
	let raffclaw = w.create_now()
		.with(Chr(Char::from('x')))
		.with(Weight(3))
		.with(Atk::<Weapon>::new(1, 1, -2))
		.with(Pos)
		.build();
	possy.set_pos(raffclaw, [4, 8, 0]);
	let leylabow = w.create_now()
		.with(Weight(1))
		.with(Bow(1, 1))
		.with(Chr(Char::from('j')))
		.build();
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('b')))
		.with(Weight(2))
		.with(Bow(4, 1))
		.with(Pos)
		.build(), [2, 5, 0]);
	let player = w.create_now()
		.with(Ai::new(AiState::Player, 10))
		.with(Bag(Vec::new()))
		.with(Chr(Char::from('@')))
		.with(Pos)
		.with(Solid)
		.with(Mortal(20))
		.with(Race::Wazzlefu)
		.with(Strength(10))
		.with(Weight(30))
		.build();
	possy.set_pos(player, [4, 4, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('r')))
		.with(Pos)
		.with(Solid)
		.with(Ai::new(AiState::Random, 12))
		.with(Mortal(4))
		.with(Weight(10))
		.with(Race::Raffbarf)
		.with(Weapon(raffclaw))
		.build(), [20, 6, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('k')))
		.with(Pos)
		.with(Solid)
		.with(Ai::new(AiState::Random, 8))
		.with(Mortal(2))
		.with(Weight(20))
		.with(Race::Leylapan)
		.with(Weapon(leylabow))
		.build(), [10, 8, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('!')))
		.with(Pos)
		.with(Atk::<Weapon>::new(2, 3, 2))
		.with(Weight(5))
		.build(), [8, 8, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('#')))
		.with(Pos)
		.with(Solid)
		.with(Def::<Armor>::new(2))
		.with(Weight(5))
		.build(), [8, 10, 0]);
	w.add_resource(possy);
	{
	let rrg = genroom_greedy::GreedyRoomGen::default();
	let frg = genroom_forest::ForestRoomGen::default();
	let mut f1 = [[20, 10, 22, 12], [30, 32, 20, 22], [20, 30, 24, 36], [50, 50, 55, 55], [60, 50, 62, 52], [80, 60, 82, 70], [90, 90, 95, 105]];
	let fadj = greedgrow::grow(&mut rng, &mut f1, 0, 0, 120, 120);
	/*for fxy in f1.iter() {
		if rng.gen() {
			rrg.generate(&mut rng, [fxy[0], fxy[1], 1], fxy[2]-fxy[0], fxy[3]-fxy[1], &mut w)
		} else {
			frg.generate(&mut rng, [fxy[0], fxy[1], 1], fxy[2]-fxy[0], fxy[3]-fxy[1], &mut w)
		}
	}*/
	rrg.generate(&mut rng, [0, 0, 0], 40, 40, &[], &mut w);
	frg.generate(&mut rng, [-10, -10, 1], 60, 60, &[], &mut w);
	frg.generate(&mut rng, [0, 0, 2], 40, 80, &[], &mut w);
	frg.generate(&mut rng, [0, 0, 3], 40, 80, &[], &mut w);
	}
	let mut curse = x1b::Curse::<x1b::RGB4>::new(80, 60);
	let _lock = TermJuggler::new();
	while !EXITGAME.load(Ordering::Relaxed) {
		render::render(player, &mut w, &mut curse);
		w.maintain();
		ailoop::ailoop(&mut rng, &mut w);
		loop {
			w.maintain();
			let todo = {
				let Todo(ref mut todos) = *w.write_resource::<Todo>();
				if todos.is_empty() { break }
				let todo = todos.drain(..).collect::<Vec<_>>();
				todo // stupid lifetime inference
			};
			for action in todo {
				action(&mut w);
			}
		}
		let mut possy = w.write_resource::<Possy>();
		possy.gc(&w);
	}
}
