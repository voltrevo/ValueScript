use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

pub fn termion_test() {
  let stdin = stdin();
  let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

  write!(
    stdout,
    "{}{}q to exit. Click, click, click!",
    termion::clear::All,
    termion::cursor::Goto(1, 1)
  )
  .unwrap();
  stdout.flush().unwrap();

  for c in stdin.events() {
    let evt = c.unwrap();
    match evt {
      Event::Key(Key::Char('q')) => break,
      Event::Mouse(me) => {
        if let MouseEvent::Press(_, x, y) = me {
          write!(stdout, "{}x", termion::cursor::Goto(x, y)).unwrap();
        }
      }
      _ => {}
    }
    stdout.flush().unwrap();
  }
}
