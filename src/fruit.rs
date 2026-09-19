use raylib::prelude::*;

use crate::spatial::Spatial;

pub struct Fruit {
    pub pos: Vector2,
    pub size: Vector2,
}

impl Fruit {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            size: Vector2::new(5.0, 5.0),
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_v(self.pos, self.size, Color::WHITE);
    }

    pub fn update(&mut self) {
        
    }
}

impl Spatial for Fruit {
    fn pos(&self) -> Vector2 {
        self.pos
    }
}