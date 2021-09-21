#![allow(dead_code)]
// this warning is on by default, but I like the explicit nature of setting the value
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::redundant_field_names)]


//! `combinator` supports lazy iteration over the combinations of
//! copyable elements in a `Vec`.

// TODO: supporting _genlex_ order and element count order would be
// friendlier and/or more efficient in many scenarios

/// `BitMaskCombinator` uses the bits in a `usize` to create masks from
/// which it generates combinations of the elements in the underlying
/// vector.
struct BitMaskCombinator<T: Copy> {
    /// `curr_mask` is the current state of the iterator. It ranges from
    /// `usize::Max << vec.len()` to `usize::Max`.
    curr_mask: usize,

    /// A convenience value for dealing only with the bits involved in
    /// generating the combinations.
    mask: usize,

    /// The combinator will return only combinations that have at least
    /// this many elements.
    min: u32,

    /// The combinator will return only combinations that have at most
    /// this many elements.
    max: u32,

    /// The underlying `Vec` of copyable elements
    vec: Vec<T>,
}

/// Returns an iterator that lazily iterates over all unique combinations
/// of elements in `vector` with no attempt at specific ordering of the
/// combinations
///
/// The order of the elements _within_ each combination is stable and consistent
/// with their order in `vector`.
pub fn all_combinations<T: Copy>(vector: Vec<T>) -> impl Iterator<Item = Vec<T>> {
    BitMaskCombinator::<T> {
        curr_mask: usize::MAX << vector.len(),
        mask: usize::MAX << vector.len(),
        min: 1,
        max: vector.len() as u32,
        vec: vector,
    }
}

/// Returns an iterator that lazily iterates over all unique combinations
/// of elements in `vector` with at least `min` elements and no attempt at
/// specific ordering of the combinations
///
/// The order of the elements _within_ each combination is stable and consistent
/// with their order in `vector`.
pub fn all_combinations_of_min_size<T: Copy>(
    vector: Vec<T>,
    min: u32,
) -> impl Iterator<Item = Vec<T>> {
    BitMaskCombinator::<T> {
        curr_mask: usize::MAX << vector.len(),
        mask: usize::MAX << vector.len(),
        min: min,
        max: vector.len() as u32,
        vec: vector,
    }
}

#[allow(dead_code)]
/// Returns an iterator that lazily iterates over all unique combinations
/// of elements in `vector` with no more than `max` elements and no attempt at
/// specific ordering of the combinations
///
/// The order of the elements _within_ each combination is stable and consistent
/// with their order in `vector`.
pub fn all_combinations_of_max_size<T: Copy>(
    vector: Vec<T>,
    max: u32,
) -> impl Iterator<Item = Vec<T>> {
    BitMaskCombinator::<T> {
        curr_mask: usize::MAX << vector.len(),
        mask: usize::MAX << vector.len(),
        min: 1,
        max: max,
        vec: vector,
    }
}

/// Returns an iterator that lazily iterates over all unique combinations
/// of elements in `vector` with at least `min` elements, no more than
/// `max` elements, and no attempt at specific ordering of the combinations
///
/// The order of the elements _within_ each combination is stable and consistent
/// with their order in `vector`.
pub fn all_combinations_of_size<T: Copy>(
    vector: Vec<T>,
    min: u32,
    max: u32,
) -> impl Iterator<Item = Vec<T>> {
    BitMaskCombinator::<T> {
        curr_mask: usize::MAX << vector.len(),
        mask: usize::MAX << vector.len(),
        min: min,
        max: max,
        vec: vector,
    }
}

impl<T: Copy> Iterator for BitMaskCombinator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // until we find a combination in `min`-`max` or reach MAX

            if self.curr_mask == usize::MAX {
                return None; // the iterator has returned all combinations
            }

            // increment the mask and apply to `vec` to assemble a combination,
            // if the combination has a desired number of elements
            self.curr_mask += 1;
            let ones = (self.curr_mask ^ self.mask).count_ones();
            if self.min <= ones && ones <= self.max {
                let mut v = Vec::<T>::new();
                for (i, t) in self.vec.iter().enumerate() {
                    let bit = 2usize.pow(i as u32);
                    if (bit & self.curr_mask) > 0 {
                        v.push(*t)
                    }
                }
                return Some(v);
            }
        }
    }
}

#[cfg(test)]
mod combinator_tests {

    use super::*;

    #[test]
    fn one_item() {
        assert_eq!(all_combinations(vec!["a"]).count(), 1);
        assert_eq!(all_combinations(vec!["a"]).next(), Some(vec!("a")))
    }

    #[test]
    fn two_items() {
        assert_eq!(all_combinations(vec![1, 2]).count(), 3);
    }

    #[test]
    fn three_items() {
        assert_eq!(all_combinations(vec![1, 2, 3]).count(), 7);
    }

    #[test]
    fn four_items() {
        assert_eq!(all_combinations(vec!["a", "b", "c", "d"]).count(), 15);
    }

    #[test]
    fn five_items() {
        assert_eq!(all_combinations(vec![1, 2, 3, 4, 5]).count(), 31);
    }

    #[test]
    fn five_items_min_2() {
        assert_eq!(
            all_combinations_of_min_size(vec![1, 2, 3, 4, 5], 2).count(),
            26
        );
    }

    #[test]
    fn five_items_min_max_4() {
        assert_eq!(
            all_combinations_of_max_size(vec![1, 2, 3, 4, 5], 4).count(),
            30
        );
    }

    #[test]
    fn five_items_min_2_max_4() {
        assert_eq!(
            all_combinations_of_size(vec![1, 2, 3, 4, 5], 2, 4).count(),
            25
        );
    }
}
