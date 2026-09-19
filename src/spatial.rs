use raylib::prelude::*;

pub trait Spatial {
    fn pos(&self) -> Vector2;
    fn distance_to(&self, other: &Self) -> f32 {
        let dist_x = other.pos().x - self.pos().x;
        let dist_y = other.pos().y - self.pos().y;
        ((dist_x*dist_x)-(dist_y*dist_y)).sqrt()
    }
}