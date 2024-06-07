#![allow(dead_code)]

use opengl_graphics::GlGraphics;
use piston::{Button, RenderArgs};

pub trait Btn {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self;

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs);

    fn mouse_cursor(&mut self, pos: [f64; 2]);

    fn is_pressed(&mut self, button: &Button) -> bool;
}

pub trait Widget {
    fn pos(&self) -> [f64; 2];

    fn size(&self) -> [f64; 2];

    fn set_pos(&mut self, x: f64, y: f64);

    fn set_size(&mut self, width: f64, height: f64);
}

pub struct HGroup {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl HGroup {
    pub fn new(x: f64, y: f64, width: f64, height: f64, items: &mut [&mut dyn Widget]) -> Self {
        let item_height = height;
        let item_width = width / items.len() as f64;
        for (i, item) in items.iter_mut().enumerate() {
            item.set_size(item_width, item_height);
            item.set_pos(x + i as f64 * item_height, y);
        }

        Self {
            x,
            y,
            width,
            height,
        }
    }
}
