mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
}

extern crate js_sys;
extern crate rand;
extern crate web_sys;

use rand::Rng;

#[wasm_bindgen]
extern "C" {
  // Use `js_namespace` here to bind `console.log(..)` instead of just
  // `log(..)`
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);

  // The `console.log` is quite polymorphic, so we can bind it with multiple
  // signatures. Note that we need to use `js_name` to ensure we always call
  // `log` in JS.
  #[wasm_bindgen(js_namespace = console, js_name = log)]
  fn log_u32(a: u32);

  // Multiple arguments too!
  #[wasm_bindgen(js_namespace = console, js_name = log)]
  fn log_many(a: &str, b: &str);
}

fn bare_bones() {
  log("Hello from Rust!");
  log_u32(42);
  log_many("Logging", "many values!");
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn using_a_macro() {
  console_log!("Hello {}!", "world");
  console_log!("Let's print some numbers...");
  console_log!("1 + 3 = {}", 1 + 3);
}

// And finally, we don't even have to define the `log` function ourselves! The
// `web_sys` crate already has it defined for us.

fn using_web_sys() {
  use web_sys::console;

  console::log_1(&"Hello using web-sys".into());

  let js: JsValue = 4.into();
  console::log_2(&"Logging arbitrary values looks like".into(), &js);
}

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

impl Cell {
  fn toggle(&mut self) {
    *self = match *self {
      Cell::Dead => Cell::Alive,
      Cell::Alive => Cell::Dead,
    };
  }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
  pub fn toggle_cell(&mut self, row: u32, column: u32) {
    let idx = self.get_index(row, column);
    self.cells[idx].toggle();
  }

  pub fn tick(&mut self) {
    let mut next = self.cells.clone();

    for row in 0..self.height {
      for col in 0..self.width {
        let idx = self.get_index(row, col);
        let cell = self.cells[idx];
        let live_neighbors = self.live_neighbor_count(row, col);

        let next_cell = match (cell, live_neighbors) {
          // Rule 1: Any live cell with fewer than two live neighbours
          // dies, as if caused by underpopulation.
          (Cell::Alive, x) if x < 2 => Cell::Dead,
          // Rule 2: Any live cell with two or three live neighbours
          // lives on to the next generation.
          (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
          // Rule 3: Any live cell with more than three live
          // neighbours dies, as if by overpopulation.
          (Cell::Alive, x) if x > 3 => Cell::Dead,
          // Rule 4: Any dead cell with exactly three live neighbours
          // becomes a live cell, as if by reproduction.
          (Cell::Dead, 3) => Cell::Alive,
          // All other cells remain in the same state.
          (otherwise, _) => otherwise,
        };

        next[idx] = next_cell;
      }
    }

    self.cells = next;
  }

  fn get_index(&self, row: u32, column: u32) -> usize {
    (row * self.width + column) as usize
  }

  fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
    let mut count = 0;
    for delta_row in [self.height - 1, 0, 1].iter().cloned() {
      for delta_col in [self.width - 1, 0, 1].iter().cloned() {
        if delta_row == 0 && delta_col == 0 {
          continue;
        }

        let neighbor_row = (row + delta_row) % self.height;
        let neighbor_col = (column + delta_col) % self.width;
        let idx = self.get_index(neighbor_row, neighbor_col);
        count += self.cells[idx] as u8;
      }
    }
    count
  }

  pub fn new() -> Universe {
    utils::set_panic_hook();

    let width = 256;
    let height = 256;

    console_log!("{}", width);

    log("Hello world!");

    web_sys::console::log_1(&"Hello, world!".into());

    let cells = (0..width * height)
      .map(|_i| {
        // if i % 9 == 0 || i % 2 == 0 {
        let mut rng = rand::thread_rng();
        let y: f64 = rng.gen(); // generates a float between 0 and 1
                                // if js_sys::Math::random() < 0.5 {
        if y < 0.5 {
          Cell::Alive
        } else {
          Cell::Dead
        }
      })
      .collect();

    Universe {
      width,
      height,
      cells,
    }
  }

  pub fn render(&self) -> String {
    self.to_string()
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  pub fn cells(&self) -> *const Cell {
    self.cells.as_ptr()
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
