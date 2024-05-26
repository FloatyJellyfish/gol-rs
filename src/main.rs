extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    Button, EventSettings, Events, Key, MouseButton, MouseCursorEvent, PressEvent, RenderArgs,
    RenderEvent, WindowSettings,
};
use rand::{thread_rng, Rng};

type Colour = [f32; 4];

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

const COLOUR_BACKGROUND: Colour = [0.09, 0.09, 0.09, 1.0];
const COLOUR_ALIVE_CELL: Colour = [1.0; 4];
const COLOUR_DEAD_CELL: Colour = COLOUR_BACKGROUND;

struct Grid<const COL: usize, const ROW: usize> {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    cells: [[bool; COL]; ROW],
    compute: [[bool; COL]; ROW],
}

impl<const COL: usize, const ROW: usize> Grid<COL, ROW> {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            cells: [[false; COL]; ROW],
            compute: [[false; COL]; ROW],
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let cell_width = self.width as usize / COL;
        let cell_height = self.height as usize / ROW;
        gl.draw(args.viewport(), |c, g| {
            rectangle(
                COLOUR_BACKGROUND,
                [
                    self.x as f64,
                    self.y as f64,
                    self.width as f64,
                    self.height as f64,
                ],
                c.transform,
                g,
            );
            for y in 0..self.cells.len() {
                for x in 0..self.cells[y].len() {
                    let cell_colour = if self.cells[y][x] {
                        COLOUR_ALIVE_CELL
                    } else {
                        COLOUR_DEAD_CELL
                    };
                    rectangle(
                        cell_colour,
                        [
                            self.x as f64 + (x * cell_width) as f64,
                            self.y as f64 + (y * cell_height) as f64,
                            cell_width as f64,
                            cell_height as f64,
                        ],
                        c.transform,
                        g,
                    );
                }
            }
        });
    }

    fn press(&mut self, button: Button, mut mouse_pos: [f64; 2]) {
        mouse_pos[0] -= self.x as f64;
        mouse_pos[1] -= self.y as f64;
        let cell_width = self.width as usize / COL;
        let cell_height = self.height as usize / ROW;
        if let Button::Mouse(MouseButton::Left) = button {
            let x = mouse_pos[0] as usize / cell_width;
            let y = mouse_pos[1] as usize / cell_height;
            self.cells[y][x] = true;
        }

        if let Button::Mouse(MouseButton::Right) = button {
            let x = mouse_pos[0] as usize / cell_width;
            let y = mouse_pos[1] as usize / cell_height;
            self.cells[y][x] = false;
        }

        if let Button::Keyboard(Key::Space) = button {
            for y in 0..self.cells.len() {
                for x in 0..self.cells[y].len() {
                    let neighbours = count_neighbours(&self.cells, x as i32, y as i32);
                    self.compute[y][x] = if self.cells[y][x] {
                        // Cell is alive
                        if neighbours < 2 {
                            // Underpopulation
                            false
                        } else if neighbours == 2 || neighbours == 3 {
                            true
                        } else {
                            // Overpopulation
                            false
                        }
                    } else {
                        // Cell is dead
                        if neighbours == 3 {
                            // Reproduction
                            true
                        } else {
                            false
                        }
                    }
                }
            }
            self.cells = self.compute;
        }

        if let Button::Keyboard(Key::R) = button {
            for row in self.cells.iter_mut() {
                for cell in row.iter_mut() {
                    *cell = thread_rng().gen::<bool>();
                }
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let window: &mut GlutinWindow = &mut WindowSettings::new("Gol", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .expect("Unable to build window");

    let mut events = Events::new(EventSettings::new());
    let mut gl = GlGraphics::new(opengl);

    let mut grid: Grid<50, 50> = Grid::new(0, 0, 500, 500);
    let mut mouse_pos = [0.0, 0.0];

    while let Some(e) = events.next(window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |_c, g| {
                clear([1.0; 4], g);
            });
            grid.render(&mut gl, &args);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_pos = pos;
        }

        if let Some(button) = e.press_args() {
            grid.press(button, mouse_pos);
        }
    }
}

fn count_neighbours<const COL: usize, const ROW: usize>(
    grid: &[[bool; COL]; ROW],
    cx: i32,
    cy: i32,
) -> u32 {
    let mut count = 0;
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if cx + dx < 0 || cx + dx >= COL as i32 {
                continue;
            }
            if cy + dy < 0 || cy + dy >= ROW as i32 {
                continue;
            }
            if (dx != 0 || dy != 0) && grid[(cy + dy) as usize][(cx + dx) as usize] {
                count += 1;
            }
        }
    }
    count
}
