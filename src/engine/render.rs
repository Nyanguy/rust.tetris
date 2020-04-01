extern crate sdl2; 

use sdl2::{
    pixels::Color,
    event::Event,
    keyboard::Keycode,
    render::TextureQuery,
    rect::Rect,
    image::{LoadTexture, InitFlag}
};
use std::time::Duration;
use std::path::Path;
use super::text::Text;


// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);


// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
pub fn scaled_rect(pos_x: u32, pos_y: u32, rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };
    rect!(pos_x, pos_y, w, h)
}


//Main application struct
pub struct Window {
    canvas: sdl2::render::WindowCanvas,
    pub width: u32,
    pub height: u32,
    ctx: sdl2::Sdl,
    img_ctx: sdl2::image::Sdl2ImageContext
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        let ctx = sdl2::init().unwrap();
        let img_ctx = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
        let video_subsystem = ctx.video().unwrap();
     
        let window = video_subsystem.window("Tetris", width, height)
            .position_centered()
            .build()
            .unwrap();
        
        Window {
            canvas: window.into_canvas().build().unwrap(),
            width: width,
            height: height,
            ctx: ctx,
            img_ctx: img_ctx
        }
    }

    pub fn set_color(&mut self, clr: Color) {
        self.canvas.set_draw_color(clr);
    }

    pub fn draw_line(&mut self, color: Color, start: (i32,i32), finish: (i32,i32)) {
        self.canvas.set_draw_color(color);
        self.canvas.draw_line(start, finish);
    }

    pub fn draw_bg(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    pub fn draw_text(&mut self, text_vec: &Vec<Text>, padding: u32) -> Result<(), String> {
        for text_obj in text_vec {
            // render a surface, and convert it to a texture bound to the canvas
            let surface = text_obj.get_surface(vec!())?;
            let texture_creator = self.canvas.texture_creator();
            let texture = texture_creator.create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
    
            let TextureQuery { width, height, .. } = texture.query();
    
            // If the example text is too big for the screen, downscale it (and center irregardless)
            let target = scaled_rect(text_obj.pos_x, text_obj.pos_y, width, height, self.width - padding, self.height - padding);
    
            self.canvas.copy(&texture, None, Some(target))?;
        }
        Ok(())
    }

    pub fn get_context(&self) -> &sdl2::Sdl {
        &self.ctx
    }

    pub fn create_event_pump(&self) -> sdl2::EventPump{
        self.ctx.event_pump().unwrap()
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn load_texture(&mut self, png: &Path, src: Option<Rect>, dst: Option<Rect>) -> Result<(), String> {
        let texture_c = self.canvas.texture_creator();
        let texture = texture_c.load_texture(png)?;
        self.canvas.copy(&texture, src, dst)?;
        Ok(())
    }

    pub fn texture_to_buffer(&mut self, texture: &sdl2::render::Texture, src: Option<Rect>, dst: Option<Rect>) {
        self.canvas.copy(texture, src, dst).unwrap();
    }

    pub fn run(&mut self) {     
        // self.canvas.set_draw_color(Color::RGB(0, 255, 255));
        // self.canvas.clear();
        // self.canvas.present();
        let mut event_pump = self.ctx.event_pump().unwrap();
        // let mut i = 0;
        'running: loop {
            // i = (i + 1) % 255;
            // self.canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            // self.canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            // The rest of the game loop goes here...
    
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

