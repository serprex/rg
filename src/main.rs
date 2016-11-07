extern crate rand;
extern crate termios;
extern crate x1b;
extern crate specs;
extern crate fnv;
extern crate smallvec;

mod actions;
mod adjacency;
mod components;
mod flood;
mod genroom;
mod greedgrow;
mod position;
mod render;
mod termjuggler;
mod tick;
mod util;

use std::sync::atomic::Ordering;
use fnv::FnvHashSet;
use rand::{Rand, Rng};
use specs::*;

use actions::Action;
use components::*;
use genroom::RoomGen;
use position::Possy;
use termjuggler::TermJuggler;
use tick::Ticker;
use util::{Char, Curse, R, EXITGAME};

macro_rules! w_register {
	($w: expr, $($comp: ty),*) => {
		$($w.register::<$comp>();)*
	}
}

fn main(){
	let mut rng = R::rand(&mut rand::thread_rng());
	let mut w = World::new();
	w_register!(w, Mortal, Ai, Portal, Race, Chr, Weight, Strength,
		Bag, Armor, Weapon, Head, Shield, Solid, Fragile, Dmg, Seeking,
		Def<Armor>, Def<Weapon>, Def<Head>, Def<Shield>,
		Atk<Armor>, Atk<Weapon>, Atk<Head>, Atk<Shield>);
	w.add_resource(Walls::default());
	w.add_resource(Log::default());
	let mut ticker = Ticker::default();
	let mut possy = Possy::default();
	let raffclaw = w.create_now()
		.with(Chr(Char::from('x')))
		.with(Weight(3))
		.with(Atk::<Weapon>::new(1, 1, -2))
		.build();
	possy.set_pos(raffclaw, [4, 8, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('b')))
		.with(Weight(10))
		.with(Strength(80))
		.build(), [2, 5, 0]);
	for _ in 0..10 {
		possy.set_pos(w.create_now()
			.with(Chr(Char::from('^')))
			.with(Weight(3))
			.build(), [3, 7, 0]);
	}
	let player = w.create_now()
		.with(Ai::new(AiState::Player, 10))
		.with(Bag(Vec::new()))
		.with(Chr(Char::from('@')))
		.with(Solid)
		.with(Mortal(20))
		.with(Race::Wazzlefu)
		.with(Strength(50))
		.with(Weight(60))
		.build();
	ticker.push(10, Action::PossyGc);
	ticker.push(0, Action::Render { src: player });
	ticker.push(10, Action::Ai { src: player });
	possy.set_pos(player, [4, 4, 0]);
	let raff = w.create_now()
		.with(Chr(Char::from('r')))
		.with(Solid)
		.with(Ai::new(AiState::Random, 12))
		.with(Mortal(4))
		.with(Weight(10))
		.with(Race::Raffbarf)
		.with(Weapon(raffclaw))
		.build();
	ticker.push(12, Action::Ai { src: raff });
	possy.set_pos(raff, [20, 6, 0]);
	let leyla = w.create_now()
		.with(Chr(Char::from('k')))
		.with(Solid)
		.with(Ai::new(AiState::Random, 8))
		.with(Mortal(2))
		.with(Weight(20))
		.with(Race::Leylapan)
		.with(Strength(15))
		.build();
	ticker.push(8, Action::Ai { src: leyla });
	possy.set_pos(leyla, [10, 8, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('!')))
		.with(Atk::<Weapon>::new(2, 3, 2))
		.with(Weight(5))
		.build(), [8, 8, 0]);
	possy.set_pos(w.create_now()
		.with(Chr(Char::from('#')))
		.with(Def::<Armor>::new(2))
		.with(Weight(5))
		.build(), [8, 10, 0]);
	let ring = w.create_now()
		.with(Chr(Char::from('o')))
		.with(Weight(1))
		.build();
	possy.set_pos(ring, [15, 20, 0]);
	let seeker = w.create_now()
		.with(Chr(Char::from('&')))
		.with(Seeking(ring, Action::Say(String::from("Thanks"))))
		.with(Weight(55))
		.with(Strength(200))
		.with(Bag(Vec::new()))
		.build();
	possy.set_pos(seeker, [5, 2, 0]);
	ticker.push(20, Action::Seek(20, seeker));
	w.add_resource(possy);
	w.add_resource(ticker);
	{
	let rrg = genroom::Greedy::default();
	let frg = (genroom::Forest::default(), genroom::Floodjoin);
	let genbag: [&RoomGen; 2] = [&rrg, &frg];
	let f1dim = [0, 0, 120, 120];
	let mut f1 = greedgrow::init(&mut rng, 7, 5, f1dim);
	let fadj = greedgrow::grow(&mut rng, &mut f1, f1dim);
	let fjlist = greedgrow::joinlist(&mut rng, &fadj, f1.len());
	let (exits, _) = greedgrow::doors(&mut rng, fjlist.into_iter(), &f1);
	for fxy in f1.iter() {
		rng.choose(&genbag).unwrap().generate(&mut rng, [fxy[0], fxy[1], 1], fxy[2]-fxy[0]+1, fxy[3]-fxy[1]+1, &exits, &mut w)
	}
	rrg.generate(&mut rng, [0, 0, 0], 40, 40, &FnvHashSet::default(), &mut w);
	}
	w.add_resource(Curse::new(80, 60));
	let _lock = TermJuggler::new();
	while !EXITGAME.load(Ordering::Relaxed) {
		w.maintain();
		let events = {
			let mut ticker = w.write_resource::<Ticker>();
			ticker.pop()
		};
		for event in events {
			event.call(&mut rng, &mut w);
		}
	}
}
