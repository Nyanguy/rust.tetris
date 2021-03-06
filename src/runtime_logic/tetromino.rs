use std::collections::HashMap;
use std::path::Path;
use std::process::exit;

use rand::{distributions::{Distribution, Standard}, Rng};
use sdl2::rect::Rect;

use crate::engine::render::Window;

//Todo scale interface
//Todo background animation
//Todo ghosting/shadow
//Todo APM
//TODO Wall-kicks for rotation

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Some(Rect::new($x as i32, $y as i32, $w as u32, $h as u32))
    )
);


pub fn draw_fn(window: &mut Window, pos: (u32,u32), offset: u32, t_size: u32) -> Result<(), String>{
    window.load_texture(Path::new("data/art/tiles.png"),
                        rect!(offset, 0, t_size, t_size),
                        rect!(pos.0, pos.1, t_size, t_size))?;
    Ok(())
}



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape { I, T, L, J, S, Z, O}
pub enum Rotation {Right, Left}

impl Shape {
    ///Returns a default matrix shape for a given figure
    pub fn matrix(&self) -> [u8; 4] {
        match self {
            Shape::I => [4,5,6,7],    // 0,  1,  2,  3
            Shape::J => [0,4,5,6],    // 4,  5,  6,  7
            Shape::L => [3,5,6,7],    // 8,  9, 10, 11
            Shape::O => [2,3,6,7],    //12, 13, 14, 15
            Shape::S => [2,3,5,6],
            Shape::Z => [0,1,5,6],
            Shape::T => [1,4,5,6],
        }
    }

    ///Returns a corresponding shift in texture
    pub fn texture_offset(&self) -> u8 {
        match self {
            Shape::I => 5*18,
            Shape::J => 0,
            Shape::L => 6*18,
            Shape::O => 4*18,
            Shape::S => 3*18,
            Shape::Z => 2*18,
            Shape::T => 18,
        }
    }
}

