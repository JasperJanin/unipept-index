pub struct LCG { // https://rosettacode.org/wiki/Linear_congruential_generator#Rust
    state: u32
}

impl LCG {
    pub fn get(&mut self, upper: usize) -> usize {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        self.state %= 1 << 31;
        self.state as usize % upper
    }
    pub fn from_seed(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn get_char_i(&mut self) -> u8 {
        1 + self.get(25) as u8
    }
}

pub struct SearchSchemePass {
    pub order: Vec<u32>,
    pub lower: Vec<u32>,
    pub upper: Vec<u32>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SearchMethod {
    Stack,
    Dynamic
}
