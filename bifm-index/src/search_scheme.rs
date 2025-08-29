/// A search scheme is a series of search passes to execute an approximate search query on
/// a bidirectional FM-index.
/// These optimal search schemes were proposed by Kianfar and Man
pub struct SearchScheme {
    /// The amount of passes
    pub pass_count: u32,
    /// The details of each pass
    pub passes: Vec<SearchSchemePass>,
}

/// A single search pass
pub struct SearchSchemePass {
    /// Order of the pattern fragments
    pub order: Vec<u32>,
    /// Lower bound of edit distance in each fragment (shown in chronological order, not pattern order)
    pub lower: Vec<u32>,
    /// Upper bound of edit distance in each fragmend (shown in chronological order, not pattern order)
    pub upper: Vec<u32>,
}

impl SearchScheme {
    pub fn get(dist: usize) -> Self {
        match dist {
            0 => SearchScheme {
                pass_count: 1,
                passes: vec![SearchSchemePass { order: vec![0], lower: vec![0], upper: vec![0] }],
            },

            1 => SearchScheme {
                pass_count: 2,
                passes: vec![
                    SearchSchemePass { order: vec![0, 1], lower: vec![0, 0], upper: vec![0, 1] },
                    SearchSchemePass { order: vec![1, 0], lower: vec![0, 1], upper: vec![0, 1] },
                ],
            },

            2 => SearchScheme {
                pass_count: 3,
                passes: vec![
                    SearchSchemePass {
                        order: vec![0, 1, 2],
                        lower: vec![0, 0, 2],
                        upper: vec![0, 1, 2],
                    },
                    SearchSchemePass {
                        order: vec![2, 1, 0],
                        lower: vec![0, 0, 0],
                        upper: vec![0, 2, 2],
                    },
                    SearchSchemePass {
                        order: vec![1, 2, 0],
                        lower: vec![0, 1, 1],
                        upper: vec![0, 1, 2],
                    },
                ],
            },

            3 => SearchScheme {
                pass_count: 3,
                passes: vec![
                    SearchSchemePass {
                        order: vec![0, 1, 2, 3],
                        lower: vec![0, 0, 0, 3],
                        upper: vec![0, 2, 3, 3],
                    },
                    SearchSchemePass {
                        order: vec![1, 2, 3, 0],
                        lower: vec![0, 0, 0, 0],
                        upper: vec![1, 2, 3, 3],
                    },
                    SearchSchemePass {
                        order: vec![2, 3, 1, 0],
                        lower: vec![0, 0, 2, 2],
                        upper: vec![0, 0, 3, 3],
                    },
                ],
            },

            4 => SearchScheme {
                pass_count: 5,
                passes: vec![
                    SearchSchemePass {
                        order: vec![0, 1, 2, 3, 4, 5],
                        lower: vec![0, 0, 0, 0, 0, 4],
                        upper: vec![0, 3, 3, 3, 4, 4],
                    },
                    SearchSchemePass {
                        order: vec![1, 2, 3, 4, 5, 0],
                        lower: vec![0, 0, 0, 0, 0, 0],
                        upper: vec![0, 2, 2, 3, 3, 4],
                    },
                    SearchSchemePass {
                        order: vec![2, 1, 3, 4, 5, 0],
                        lower: vec![0, 1, 1, 1, 1, 1],
                        upper: vec![0, 2, 2, 3, 3, 4],
                    },
                    SearchSchemePass {
                        order: vec![3, 2, 1, 4, 5, 0],
                        lower: vec![0, 1, 2, 2, 2, 2],
                        upper: vec![0, 1, 2, 3, 3, 4],
                    },
                    SearchSchemePass {
                        order: vec![5, 4, 3, 2, 1, 0],
                        lower: vec![0, 0, 0, 0, 3, 3],
                        upper: vec![0, 0, 4, 4, 4, 4],
                    },
                ],
            },

            _ => unimplemented!("No search schemes provided for distance 5 and up"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::search_scheme::SearchScheme;

    #[test]
    /// Test that `order`, `lower` and `upper` vectors are of equal size
    fn pass_sizes() {
        for i in 1..5 {
            let scheme = SearchScheme::get(i);
            for pass in &scheme.passes {
                assert_eq!(pass.order.len(), pass.lower.len());
                assert_eq!(pass.order.len(), pass.upper.len());
            }
        }
    }
}
