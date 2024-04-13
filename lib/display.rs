pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[derive(Debug)]
pub struct Display {
    data: [[bool; HEIGHT]; WIDTH],
}

impl Default for Display {
    fn default() -> Self {
        Display::new()
    }
}

impl Display {
    pub fn new() -> Self {
        Display {
            data: [[false; HEIGHT]; WIDTH],
        }
    }

    pub fn clear_screen(&mut self) {
        self.data.fill([false; HEIGHT])
    }

    pub fn set_pixel(&mut self, row: usize, col: usize, val: bool) {
        *self.data.get_mut(row).unwrap().get_mut(col).unwrap() = val
    }

    pub fn get_pixel(&self, row: usize, col: usize) -> bool {
        *self.data.get(row).unwrap().get(col).unwrap()
    }

    pub fn display_terminal(&self) {
        for col in 0..HEIGHT {
            for row in 0..WIDTH {
                if !self.get_pixel(row, col) {
                    print!(" ");
                } else {
                    print!("o");
                }
            }
            println!();
            //print!("\n");
        }
    }
}
