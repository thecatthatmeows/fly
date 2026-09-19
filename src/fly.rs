use raylib::prelude::*;

use crate::spatial::Spatial;

pub struct Fly {
    pos: Vector2,
    size: Vector2,
    velocity: Vector2
}

impl Fly {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vector2::new(x, y),
            size: Vector2::new(10.0, 10.0),
            velocity: Vector2::zero()
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_v(self.pos, self.size, Color::WHITE);
    }

    pub fn update(&mut self) {
        self.pos += self.velocity;
        self.velocity *= 0.99;
    }

    pub fn activity_to_velocity(&self, motor_neuron_activities: &[f32]) -> Vector2 {
        todo!()
    }
}

impl Spatial for Fly {
    fn pos(&self) -> Vector2 {
        self.pos
    }
}