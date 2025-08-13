use std::cmp::{max, min};
use std::process::exit;
use crate::search::BDFMSearch;

pub(crate) struct SearchSchemePass {
    pub(crate) order: Vec<u32>,
    pub(crate) lower: Vec<u32>,
    pub(crate) upper: Vec<u32>,
}

pub enum SearchMethod {
    Stack,
    Dynamic
}

pub struct Range {
    pub(crate) begin: u64,
    pub(crate) end: u64
}

impl Range {
    pub fn new(begin: u64, end: u64) -> Range {
        Range {
            begin, end
        }
    }

    pub fn get_begin(&self) -> u64 {self.begin}
    pub fn get_end(&self) -> u64 {self.end}
    
    pub fn empty(&self) -> bool {
        self.begin >= self.end
    }
    
    pub fn width(&self) -> u64 {
        if self.empty() { 0 } else { self.end - self.begin } 
    }
}

pub struct RangePair {
    backward_range: Range,
    forward_range: Range
}

pub struct BiFMPos {
    backward_range: Range,
    forward_range: Range,
    depth: u32
}

pub struct BiFMPosExt {
    pos: BiFMPos,
    c: char
}

pub struct SearchDepthChar<'a> { // bifmext
    pub search: BDFMSearch<'a>,
    pub depth: u32,
    pub char: u8,
}
pub struct SearchDepthDist<'a> { // bifmocc
    pub search: BDFMSearch<'a>,
    pub depth: u32,
    pub dist: u32,
}

pub struct PassDetails {
    pub(crate) order: Vec<u32>,
    pub(crate) lower: Vec<u32>,
    pub(crate) upper: Vec<u32>,
    pub(crate) forward: Vec<bool>,
}

pub struct BandedMatrix {
    matrix: Vec<u32>,
    W: u32, // width of rows from the diagonal
    m: u32, // rows
    n: u32, // columns
    col_per_row: u32
}

impl BandedMatrix {
    pub fn new(pattern_size: u32, W: u32, start_value: u32) -> Self {
        let m = pattern_size + W + 1;
        let col_per_row = (2*W+1) + 2;
        let mut r = Self {
            matrix: vec![0; (m * col_per_row) as usize],
            W,
            m, // rows
            n: pattern_size + 1, // cols
            col_per_row
        };
        r.init(start_value);
        r
    }

    fn init(&mut self, start_value: u32) {
        // init top row and left col
        for i in 0..self.W+2 {
            self.set(0, i, i + start_value);
            self.set(i, 0, i + start_value);
        }
        // init max elements at the sides
        // first elements on rows [1, W]
        for i in 1..(self.W+1) {
            self.set(i, i+self.W+1, self.W + 1 + start_value);
        }
        
        // rows [W+1, x]
        for i in (self.W+1)..(self.n-self.W-1) {
            self.set(i, i+self.W+1, self.W+1+start_value);
        }
        
        // last rows
        for i in max(self.n-(self.W+1), self.W+1)..self.m {
            self.set(i, i-self.W-1, self.W+1+start_value);
        }
    }

    fn set(&mut self, row: u32, col: u32, value: u32) {
        // println!("Setting M({row}, {col}) to {value}");
        self.matrix[(row * self.col_per_row + col - row + self.W) as usize] = value;
        // self.print_matrix();
    }
    
    fn get(&self, row: u32, col: u32) -> u32 {
        self.matrix[(row * self.col_per_row + col - row + self.W) as usize]
    }

    pub fn get_row_count(&self) -> u32 {
        self.m
    }

    pub fn get_first_col(&self, row: u32) -> u32 {
        max(1, row as i32 - self.W as i32) as u32
    }
    pub fn get_last_col(&self, row: u32) -> u32 {
        min(self.n as i32 - 1, (row + self.W) as i32) as u32
    }

    pub fn in_final_col(&self, row: u32) -> bool {
        assert!(row < self.get_row_count());
        self.get_last_col(row) == self.n - 1
    }

    pub fn get_final_col_value(&self, row: u32) -> u32 {
        assert!(self.in_final_col(row));
        self.get(row, self.n-1)
    }

    fn update_matrix_cell(&mut self, mismatch: bool, row: u32, col: u32) -> u32 {
        let mut dist = (if mismatch {1} else {0} ) + self.get(row-1, col-1);
        if self.within_band(row-1, col) { dist = min(dist, self.get(row-1, col) + 1); }
        if self.within_band(row, col-1) { dist = min(dist, self.get(row, col-1) + 1); }
        self.set(row, col, dist);
        dist
    }

    pub fn update_matrix_row(&mut self, pattern: &str, row: u32, c: u8, forward: bool) -> u32 {
        if row >= self.m {
            return u32::MAX;
        }

        let mut min_val = u32::MAX;
        for col in self.get_first_col(row) .. self.get_last_col(row)+1 {
            let mismatch = pattern.as_bytes()[if forward {col as usize - 1} else {pattern.len() - col as usize}] != c;
            min_val = min(min_val, self.update_matrix_cell(mismatch, row, col));
        }
        min_val
    }

    pub fn within_band(&self, row: u32, col: u32) -> bool {
        if row >= self.m || col >= self.n { return false; }
        (row as i32 - col as i32).abs() as u32 <= self.W
    }

    pub fn print_matrix(&self) {
        print!("Matrix ({} x {})\n===========\n", self.m, self.n);
        for i in 0..self.m {
            for j in 0..self.n {
                if self.within_band(i, j) {
                    print!("{}", self.get(i, j));
                } else {
                    print!(" ");
                }
            }
            print!("\n");
        }
        println!();
    }
}
