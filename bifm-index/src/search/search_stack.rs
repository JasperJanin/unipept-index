use crate::bifm_index::BiFMIndex;
use crate::search::search::BiFMSearch;
use crate::search_scheme::{SearchScheme, SearchSchemePass};
use fm_index::BackwardSearchIndex;
use std::collections::HashSet;
use std::str;
use std::string::ToString;

/// Struct representing a 'state' to add to the serach stack
struct SearchStateEntry<'a> {
    /// Search ranges denoting the specific resulting intervals
    search: BiFMSearch<'a>,
    /// Errors previously encountered in this search
    errors: u8,
    /// Characters taken from the input pattern
    chars_taken: usize,
    /// Index of current part of the search pass
    scheme_part_index: usize,
    /// Denotes whether the current search direction is forward
    forward: bool,
    /// String slice of the pattern part that still needs matching
    remaining_string_slice: &'a str,
}

/// Struct to perform one search pass using the `Stack` algorithm
pub struct ApproximateStackSearch<'a> {
    /// Pattern parts
    parts: &'a Vec<String>,
    /// Total pattern length
    pattern_length: usize,
    /// Index for the pass in the search scheme
    scheme_pass: &'a SearchSchemePass,
    /// Set to write results to
    results: HashSet<u64>,
    /// State stack
    stack: Vec<SearchStateEntry<'a>>,
}

impl<'a> ApproximateStackSearch<'a> {
    /// Alphabet of characters to consider when matching
    // pub const ALPHABET: &'static [u8] = "ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    pub const ALPHABET: &'static [u8] = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes(); // TODO verander naar echt alfabet ^

    /// Returns a vector of text positions that contain an approximate match to the given pattern.
    ///
    /// # Arguments
    /// * `index` - The BiFMIndex containing the text
    /// * `pattern` - The pattern to find approximate matches for
    /// * `dist` - The maximally allowed edit distance
    pub fn search(index: &BiFMIndex, pattern: String, dist: usize) -> Vec<u64> {
        if pattern.len() <= dist {
            panic!("Pattern length must be greater than edit distance");
        }

        // Normal search when dist == 0
        if dist <= 0 {
            return index.normal_index.search_backward(pattern).locate();
        }
        
        // Initialise pattern parts and search scheme passes
        let scheme = SearchScheme::get(dist);
        let part_ct = scheme.passes[0].order.len();
        let chars_per_part = pattern.len() / part_ct;
        let mut parts = Vec::new();
        for i in 0..part_ct - 1 {
            parts.push(pattern[i * chars_per_part..(i + 1) * chars_per_part].to_string());
        }
        parts.push(pattern[(part_ct - 1) * chars_per_part..pattern.len()].to_string());

        let mut results = HashSet::new();
        
        // Perform passes
        for i in 0..scheme.pass_count {
            let mut search = ApproximateStackSearch {
                parts: &parts,
                pattern_length: pattern.len(),
                scheme_pass: &scheme.passes[i as usize],
                results: HashSet::new(),
                stack: Vec::new(),
            };

            search.stack.push(SearchStateEntry {
                search: BiFMSearch::new(&index),
                errors: 0,
                chars_taken: 0,
                scheme_part_index: 0,
                forward: false,
                remaining_string_slice: parts[scheme.passes[i as usize].order[0] as usize].as_str(),
            });
            
            // Iterate: while search space is not exhausted, extend search
            // from state at the top of the stack
            while search.stack.len() > 0 {
                let state = search.stack.pop().unwrap();

                // check for total result
                if state.chars_taken >= search.pattern_length {
                    state.search.locate().iter().for_each(|r| {
                        search.results.insert(*r);
                    });
                    continue;
                }

                search.extend_search(state);
            }
            
            // Add results from search to output
            search.results.iter().for_each(|r| {
                results.insert(*r);
            });
        }
        Vec::from_iter(results)
    }
    
    /// Extends a stack state into all possibilities and adds them to the stack.
    ///
    /// # Arguments
    /// * `state` - A stack state
    fn extend_search(&mut self, mut state: SearchStateEntry<'a>) {
        // If part is completely matched, transition to next search pass part
        if state.remaining_string_slice.len() == 0 {
            state.scheme_part_index += 1;
            state.remaining_string_slice =
                self.parts[self.scheme_pass.order[state.scheme_part_index] as usize].as_str();
            state.forward = self.scheme_pass.order[state.scheme_part_index] > self.scheme_pass.order[0];
        }

        let permitted_mistakes = self.scheme_pass.upper[state.scheme_part_index];

        // Get next char to match
        let next_char = if state.forward {
            state.remaining_string_slice.bytes().next().unwrap()
        } else {
            state.remaining_string_slice.bytes().last().unwrap()
        };

        // If additional errors allowed, match substitutions and indels
        if state.errors < permitted_mistakes as u8 {
            for c in Self::ALPHABET {
                let mut search = BiFMSearch {
                    index: state.search.index,
                    backward_s: state.search.backward_s,
                    backward_e: state.search.backward_e,
                    forward_s: state.search.forward_s,
                    forward_e: state.search.forward_e,
                };
                search = search.search_char(*c, state.forward);

                if search.count() > 0 {
                    // match/substitution
                    self.stack.push(SearchStateEntry {
                        search: BiFMSearch {
                            index: search.index,
                            backward_s: search.backward_s,
                            backward_e: search.backward_e,
                            forward_s: search.forward_s,
                            forward_e: search.forward_e,
                        },
                        errors: if *c == next_char { state.errors } else { state.errors + 1 },
                        chars_taken: state.chars_taken + 1,
                        scheme_part_index: state.scheme_part_index,
                        forward: state.forward,
                        remaining_string_slice: if state.forward {
                            &state.remaining_string_slice[1..]
                        } else {
                            &state.remaining_string_slice[..state.remaining_string_slice.len() - 1]
                        },
                    });

                    // insertion
                    self.stack.push(SearchStateEntry {
                        search,
                        errors: state.errors + 1,
                        chars_taken: state.chars_taken,
                        scheme_part_index: state.scheme_part_index,
                        forward: state.forward,
                        remaining_string_slice: state.remaining_string_slice,
                    });
                }
            }

            // deletion
            self.stack.push(SearchStateEntry {
                search: state.search,
                errors: state.errors + 1,
                chars_taken: state.chars_taken + 1,
                scheme_part_index: state.scheme_part_index,
                forward: state.forward,
                remaining_string_slice: if state.forward {
                    &state.remaining_string_slice[1..]
                } else {
                    &state.remaining_string_slice[..state.remaining_string_slice.len() - 1]
                },
            });
        }
        // If no more errors allowed in this part, perform exact match
        else {
            let search = state.search.search(state.remaining_string_slice, state.forward);
            if search.count() > 0 {
                state.search = search;
                state.chars_taken += state.remaining_string_slice.len();
                state.remaining_string_slice = "";
                self.stack.push(state);
            }
        }
    }
}
