
use sdl3::Error;
use sdl3::event::Event;
use sdl3::rect::{Point, Rect};
use sdl3::{self, pixels::Color};
use sdl3::render::WindowCanvas;
use rand::random_range;

use std::sync::{Arc, Mutex};



// Global configuration variables
const DEBUG_MODE: bool = true;
const DEBUG_STAGE_COLOR: Color = Color { r: 0, g: 255, b: 0, a: 124 };
const CELL_SIZE: f32 = 10.;

#[derive(Clone)]
struct Vector2<T> {
    x: T,
    y: T,
}

impl<T> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x, y,
        }
    }
}

impl Into<Point> for Vector2<f32> {
    fn into(self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

impl Into<Point> for Vector2<u32> {
    fn into(self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

type Position = Vector2<f32>;

/// Handles the rendering of the A* algorithm into
/// specific parts of the SDL screen.
struct Stage {
    center: Position,
    width: f32,
    height: f32,
    canvas_ref: Arc<Mutex<WindowCanvas>>,
}


impl Stage {
    pub fn new(center: Position, width: f32, height: f32, canvas_ref: Arc<Mutex<WindowCanvas>>) -> Self {
        Self {
            center,
            width,
            height,
            canvas_ref,
        }
    }

    pub fn top_left(&self) -> Position {
        Position::new(
            self.center.x - self.width / 2.,
            self.center.y - self.height / 2.,
        )
    }

    pub fn render_grid(&self, grid: &Grid) -> Result<(), Error> {
        // If debug mode is active, render the stage's field
        // with a transparent green.
        if DEBUG_MODE {
            self.canvas_ref.lock().unwrap().set_draw_color(DEBUG_STAGE_COLOR);
            // render the transparent green square
            let rect = Rect::from_center(self.center.clone(), self.width as u32, self.height as u32);
            self.canvas_ref.lock().unwrap().draw_rect(rect)?;
        }

        let mut canv = self.canvas_ref.lock().unwrap();
        canv.set_draw_color(Color::CYAN);
        let top_left = self.top_left();

        for x in 0..grid.width {
            for y in 0..grid.height {
                let cell = grid.cell_at(x, y);
                let p = (top_left.x + x as f32 * CELL_SIZE, top_left.y + y as f32 * CELL_SIZE);

                // draw a line across the top of the cell
                if !cell.north_open() {
                    canv.draw_line(p, (p.0 + CELL_SIZE, p.1));
                }

                // draw a line across the right of the cell
                if !cell.east_open() {
                    canv.draw_line((p.0 + CELL_SIZE, p.1), (p.0 + CELL_SIZE, p.1 + CELL_SIZE));
                }

                // draw a line along the bottom of the cell
                if !cell.west_open() {
                    canv.draw_line(p, (p.0, p.1 + CELL_SIZE));
                }

                // draw a line on the left of the cell
                if !cell.south_open() {
                    canv.draw_line((p.0, p.1 + CELL_SIZE), (p.0 + CELL_SIZE, p.1 + CELL_SIZE));
                }
            }
        }

        Ok(())
    }
}


// Cells model the available directions at a given point in 
// the grid. Movement is restricted to north, east, south, and west, 
// so is stored as a boolean array with length 4
#[derive(Clone)]
pub struct Cell {
    doors: [bool; 4],
}

impl Cell {
    pub fn new(doors: [bool; 4]) -> Self {
        Self { 
            doors,
        }
    }

    pub fn closed() -> Self {
        Self {
            doors: [false; 4],
        }
    }

    pub fn doors(&self) -> &[bool] {
        &self.doors
    }

    pub fn north_open(&self) -> bool {
        self.doors[0]
    }

    pub fn east_open(&self) -> bool {
        self.doors[1]
    }

    pub fn south_open(&self) -> bool {
        self.doors[2]
    }

    pub fn west_open(&self) -> bool {
        self.doors[3]
    }

    pub fn set_north(&mut self, state: bool) {
        self.doors[0] = state;
    }

    pub fn set_east(&mut self, state: bool) {
        self.doors[1] = state;
    }

    pub fn set_south(&mut self, state: bool) {
        self.doors[2] = state;
    }

    pub fn set_west(&mut self, state: bool) {
        self.doors[3] = state;
    }
}


pub struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    // Uses a marching squares algorithm to create a maze
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = vec![Cell::closed(); width * height];

        // Start at the top left and march through the maze, creating at least
        // 1 viable path to the bottom right

        let mut current_idx: i32 = 0;
        let width_i32 = width as i32;

        while current_idx != (width * height) as i32 - 1 {
            // calculate the next possible indices if we moved one cell in 
            // orthogonally.
            let possible_idcs = [
                // increment x
                current_idx + 1,
                // increment y
                current_idx + width_i32,
                // decrement x,
                current_idx - 1,
                // decrement y
                current_idx - width_i32,
            ];
        
            let possible_idcs: Vec<&i32> = possible_idcs.iter().filter(|idx| {
                **idx < (width * height) as i32 && **idx > 0
            })
            .collect();

            let new_idx = *possible_idcs[random_range(0..possible_idcs.len())];

            println!("{}", new_idx);

            match new_idx - current_idx {
                // moved east
                1 => {
                    cells[current_idx as usize].set_east(true);
                    cells[new_idx as usize].set_west(true);
                },
                // moved west
                -1 => {
                    cells[current_idx as usize].set_west(true);
                    cells[new_idx as usize].set_east(true);
                },
                dy => {
                    if dy == width_i32 {
                        // moved north
                        cells[current_idx as usize].set_north(true);
                        cells[new_idx as usize].set_south(true);
                    } else {
                        // moved south
                        cells[current_idx as usize].set_south(true);
                        cells[new_idx as usize].set_north(true);
                    }

                }
            }

            current_idx = new_idx;
        }

        Self {
            cells,
            width,
            height,
        }
    }

    pub fn cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.cells[x + y * self.width]
    }

}




fn main() {
    let sdl = sdl3::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("A* SDL", 800, 600)
        .build()
        .unwrap();

    let canvas = Arc::new(Mutex::new(window.into_canvas()));
    let mut event_pump = sdl.event_pump().unwrap();


    let stage1 = Stage::new(
        Position::new(200., 150.),
        400., 
        300.,
        canvas.clone(),
    );

    let stage2 = Stage::new(
        Position::new(200., 450.),
        400., 
        300.,
        canvas.clone(),
    );


    let grid = Grid::new(20, 20);
    let grid_temp = Grid::new(10, 10);

    'running: loop {

        canvas.lock().unwrap().set_draw_color(Color::BLACK);
        canvas.lock().unwrap().clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {},
            }
        }


        // draw the stages 
        stage1.render_grid(&grid).unwrap();
        stage2.render_grid(&grid_temp).unwrap();

        canvas.lock().unwrap().present();

    }
}
