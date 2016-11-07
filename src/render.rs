use specs::{Entity, World};
use x1b::{self, RGB4};

use actions::Action;
use components::*;
use position::Possy;
use tick::Ticker;
use util::{Char, Curse, R};

pub fn render(player: Entity, _rng: &mut R, w: &mut World) {
	let possy = w.read_resource::<Possy>();
	if let Some(plpos) = possy.get_pos(player) {
		let mut curse = w.write_resource::<Curse>();
		let mut ticker = w.write_resource::<Ticker>();
		ticker.push(1, Action::Render { src: player });
		let cai = w.read::<Ai>();
		let chr = w.read::<Chr>();
		let cbag = w.read::<Bag>();
		let pxy = plpos;
		{
		let Walls(ref walls) = *w.read_resource::<Walls>();
		let mut xyz = pxy;
		for x in 0..12 {
			xyz[0] = pxy[0] + x - 6;
			for y in 0..12 {
				xyz[1] = pxy[1] + y - 6;
				if let Some(&ch) = walls.get(&xyz) {
					curse.set(x as u16, y as u16, ch);
				} else {
					for &e in possy.get_ents(xyz).iter() {
						if let Some(&Chr(ch)) = chr.get(e) {
							curse.set(x as u16, y as u16, ch);
							break
						}
					}
				}
			}
		}
		}
		{
			let Log(ref log) = *w.read_resource::<Log>();
			for (i, l) in log.iter().enumerate() {
				curse.printnows(0, 48 + i as u16, &l, x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
			}
		}
		if let Some(ai) = cai.get(player) {
			if let AiState::PlayerCasting(ref cast) = ai.state {
				curse.set(40, 1, Char::from('>'));
				curse.printnows(42, 1, &cast, x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
			} else if let AiState::PlayerInventory(invp) = ai.state {
				if let Some(&Bag(ref bag)) = cbag.get(player) {
					if bag.is_empty() {
						curse.printnows(40, 1, "Empty", x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
					} else {
						curse.set(40, 1 + invp as u16, Char::from('>'));
						for (idx, &item) in bag.iter().enumerate() {
							if idx > 9 {
								curse.set(41, 1 + idx as u16, Char::from((b'0' + (idx/10) as u8) as char));
							}
							curse.set(42, 1 + idx as u16, Char::from((b'0' + (idx%10) as u8) as char));
							if let Some(&Chr(ch)) = chr.get(item) {
								curse.set(44, 1 + idx as u16, ch);
							}
						}
					}
				}
				let weapons = w.read::<Weapon>();
				let armors = w.read::<Armor>();
				let shields = w.read::<Shield>();
				if let Some(&Weapon(item)) = weapons.get(player) {
					curse.printnows(61, 1, "Weapon",
						x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
					if let Some(&Chr(ch)) = chr.get(item) {
						curse.set(69, 1, ch);
					}
				}
				if let Some(&Shield(item)) = shields.get(player) {
					curse.printnows(60, 2, "Offhand",
						x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
					if let Some(&Chr(ch)) = chr.get(item) {
						curse.set(69, 2, ch);
					}
				}
				if let Some(&Armor(item)) = armors.get(player) {
					curse.printnows(62, 3, "Armor",
						x1b::TextAttr::empty(), RGB4::Default, RGB4::Default);
					if let Some(&Chr(ch)) = chr.get(item) {
						curse.set(69, 3, ch);
					}
				}
			}
		}
		curse.perframe_refresh_then_clear(Char::from(' ')).expect("tty fail");
	}
}
