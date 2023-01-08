use crate::screen;
use mos_hardware::petscii::Petscii;
use ufmt::uWrite;

pub struct MapTextWriter {
    cursor: u8,
}

impl MapTextWriter {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    fn write_byte(&mut self, c: u8) {
        if self.cursor >= 40 {
            return;
        }

        let c = Petscii::from_byte(c as u8).to_screen_code();
        let screen = screen::current_text();

        unsafe {
            (screen as *mut u8).offset(self.cursor as isize).write(c);
        }

        self.cursor += 1;
    }
}

impl uWrite for MapTextWriter {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.as_bytes() {
            self.write_byte(*c);
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.write_byte(c as u8);
        Ok(())
    }
}
