
/// Pseudorandom number generator based on
/// https://rosettacode.org/wiki/Linear_congruential_generator#Rust
pub struct LCG {
    state: u32,
}

impl LCG {
    /// Get the next number between 0 and upper (excl.)
    pub fn get(&mut self, upper: usize) -> usize {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        self.state %= 1 << 31;
        self.state as usize % upper
    }
    /// Instantiate the LCG from a seed
    pub fn from_seed(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Get a random character
    pub fn get_char_i(&mut self) -> u8 {
        1 + self.get(25) as u8
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SearchMethod {
    Stack,
    Dynamic,
}
