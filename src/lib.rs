///
/// Rust code for Conway's Game of Life which includes WASM
/// bindings for running in the browser
///
/// NOTE: I believe there are some bugs when width != height
/// TODO: Add some more tests
///

mod utils;

use rand::Rng;
use std::fmt;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}


// u8 for efficient use of memory
// values are important for efficient summing of alive neighbours
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    fn new(width: u32, height: u32, alive_cells: Vec<(u32, u32)>) -> Self {
        // Initialize all dead cells then update alive ones
        let mut cells = vec![Cell::Dead; (width * height) as usize];
        for (alive_cell_row, alive_cell_column) in alive_cells {
            cells[(alive_cell_row * width + alive_cell_column) as usize] = Cell::Alive;
        }

        Universe { width, height, cells }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn alive_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut alive_count = 0;

        for row_iter in [self.height - 1, 0, 1].iter().cloned() {
            for column_iter in [self.width - 1, 0, 1].iter().cloned() {
                if row_iter == 0 && column_iter == 0 {
                    continue // Don't count our own cell, we only want the neighbours
                }

                // Do these have overflow issues if height/width == 2**32?
                let neighbour_row = (row + row_iter) % self.height;
                let neighbour_column = (column + column_iter) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_column);
                alive_count += self.cells[idx] as u8;
            }
        }

        alive_count
    }

    fn cell_transform(current_cell: Cell, alive_neighbour_count: u8) -> Cell {

        // Apply Conway's Game of Life rules
        let new_cell = match (current_cell, alive_neighbour_count) {
            (Cell::Alive, x) if x < 2 || x > 3 => Cell::Dead,
            (Cell::Alive, _) => Cell::Alive,
            (Cell::Dead, 3) => Cell::Alive,
            (Cell::Dead, _) => Cell::Dead,
        };

        new_cell
    }

    pub fn tick(&mut self) {
        let mut new_cells = self.cells.clone();

        // Loop over all cells in the game
        for row in 0..self.height {
            for column in 0..self.width {
                // Grab current cell state and neighbour alive count
                let idx = self.get_index(row, column);
                let current_cell = self.cells[idx];
                let alive_neighbour_count = self.alive_neighbour_count(row, column);

                // Update the cell with the next value
                new_cells[idx] = Self::cell_transform(current_cell, alive_neighbour_count);
            }
        }

        self.cells = new_cells; // Overwrite cell array
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub fn generate_universe(width: u32, height: u32) -> Universe {
    // Not sure the best way to define the initial state of the board

    let mut rng = rand::thread_rng();

    let mut init_alive_cells = vec![];
    for row in 0..width {
        for column in 0..height {
            let random_number: u8 = rng.gen();
            if random_number < 32 {
                init_alive_cells.push((row, column));
            }
        }
    }

    Universe::new(width, height, init_alive_cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_index() {
        let universe = Universe::new(4, 4, vec![]);
        assert_eq!(2, universe.get_index(0, 2));
        assert_eq!(10, universe.get_index(2, 2));

        let universe = Universe::new(10, 10, vec![]);
        assert_eq!(2, universe.get_index(0, 2));
        assert_eq!(22, universe.get_index(2, 2));
    }

    #[test]
    fn alive_count() {
        // None alive
        let universe = Universe::new(4, 4, vec![]);
        assert_eq!(0, universe.alive_neighbour_count(2, 2));

        // Row alive
        let universe = Universe::new(4, 4, vec![(0, 2), (2, 2)]);
        assert_eq!(0, universe.alive_neighbour_count(2, 2));
        assert_eq!(2, universe.alive_neighbour_count(1, 2));
        assert_eq!(1, universe.alive_neighbour_count(0, 3));

        // Column alive
        let universe = Universe::new(4, 4, vec![(2, 0), (2, 2)]);
        assert_eq!(0, universe.alive_neighbour_count(2, 2));
        assert_eq!(2, universe.alive_neighbour_count(2, 1));
        assert_eq!(1, universe.alive_neighbour_count(3, 0));

        // TODO: Check wraps
    }

    #[test]
    fn cell_transform() {
        // Alive transformations
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Alive, 0));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Alive, 1));
        assert_eq!(Cell::Alive, Universe::cell_transform(Cell::Alive, 2));
        assert_eq!(Cell::Alive, Universe::cell_transform(Cell::Alive, 3));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Alive, 4));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Alive, 5));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Alive, 6));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Alive, 7));

        // Dead transformations
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 0));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 1));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 2));
        assert_eq!(Cell::Alive, Universe::cell_transform(Cell::Dead, 3));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 4));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 5));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 6));
        assert_eq!(Cell::Dead, Universe::cell_transform(Cell::Dead, 7));
    }

    #[test]
    fn test_generate_universe() {
        let mut universe = generate_universe(64, 64);
        println!("{}", universe.render());
        universe.tick();
        println!("{}", universe.render());
    }
}