impl Distribution<Shape> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Shape {
        match rng.gen_range(0, 7) {
            0 => Shape::I,
            1 => Shape::T,
            2 => Shape::L,
            3 => Shape::J,
            4 => Shape::S,
            5 => Shape::Z,
            _ => Shape::O
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tetromino {
    shape: Shape,
    pos_x: u32, // Coordinates with respect to the tile size: 0 -> t_size
    pos_y: u32, // Coordinates with respect to the tile size: 0 -> t_size
    t_size: u32,
    m_shape: [u8; 4],
    tiles: [(u32,u32);4],
    r_angle: usize,
    color_offset: u8,
    on_hold: bool,
    active: bool
}

impl Tetromino {
    pub fn new(shape: Shape) -> Self {
        let mut t = Tetromino {
            r_angle: 0,
            m_shape: shape.matrix(),
            color_offset: shape.texture_offset(),
            shape,
            tiles: [(0,0);4],
            pos_x: 10,
            pos_y: 0,
            t_size: 18,
            on_hold: false,
            active: true
        };
        t.mut_tiles_pos();
        t
    }

    pub fn id(&self) -> &Shape {
        &self.shape
    }

    pub fn is_active(&self) -> bool {self.active}

    pub fn deactivate(&mut self) {self.active = false}

    pub fn get_shape(&self) -> Shape {
        self.shape
    }

    // does a fixed angle rotation to acknowledge T-spin/wall kicks, save on performance
    pub fn rotate(&mut self, r: Rotation, all_tiles: &HashMap<(u32,u32), u32>, lb: u32, rb: u32, f: u32) {
        let variations: [[u8;4];4] = match self.shape {
            Shape::I => [[4,5,6,7], [2,6,10,14], [8,9,10,11], [1,5,9,13]],  // 0,  1,  2,  3
            Shape::J => [[0,4,5,6], [1,2,5,9], [4,5,6,10], [1,5,8,9]],      // 4,  5,  6,  7
            Shape::L => [[3,5,6,7], [2,6,10,11], [5,6,7,9], [1,2,6,10]],    // 8,  9, 10, 11
            Shape::O => [[2,3,6,7], [2,3,6,7], [2,3,6,7], [2,3,6,7]],       //12, 13, 14, 15
            Shape::S => [[2,3,5,6], [2,6,7,11], [6,7,9,10], [1,5,6,10]],
            Shape::Z => [[0,1,5,6], [2,5,6,9], [4,5,9,10], [1,4,5,8]],
            Shape::T => [[1,4,5,6], [1,5,6,9], [4,5,6,9], [1,4,5,9]],
        };
        let prev_angle = self.r_angle;
        match r {
            Rotation::Right => {
                if self.r_angle < 3 {self.r_angle += 1}
                else {self.r_angle = 0}
            },
            Rotation::Left => {
                if self.r_angle > 0 {self.r_angle -= 1}
                else {self.r_angle = 3}
            }
        }
        self.m_shape = variations[self.r_angle];
        self.mut_tiles_pos();
        let org_x = self.pos_x;

        // EDGE CASE FOR ROTATION
        while self.collides_with_frame(lb, rb, f) {
            if self.tiles.iter().any(|t| all_tiles.contains_key(t)) {
                self.pos_x = org_x;
                self.r_angle = prev_angle;
                self.m_shape = variations[self.r_angle];
                return ();
            }
            else if self.pos_x<=9 // approximate middle
                 {self.pos_x += 1}
            else {self.pos_x -= 1}
        }
        self.mut_tiles_pos();
        if self.tiles.iter().any(|t| all_tiles.contains_key(t)) {
            let mut columns = 0;
            let mut pos_y = 0;
            for tile in self.tiles.iter() {
                if all_tiles.contains_key(tile) && columns < 2 {
                    if pos_y != tile.1 {
                        columns += 1;
                        pos_y = tile.1;
                    }
                } else {
                    self.pos_x = org_x;
                    self.r_angle = prev_angle;
                    self.m_shape = variations[self.r_angle];
                    return ();
                }
            }
        }


    }

    pub fn set_default_pos(&mut self) {
        self.pos_x = 8;
        self.pos_y = 0;
        self.r_angle = 0;
        self.on_hold = false;
        self.m_shape = self.shape.matrix();
    }

    pub fn set_for_next(&mut self) {
        self.pos_x = 12;
        self.pos_y = 0;
        self.r_angle = 0;
        self.on_hold = true;
        self.m_shape = self.shape.matrix();
    }

    pub fn set_to_pocket(&mut self) {
        self.pos_x = 13;
        self.pos_y = 24;
        self.r_angle = 0;
        self.m_shape = self.shape.matrix();
    }

    pub fn make_move(&mut self, steps: i32, direction: i32, axis: u8, left_border: u32, right_border: u32, floor: u32) {
        let prev = (self.pos_x,self.pos_y);
        match axis {
            0 => self.pos_x = (steps*direction + self.pos_x as i32) as u32,
            1 => self.pos_y = (steps*direction + self.pos_y as i32) as u32,
            _ => exit(12),
        }
        self.mut_tiles_pos();
        if self.collides_with_frame(left_border, right_border, floor) {
            self.pos_x = prev.0;
            self.pos_y = prev.1;
        }
        self.mut_tiles_pos();
    }

    pub fn delete_tile(&mut self, t_pos: (u32,u32)) {
        let v = self.tiles.iter().position(|&t| t == t_pos);
        for idx in v {
            self.tiles[idx]   = (0,0);
            self.m_shape[idx] = 0;
        }
    }


    pub fn mut_tiles_pos(&mut self) {
        for (i,t) in self.m_shape.iter().enumerate() {
            self.tiles[i].0 = self.t_size * (self.pos_x + (t % 4 * 1) as u32);
            self.tiles[i].1 = self.t_size * (self.pos_y + (t / 4 * 1) as u32);
        }
    }

    pub fn get_tiles_pos(&self) -> [(u32, u32);4] {
        self.tiles
    }

    pub fn collides_with_frame(&mut self, left_border: u32, right_border: u32, floor: u32) -> bool {
        let coord_s = self.tiles;
        for tile_s in coord_s.iter() {
            if tile_s.0 <= left_border || tile_s.0 >= right_border {return true}
            else if tile_s.1 >= floor {self.active = false; return true}
        }
        false
    }

    pub fn draw(&self, window: &mut Window) -> Result<(), String> {
        for tile in self.m_shape.iter() {
            let x = self.pos_x + (tile%4*1) as u32;
            let y = self.pos_y + (tile/4*1) as u32;
            if y > 2 || self.on_hold {
                window.load_texture(Path::new("data/art/tiles.png"),
                                    rect!(self.color_offset, 0, self.t_size, self.t_size),
                                    rect!(x * self.t_size, y * self.t_size, self.t_size, self.t_size))?;
            }
        }
        Ok(())
    }
}