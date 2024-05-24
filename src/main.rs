#![feature(const_fn_floating_point_arithmetic)]

use glium_graphics::{Glium2d, GliumWindow};
use glutin_window::OpenGL;
use graphics::*;
use piston::{
    Button, EventLoop, EventSettings, Events, Key, MouseButton, MouseCursorEvent, PressEvent,
    RenderEvent, WindowSettings,
};

type Colour = [f32; 4];

const fn rgb(r: u8, g: u8, b: u8, a: u8) -> Colour {
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
}

const COLOUR_BACKGROUND: Colour = rgb(0x18, 0x18, 0x18, 0x18);
const COLOUR_CELL: Colour = rgb(255, 255, 255, 255);

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;
const CELL_SIZE: u32 = 10;
const GRID_COLUMNS: u32 = WIDTH / CELL_SIZE;
const GRID_ROWS: u32 = HEIGHT / CELL_SIZE;

fn main() {
    let opengl = OpenGL::V4_5;
    let window: &mut GliumWindow = &mut WindowSettings::new("Gol", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .build()
        .expect("Unable to build window");

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut g2d = Glium2d::new(opengl, window);

    let mut grid = [[false; GRID_COLUMNS as usize]; GRID_ROWS as usize];
    let mut grid_compute = [[false; GRID_COLUMNS as usize]; GRID_ROWS as usize];
    let mut mouse_pos = [0.0, 0.0];

    while let Some(e) = events.next(window) {
        if let Some(args) = e.render_args() {
            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                clear(COLOUR_BACKGROUND, g);
                for y in 0..GRID_ROWS {
                    for x in 0..GRID_COLUMNS {
                        if grid[y as usize][x as usize] {
                            rectangle(
                                COLOUR_CELL,
                                [
                                    (x * CELL_SIZE) as f64,
                                    (y * CELL_SIZE) as f64,
                                    CELL_SIZE as f64,
                                    CELL_SIZE as f64,
                                ],
                                c.transform,
                                g,
                            );
                        }
                    }
                }
            });
            target.finish().expect("Unable to finish drawing");
        }

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            let x = mouse_pos[0] as u32 / CELL_SIZE;
            let y = mouse_pos[1] as u32 / CELL_SIZE;
            grid[y as usize][x as usize] = true;
        }

        if let Some(Button::Mouse(MouseButton::Right)) = e.press_args() {
            let x = mouse_pos[0] as u32 / CELL_SIZE;
            let y = mouse_pos[1] as u32 / CELL_SIZE;
            grid[y as usize][x as usize] = false;
        }

        if let Some(Button::Keyboard(Key::Space)) = e.press_args() {
            for y in 0..GRID_COLUMNS as usize {
                for x in 0..GRID_ROWS as usize {
                    let neighbours = count_neighbours(&grid, x as i32, y as i32);
                    grid_compute[y][x] = if grid[y][x] {
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
            grid = grid_compute;
        }
    }
}

fn count_neighbours(
    grid: &[[bool; GRID_COLUMNS as usize]; GRID_ROWS as usize],
    cx: i32,
    cy: i32,
) -> u32 {
    let mut count = 0;
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if cx + dx < 0 || cx + dx >= GRID_COLUMNS as i32 {
                continue;
            }
            if cy + dy < 0 || cy + dy >= GRID_ROWS as i32 {
                continue;
            }
            if (dx != 0 || dy != 0) && grid[(cy + dy) as usize][(cx + dx) as usize] {
                count += 1;
            }
        }
    }
    count
}