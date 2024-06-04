use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl, VideoSubsystem};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub const PIX_SIZE: usize = 20;

pub struct Display {
    // data: [[bool; HEIGHT]; WIDTH],
    pub data: [bool; WIDTH * HEIGHT],
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    // window: Window,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub redraw: bool,
}

impl Default for Display {
    fn default() -> Self {
        Display::new()
    }
}

impl Display {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context
            .video()
            .expect("Cannot initialize video subsystem!");

        let window = video_subsystem
            .window(
                "Chip 8 Emulator",
                (WIDTH * PIX_SIZE) as u32,
                (HEIGHT * PIX_SIZE) as u32,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump().unwrap();

        Display {
            data: [false; WIDTH * HEIGHT],
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            canvas: canvas,
            event_pump: event_pump,
            redraw: true
        }
    }

    pub fn clear_screen(&mut self) {
        self.data.fill(false);
    }

    pub fn set_pixel(&mut self, row: usize, col: usize, val: bool) {
        *self.data.get_mut(row * WIDTH + col).unwrap() = val;
    }

    pub fn get_pixel(&self, row: usize, col: usize) -> bool {
        *self.data.get(row * WIDTH + col).unwrap()
    }

    pub fn create_white_rects(&self) -> Vec<Rect> {
        let mut rects: Vec<Rect> = Vec::new();
    
        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                if self.get_pixel(j, i) == true {
                // if self.data[j * WIDTH + i] == true {
                    // TODO: Attenzione (j * PIX_SIZE - 1) potrebbe essere
                    rects.push(Rect::new((i * PIX_SIZE) as i32, (j * PIX_SIZE) as i32, PIX_SIZE as u32, PIX_SIZE as u32));
                }
            }
        }
    
        rects
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
        }
    }
}
