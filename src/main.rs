extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::time::SystemTime;

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
const HEIGHT: u32 = 550;

const COLOUR_BACKGROUND: Colour = [0.09, 0.09, 0.09, 1.0];
const COLOUR_ALIVE_CELL: Colour = [1.0; 4];
const COLOUR_DEAD_CELL: Colour = COLOUR_BACKGROUND;
const COLOUR_BUTTON: Colour = [1.0; 4];
const COLOUR_BUTTON_HOVER: Colour = [0.8, 0.8, 0.8, 1.0];

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
        if mouse_pos[0] > self.x as f64
            && mouse_pos[0] < self.x as f64 + self.width as f64
            && mouse_pos[1] > self.y as f64
            && mouse_pos[1] < self.y as f64 + self.width as f64
        {
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
        }

        if let Button::Keyboard(Key::Space) = button {
            self.calc_next();
        }

        if let Button::Keyboard(Key::R) = button {
            for row in self.cells.iter_mut() {
                for cell in row.iter_mut() {
                    *cell = thread_rng().gen::<bool>();
                }
            }
        }
    }

    fn calc_next(&mut self) {
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
}

trait Btn {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self;

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs);

    fn mouse_cursor(&mut self, pos: [f64; 2]);

    fn is_pressed(&mut self, button: &Button) -> bool;
}

struct Next {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    hover: bool,
}

impl Btn for Next {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            hover: false,
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        gl.draw(args.viewport(), |c, g| {
            let colour = if self.hover {
                COLOUR_BUTTON_HOVER
            } else {
                COLOUR_BUTTON
            };
            let pad = self.width / 5.0;
            Polygon::new(colour).draw(
                &[
                    [self.x + pad, self.y + pad],
                    [self.x + pad, self.y + self.height - pad],
                    [self.x + self.width - pad, self.y + (self.height / 2.0)],
                ],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
            let bar_width = self.width / 10.0;
            let bar_height = self.height - 2.0 * pad;
            Rectangle::new(colour).draw(
                [
                    self.x + self.width - pad - bar_width,
                    self.y + pad,
                    bar_width,
                    bar_height,
                ],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
        })
    }

    fn mouse_cursor(&mut self, pos: [f64; 2]) {
        self.hover = pos[0] > self.x
            && pos[0] < self.x + self.width
            && pos[1] > self.y
            && pos[1] < self.y + self.height;
    }

    fn is_pressed(&mut self, button: &Button) -> bool {
        Button::Mouse(MouseButton::Left) == *button && self.hover
    }
}

struct Play {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    hover: bool,
    toggle: bool,
}

impl Btn for Play {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            hover: false,
            toggle: false,
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let colour = if self.hover {
            COLOUR_BUTTON_HOVER
        } else {
            COLOUR_BUTTON
        };
        let pad = self.width / 5.0;
        if self.toggle {
            gl.draw(args.viewport(), |c, g| {
                Rectangle::new(colour).draw(
                    [
                        self.x + pad,
                        self.y + pad,
                        self.width / 5.0,
                        self.height - pad * 2.0,
                    ],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
                Rectangle::new(colour).draw(
                    [
                        self.x + self.width - pad - (self.width / 5.0),
                        self.y + pad,
                        self.width / 5.0,
                        self.height - pad * 2.0,
                    ],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
            })
        } else {
            gl.draw(args.viewport(), |c, g| {
                Polygon::new(colour).draw(
                    &[
                        [self.x + pad, self.y + pad],
                        [self.x + pad, self.y + self.height - pad],
                        [self.x + self.width - pad, self.y + (self.height / 2.0)],
                    ],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
            })
        }
    }

    fn mouse_cursor(&mut self, pos: [f64; 2]) {
        self.hover = pos[0] > self.x
            && pos[0] < self.x + self.width
            && pos[1] > self.y
            && pos[1] < self.y + self.height;
    }

    fn is_pressed(&mut self, button: &Button) -> bool {
        if *button == Button::Mouse(MouseButton::Left) && self.hover {
            self.toggle = !self.toggle;
            return true;
        }
        false
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

    let mut grid: Grid<50, 50> = Grid::new(0, 50, 500, 500);
    let mut next = Next::new((WIDTH / 2) - 50, 0, 50, 50);
    let mut play = Play::new(WIDTH / 2, 0, 50, 50);
    let mut mouse_pos = [0.0, 0.0];
    let mut playing = false;
    let mut last_tick = SystemTime::now();

    while let Some(e) = events.next(window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |_c, g| {
                clear(COLOUR_BACKGROUND, g);
            });
            grid.render(&mut gl, &args);
            next.render(&mut gl, &args);
            play.render(&mut gl, &args);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_pos = pos;
            next.mouse_cursor(pos);
            play.mouse_cursor(pos);
        }

        if let Some(button) = e.press_args() {
            grid.press(button, mouse_pos);
            if next.is_pressed(&button) {
                grid.calc_next();
            }

            if play.is_pressed(&button) {
                playing = !playing;
            }
        }

        if playing && SystemTime::now()
                .duration_since(last_tick)
                .expect("Time went backwards")
                .as_millis() > 250 {
            last_tick = SystemTime::now();
            grid.calc_next();
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
