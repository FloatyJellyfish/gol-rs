extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::time::SystemTime;

use glutin_window::GlutinWindow;
use graphics::{ellipse::centered, types::FontSize, *};
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::{
    Button, EventSettings, Events, Key, MouseButton, MouseCursorEvent, PressEvent, RenderArgs,
    RenderEvent, WindowSettings,
};
use rand::{
    distributions::{Distribution, Standard},
    random, thread_rng, Rng,
};
use rectangle::Border;

type Colour = [f32; 4];

const WIDTH: u32 = 500;
const HEIGHT: u32 = 550;

const COLOUR_BACKGROUND: Colour = [0.09, 0.09, 0.09, 1.0];
const COLOUR_ALIVE_CELL: Colour = [1.0; 4];
const COLOUR_DEAD_CELL: Colour = COLOUR_BACKGROUND;
const COLOUR_BUTTON: Colour = [1.0; 4];
const COLOUR_HOVER: Colour = [0.8, 0.8, 0.8, 1.0];

struct Grid<const COL: usize, const ROW: usize> {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    cells: [[bool; COL]; ROW],
    compute: [[bool; COL]; ROW],
    hover: Option<[usize; 2]>,
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
            hover: None,
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
            if let Some([x, y]) = self.hover {
                Rectangle::new([0.0; 4])
                    .border(Border {
                        color: COLOUR_HOVER,
                        radius: 0.0,
                    })
                    .draw(
                        [
                            self.x as f64 + (x * cell_width) as f64,
                            self.y as f64 + (y * cell_height) as f64,
                            cell_width as f64,
                            cell_height as f64,
                        ],
                        &DrawState::new_alpha(),
                        c.transform,
                        g,
                    );
                let x = self.x as f64 + (x * cell_width) as f64;
                let y = self.y as f64 + (y * cell_height) as f64;
                Line::new(COLOUR_HOVER, 1.0).draw(
                    [x, y, x, y + cell_height as f64],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
                Line::new(COLOUR_HOVER, 1.0).draw(
                    [
                        x + cell_width as f64,
                        y,
                        x + cell_width as f64,
                        y + cell_height as f64,
                    ],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
                Line::new(COLOUR_HOVER, 1.0).draw(
                    [x, y, x + cell_width as f64, y],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
                Line::new(COLOUR_HOVER, 1.0).draw(
                    [
                        x,
                        y + cell_height as f64,
                        x + cell_width as f64,
                        y + cell_height as f64,
                    ],
                    &DrawState::new_alpha(),
                    c.transform,
                    g,
                );
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
            self.randomize();
        }
    }

    fn mouse_cursor(&mut self, mut pos: [f64; 2]) {
        pos[0] -= self.x as f64;
        pos[1] -= self.y as f64;
        if pos[0] > self.x as f64
            && pos[0] < self.x as f64 + self.width as f64
            && pos[1] > self.y as f64
            && pos[1] < self.y as f64 + self.width as f64
        {
            let cell_width = self.width as usize / COL;
            let cell_height = self.height as usize / ROW;
            self.hover = Some([pos[0] as usize / cell_width, pos[1] as usize / cell_height]);
        } else {
            self.hover = None;
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

    fn randomize(&mut self) {
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                *cell = thread_rng().gen::<bool>();
            }
        }
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
                COLOUR_HOVER
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
            COLOUR_HOVER
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

#[derive(PartialEq)]
enum Dice {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Distribution<Dice> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dice {
        let index: u8 = rng.gen_range(1..=6);
        match index {
            1 => Dice::One,
            2 => Dice::Two,
            3 => Dice::Three,
            4 => Dice::Four,
            5 => Dice::Five,
            6 => Dice::Six,
            _ => unreachable!(),
        }
    }
}

struct Random {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    hover: bool,
    dice_state: Dice,
}

impl Btn for Random {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            hover: false,
            dice_state: Dice::Six,
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let colour = if self.hover {
            COLOUR_HOVER
        } else {
            COLOUR_BUTTON
        };
        let pad = self.width / 5.0;
        let btn_width = self.width - 2.0 * pad;
        let btn_height = self.height - 2.0 * pad;
        gl.draw(args.viewport(), |c, g| {
            Rectangle::new_round(colour, 5.0).draw(
                [self.x + pad, self.y + pad, btn_width, btn_height],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
            let dot_size = 3.25;
            match self.dice_state {
                Dice::One => Ellipse::new(COLOUR_BACKGROUND).draw(
                    centered([
                        self.x + self.width / 2.0,
                        self.y + self.height / 2.0,
                        dot_size,
                        dot_size,
                    ]),
                    &DrawState::default(),
                    c.transform,
                    g,
                ),
                Dice::Two => {
                    for x in 1..=2 {
                        Ellipse::new(COLOUR_BACKGROUND).draw(
                            centered([
                                self.x + self.height / 2.0,
                                self.y + pad + x as f64 * (self.height - 2.0 * pad) / 3.0,
                                dot_size,
                                dot_size,
                            ]),
                            &DrawState::default(),
                            c.transform,
                            g,
                        );
                    }
                }
                Dice::Three => {
                    for i in 1..=3 {
                        Ellipse::new(COLOUR_BACKGROUND).draw(
                            centered([
                                self.x + pad + i as f64 * (self.width - 2.0 * pad) / 4.0,
                                self.y + pad + i as f64 * (self.height - 2.0 * pad) / 4.0,
                                dot_size,
                                dot_size,
                            ]),
                            &DrawState::default(),
                            c.transform,
                            g,
                        );
                    }
                }
                Dice::Four => {
                    for x in 1..=2 {
                        for y in 1..=2 {
                            Ellipse::new(COLOUR_BACKGROUND).draw(
                                centered([
                                    self.x + pad + x as f64 * (self.width - 2.0 * pad) / 3.0,
                                    self.y + pad + y as f64 * (self.height - 2.0 * pad) / 3.0,
                                    dot_size,
                                    dot_size,
                                ]),
                                &DrawState::default(),
                                c.transform,
                                g,
                            );
                        }
                    }
                }
                Dice::Five => {
                    for x in 1..=3 {
                        for y in 1..=3 {
                            if (x != 2 && y != 2) || (x == 2 && y == 2) {
                                Ellipse::new(COLOUR_BACKGROUND).draw(
                                    centered([
                                        self.x + pad + x as f64 * (self.width - 2.0 * pad) / 4.0,
                                        self.y + pad + y as f64 * (self.height - 2.0 * pad) / 4.0,
                                        dot_size,
                                        dot_size,
                                    ]),
                                    &DrawState::default(),
                                    c.transform,
                                    g,
                                );
                            }
                        }
                    }
                }
                Dice::Six => {
                    for x in 0..2 {
                        for y in 0..3 {
                            let cx = self.x + pad + (x + 1) as f64 * (btn_width / 3.0);
                            let cy = self.y + pad + (y + 1) as f64 * ((btn_height) / 4.0);
                            Ellipse::new(COLOUR_BACKGROUND).draw(
                                centered([cx, cy, dot_size, dot_size]),
                                &DrawState::new_alpha(),
                                c.transform,
                                g,
                            );
                        }
                    }
                }
            }
        });
    }

    fn mouse_cursor(&mut self, pos: [f64; 2]) {
        self.hover = pos[0] > self.x
            && pos[0] < self.x + self.width
            && pos[1] > self.y
            && pos[1] < self.y + self.height;
    }

    fn is_pressed(&mut self, button: &Button) -> bool {
        if *button == Button::Mouse(MouseButton::Left) && self.hover {
            let mut rand = random();
            while rand == self.dice_state {
                rand = random();
            }
            self.dice_state = rand;
            return true;
        }
        false
    }
}

struct Increase {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    hover: bool,
}

impl Btn for Increase {
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
        let colour = if self.hover {
            COLOUR_HOVER
        } else {
            COLOUR_BUTTON
        };
        let pad = self.width / 5.0;
        gl.draw(args.viewport(), |c, g| {
            Polygon::new(colour).draw(
                &[
                    [self.x + pad, self.y + pad],
                    [self.x + pad, self.y + self.height - pad],
                    [self.x + self.width / 2.0, self.y + self.height / 2.0],
                ],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
            Polygon::new(colour).draw(
                &[
                    [self.x + self.width / 2.0, self.y + pad],
                    [self.x + self.width / 2.0, self.y + self.height - pad],
                    [self.x + self.width - pad, self.y + self.width / 2.0],
                ],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
        });
    }

    fn mouse_cursor(&mut self, pos: [f64; 2]) {
        self.hover = pos[0] > self.x
            && pos[0] < self.x + self.width
            && pos[1] > self.y
            && pos[1] < self.y + self.height;
    }

    fn is_pressed(&mut self, button: &Button) -> bool {
        if *button == Button::Mouse(MouseButton::Left) && self.hover {
            return true;
        }
        false
    }
}

struct Decrease {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    hover: bool,
}

impl Btn for Decrease {
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
        let colour = if self.hover {
            COLOUR_HOVER
        } else {
            COLOUR_BUTTON
        };
        let pad = self.width / 5.0;
        gl.draw(args.viewport(), |c, g| {
            Polygon::new(colour).draw(
                &[
                    [self.x + self.width / 2.0, self.y + pad],
                    [self.x + self.width / 2.0, self.y + self.height - pad],
                    [self.x + pad, self.y + self.height / 2.0],
                ],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
            Polygon::new(colour).draw(
                &[
                    [self.x + self.width - pad, self.y + pad],
                    [self.x + self.width - pad, self.y + self.height - pad],
                    [self.x + self.width / 2.0, self.y + self.width / 2.0],
                ],
                &DrawState::new_alpha(),
                c.transform,
                g,
            );
        });
    }

    fn mouse_cursor(&mut self, pos: [f64; 2]) {
        self.hover = pos[0] > self.x
            && pos[0] < self.x + self.width
            && pos[1] > self.y
            && pos[1] < self.y + self.height;
    }

    fn is_pressed(&mut self, button: &Button) -> bool {
        if *button == Button::Mouse(MouseButton::Left) && self.hover {
            return true;
        }
        false
    }
}

struct Speed {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    speed: usize,
    min: usize,
    max: usize,
    font_size: FontSize,
}

impl Speed {
    #[allow(clippy::too_many_arguments)]
    fn new(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        speed: usize,
        min: usize,
        max: usize,
        font_size: FontSize,
    ) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            speed,
            min,
            max,
            font_size,
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs, glyph_cache: &mut GlyphCache) {
        let text = format!("{}x", self.speed);
        let x = self.x + self.width / 2.0
            - glyph_cache
                .width(self.font_size, &text)
                .expect("Unable to measure text") as f64
                / 2.0;
        let y = self.y + self.height / 2.0 + self.font_size as f64 / 2.0 - 5.0;
        gl.draw(args.viewport(), |c, g| {
            Text::new_color(COLOUR_BUTTON, self.font_size)
                .draw(
                    &text,
                    glyph_cache,
                    &DrawState::default(),
                    c.transform.trans(x, y),
                    g,
                )
                .expect("Unable to draw text");
        });
    }

    fn increase(&mut self) {
        if self.speed < self.max {
            self.speed *= 2;
        }
    }

    fn decrease(&mut self) {
        if self.speed > self.min {
            self.speed /= 2;
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
    let mut glyph_cache =
        GlyphCache::new("fonts/Nexa-Heavy.ttf", (), TextureSettings::new()).unwrap();

    let mut grid: Grid<50, 50> = Grid::new(0, 50, 500, 500);
    let mut next = Next::new((WIDTH / 2) - 150, 0, 50, 50);
    let mut play = Play::new((WIDTH / 2) - 100, 0, 50, 50);
    let mut random = Random::new(WIDTH / 2 - 50, 0, 50, 50);
    let mut decrease = Decrease::new(WIDTH / 2, 0, 50, 50);
    let mut speed = Speed::new(WIDTH / 2 + 50, 0, 50, 50, 4, 1, 16, 30);
    let mut increase = Increase::new((WIDTH / 2) + 100, 0, 50, 50);

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
            random.render(&mut gl, &args);
            decrease.render(&mut gl, &args);
            increase.render(&mut gl, &args);
            speed.render(&mut gl, &args, &mut glyph_cache)
        }

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_pos = pos;
            grid.mouse_cursor(pos);
            next.mouse_cursor(pos);
            play.mouse_cursor(pos);
            random.mouse_cursor(pos);
            decrease.mouse_cursor(pos);
            increase.mouse_cursor(pos);
        }

        if let Some(button) = e.press_args() {
            grid.press(button, mouse_pos);
            if next.is_pressed(&button) {
                grid.calc_next();
            }

            if play.is_pressed(&button) {
                playing = !playing;
            }

            if random.is_pressed(&button) {
                grid.randomize();
            }

            if increase.is_pressed(&button) {
                speed.increase();
            }

            if decrease.is_pressed(&button) {
                speed.decrease();
            }
        }

        if playing
            && SystemTime::now()
                .duration_since(last_tick)
                .expect("Time went backwards")
                .as_millis()
                > (1000 / speed.speed as u128)
        {
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
