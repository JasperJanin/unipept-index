use std::cmp::{max, min};

pub struct BandedMatrix {
    matrix: Vec<i32>,
    w: i32, // width of rows from the diagonal
    m: i32, // rows
    n: i32, // columns
    col_per_row: i32,
}

impl BandedMatrix {
    pub fn new(pattern_size: i32, w: i32, start_value: i32) -> Self {
        let m = pattern_size + w + 1;
        let col_per_row = (2 * w + 1) + 2;
        let mut r = Self {
            matrix: vec![0; (m * col_per_row) as usize],
            w,
            m,                   // rows
            n: pattern_size + 1, // cols
            col_per_row,
        };
        r.init(start_value);
        r
    }

    fn init(&mut self, start_value: i32) {
        // init top row and left col
        for i in 0..self.w + 2 {
            self.set(0, i, i + start_value);
            self.set(i, 0, i + start_value);
        }
        // init max elements at the sides
        // first elements on rows [1, w]
        for i in 1..(self.w + 1) {
            self.set(i, i + self.w + 1, self.w + 1 + start_value);
        }

        // rows [w+1, x]
        for i in (self.w + 1)..(self.n - self.w - 1) {
            self.set(i, i + self.w + 1, self.w + 1 + start_value);
        }

        // last rows
        for i in max(self.n - (self.w + 1), self.w + 1)..self.m {
            self.set(i, i - self.w - 1, self.w + 1 + start_value);
        }
    }

    fn set(&mut self, row: i32, col: i32, value: i32) {
        self.matrix[(row * self.col_per_row + col - row + self.w) as usize] = value;
    }

    fn get(&self, row: i32, col: i32) -> i32 {
        self.matrix[(row * self.col_per_row + col - row + self.w) as usize]
    }

    pub fn get_row_count(&self) -> i32 {
        self.m
    }

    pub fn get_first_col(&self, row: i32) -> i32 {
        max(1, row - self.w)
    }
    pub fn get_last_col(&self, row: i32) -> i32 {
        min(self.n - 1, row + self.w)
    }

    pub fn in_final_col(&self, row: i32) -> bool {
        assert!(row < self.get_row_count());
        self.get_last_col(row) == self.n - 1
    }

    pub fn get_final_col_value(&self, row: i32) -> i32 {
        assert!(self.in_final_col(row));
        self.get(row, self.n - 1)
    }

    fn update_matrix_cell(&mut self, mismatch: bool, row: i32, col: i32) -> i32 {
        let mut dist = (if mismatch { 1 } else { 0 }) + self.get(row - 1, col - 1);
        if self.within_band(row - 1, col) {
            dist = min(dist, self.get(row - 1, col) + 1);
        }
        if self.within_band(row, col - 1) {
            dist = min(dist, self.get(row, col - 1) + 1);
        }
        self.set(row, col, dist);
        dist
    }

    pub fn update_matrix_row(&mut self, pattern: &str, row: i32, c: u8, forward: bool) -> i32 {
        if row >= self.m {
            return i32::MAX;
        }

        let mut min_val = i32::MAX;
        for col in self.get_first_col(row)..self.get_last_col(row) + 1 {
            let mismatch =
                pattern.as_bytes()[if forward { col as usize - 1 } else { pattern.len() - col as usize }] != c;
            min_val = min(min_val, self.update_matrix_cell(mismatch, row, col));
        }
        min_val
    }

    pub fn within_band(&self, row: i32, col: i32) -> bool {
        if row >= self.m || col >= self.n {
            return false;
        }
        (row - col).abs() <= self.w
    }

    #[allow(dead_code)]
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
