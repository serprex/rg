use termios::*;
use x1b;

pub struct TermJuggler(Termios);
impl TermJuggler {
	pub fn new() -> Option<Self> {
		if let Ok(ref mut term) = Termios::from_fd(0) {
			let oldterm = *term;
			cfmakeraw(term);
			tcsetattr(0, TCSANOW, term).expect("tcsetattr failed");
			print!("\x1bc\x1b[?25l");
			Some(TermJuggler(oldterm))
		} else {
			None
		}
	}
}
impl Drop for TermJuggler {
	fn drop(&mut self) {
		x1b::Cursor::dropclear().ok();
		tcsetattr(0, TCSAFLUSH, &self.0).ok();
	}
}
