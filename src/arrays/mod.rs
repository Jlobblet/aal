use itertools::Itertools;

pub mod array;
pub mod array_or_atom;
pub mod atom;
pub mod generic_array;
pub mod generic_matching_nouns;
pub mod matching_nouns;
pub mod noun;

pub type IntegerElt = i32;
pub type DecimalElt = f64;

fn odometer(range: &[usize]) -> Box<dyn Iterator<Item = Vec<usize>>> {
    Box::new(range.iter().map(|&w| 0..w).multi_cartesian_product().fuse())
}

#[cfg(test)]
mod test {
    use crate::arrays::odometer;

    #[test]
    fn test_odometer() {
        let mut mp = odometer(&[2, 3, 4]);
        assert_eq!(mp.next(), Some(vec![0, 0, 0]));
        assert_eq!(mp.next(), Some(vec![0, 0, 1]));
        assert_eq!(mp.next(), Some(vec![0, 0, 2]));
        assert_eq!(mp.next(), Some(vec![0, 0, 3]));
        assert_eq!(mp.next(), Some(vec![0, 1, 0]));
        assert_eq!(mp.next(), Some(vec![0, 1, 1]));
        assert_eq!(mp.next(), Some(vec![0, 1, 2]));
        assert_eq!(mp.next(), Some(vec![0, 1, 3]));
        assert_eq!(mp.next(), Some(vec![0, 2, 0]));
        assert_eq!(mp.next(), Some(vec![0, 2, 1]));
        assert_eq!(mp.next(), Some(vec![0, 2, 2]));
        assert_eq!(mp.next(), Some(vec![0, 2, 3]));
        assert_eq!(mp.next(), Some(vec![1, 0, 0]));
        assert_eq!(mp.next(), Some(vec![1, 0, 1]));
        assert_eq!(mp.next(), Some(vec![1, 0, 2]));
        assert_eq!(mp.next(), Some(vec![1, 0, 3]));
        assert_eq!(mp.next(), Some(vec![1, 1, 0]));
        assert_eq!(mp.next(), Some(vec![1, 1, 1]));
        assert_eq!(mp.next(), Some(vec![1, 1, 2]));
        assert_eq!(mp.next(), Some(vec![1, 1, 3]));
        assert_eq!(mp.next(), Some(vec![1, 2, 0]));
        assert_eq!(mp.next(), Some(vec![1, 2, 1]));
        assert_eq!(mp.next(), Some(vec![1, 2, 2]));
        assert_eq!(mp.next(), Some(vec![1, 2, 3]));
        assert_eq!(mp.next(), None);
    }
}
