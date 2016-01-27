use std::io::{self, Write};

bitflags! {
	flags TextAttr: u8 {
		const TA_BOLD = 1,
		const TA_LOW = 2,
		const TA_UNDER = 4,
		const TA_BLINK = 8,
		const TA_REV = 16,
		const TA_INVIS = 32,
	}
}
impl TextAttr {
	pub fn clear(&mut self) -> bool {
		let ret = self.bits != 0;
		self.bits = 0;
		ret
	}
}
pub struct Cursor {
	pub buf: String,
	pub attr: TextAttr,
	pub x: u16,
	pub y: u16,
}

impl Default for Cursor {
	fn default() -> Self{
		Cursor {
			buf: String::new(),
			attr: TextAttr::empty(),
			x: 1,
			y: 1,
		}
	}
}

impl Cursor {
	pub fn esc(&mut self, s: &str){
		self.buf.push('\x1b');
		self.buf.push('[');
		self.buf.push_str(s)
	}
	pub fn escch(&mut self, c: char){
		self.buf.push('\x1b');
		self.buf.push('[');
		self.buf.push(c)
	}
	pub fn clearattr(&mut self){
		self.attr.clear();
		self.escch('m')
	}
	pub fn hasallattr(&self, ta: TextAttr) -> bool{
		self.attr.contains(ta)
	}
	pub fn hasanyattr(&self, ta: TextAttr) -> bool{
		self.attr.intersects(ta)
	}
	pub fn setattr(&mut self, ta: TextAttr){
		let mapping = [
			(TA_BOLD, "1m"),
			(TA_LOW, "2m"),
			(TA_UNDER, "4m"),
			(TA_BLINK, "5m"),
			(TA_REV, "7m"),
			(TA_INVIS, "8m")];
		for &(attr, code) in mapping.iter() {
			if ta.contains(attr) && !self.attr.contains(attr) {
				self.esc(code)
			}
		}
		self.attr.insert(ta)
	}
	pub fn setbold(&mut self){
		self.attr.insert(TA_BOLD);
		self.esc("1m")
	}
	pub fn setlow(&mut self){
		self.attr.insert(TA_LOW);
		self.esc("2m")
	}
	pub fn setunder(&mut self){
		self.attr.insert(TA_UNDER);
		self.esc("4m")
	}
	pub fn setblink(&mut self){
		self.attr.insert(TA_BLINK);
		self.esc("5m")
	}
	pub fn setrev(&mut self){
		self.attr.insert(TA_REV);
		self.esc("7m")
	}
	pub fn setinvis(&mut self){
		self.attr.insert(TA_INVIS);
		self.esc("8m")
	}
	pub fn up1(&mut self){
		self.y -= 1;
		self.escch('A')
	}
	pub fn down1(&mut self){
		self.y += 1;
		self.escch('B')
	}
	pub fn right1(&mut self){
		self.x -= 1;
		self.escch('C')
	}
	pub fn left1(&mut self){
		self.x += 1;
		self.escch('D')
	}
	pub fn up(&mut self, n: u16){
		self.y -= n;
		self.esc(&format!("{}A", n))
	}
	pub fn down(&mut self, n: u16){
		self.y += n;
		self.esc(&format!("{}B", n))
	}
	pub fn right(&mut self, n: u16){
		self.x -= n;
		self.esc(&format!("{}C", n))
	}
	pub fn left(&mut self, n: u16){
		self.x += n;
		self.esc(&format!("{}D", n))
	}
	pub fn x1down(&mut self, n: u16){
		self.x = 1;
		self.y += n;
		self.esc(&format!("{}E", n))
	}
	pub fn x1up(&mut self, n: u16){
		self.x = 1;
		self.y -= n;
		self.esc(&format!("{}F", n))
	}
	pub fn setx(&mut self, x: u16){
		self.x = x;
		self.esc(&format!("{}G", x))
	}
	pub fn sety(&mut self, y: u16){
		self.y = y;
		self.esc(&format!("{}d", y))
	}
	pub fn resetxy(&mut self){
		self.x = 1;
		self.y = 1;
		self.escch('H')
	}
	pub fn mv(&mut self, x: u16, y: u16){
		self.x = x;
		self.y = y;
		self.esc(&format!("{};{}H",y,x))
	}
	pub fn erasebelow(&mut self){
		self.escch('J')
	}
	pub fn eraseabove(&mut self){
		self.esc("1J")
	}
	pub fn eraseall(&mut self){
		self.esc("2J")
	}
	pub fn eraseleft(&mut self){
		self.escch('K')
	}
	pub fn eraseright(&mut self){
		self.esc("1K")
	}
	pub fn eraseline(&mut self){
		self.esc("2K")
	}
	pub fn delln(&mut self){
		self.escch('M')
	}
	pub fn dellns(&mut self, n: u16){
		self.esc(&format!("{}", n))
	}
	pub fn delch(&mut self){
		self.escch('S')
	}
	pub fn delchs(&mut self, n: u16){
		self.esc(&format!("{}", n))
	}
	pub fn getattr(&self) -> TextAttr{
		self.attr
	}
	pub fn getxy(&self) -> (u16, u16){
		(self.x, self.y)
	}
	pub fn rgbbg(&mut self, rgb: (u8, u8, u8), bg: u8){
		self.esc(&format!("{};2;{};{};{}m", bg, rgb.0, rgb.1, rgb.2))
	}
	pub fn rgb(&mut self, rgb: (u8, u8, u8)){
		self.rgbbg(rgb, 38)
	}
	pub fn prchr(&mut self, c: char){
		self.x += 1;
		self.buf.push(c)
	}
	pub fn print(&mut self, s: &str){
		let mut rsp = s.rsplit('\n');
		let last = rsp.next().unwrap();
		let lines = rsp.count();
		self.x += last.len() as u16;
		self.y += lines as u16;
		self.buf.push_str(s)
	}
	pub fn clear(&mut self) -> io::Result<usize> {
		self.buf.clear();
		self.x = 0;
		self.y = 0;
		Cursor::dropclear()
	}
	pub fn dropclear() -> io::Result<usize> {
		io::stdout().write(b"\x1bc")
	}
	pub fn flush(&mut self) -> io::Result<usize> {
		let ret = io::stdout().write(self.buf.as_bytes());
		self.buf.clear();
		ret
	}
}
