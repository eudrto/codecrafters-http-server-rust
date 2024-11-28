/// Like `slice::split_mut` but takes a pattern.
/// Like `str::split` but returns an iterator over mutable references.
pub fn split_pattern_mut<'a, 'b, T>(
    me: &'a mut [T],
    pattern: &'b [T],
) -> SplitPatternMut<'a, 'b, T> {
    SplitPatternMut::new(me, pattern)
}

pub struct SplitPatternMut<'a, 'b, T> {
    s: Option<&'a mut [T]>,
    p: &'b [T],
}

impl<'a, 'b, T> SplitPatternMut<'a, 'b, T> {
    fn new(s: &'a mut [T], p: &'b [T]) -> Self {
        Self { s: Some(s), p }
    }
}

impl<'a, 'b, T: PartialEq> Iterator for SplitPatternMut<'a, 'b, T> {
    type Item = &'a mut [T];
    fn next(&mut self) -> Option<Self::Item> {
        let Some(s) = self.s.take() else { return None };

        let Some(pos) = position_pattern(s, self.p) else {
            return Some(s);
        };

        let (left, mut right) = s.split_at_mut(pos);
        right = &mut right[self.p.len()..];
        self.s.replace(right);
        Some(left)
    }
}

/// Like `str::split_once` but returns mutable references.
pub fn split_once_mut<'a, 'b, T: PartialEq>(
    me: &'a mut [T],
    pattern: &'b [T],
) -> Option<(&'a mut [T], &'a mut [T])> {
    let (left, mut right) = me.split_at_mut(position_pattern(me, pattern)?);
    right = &mut right[pattern.len()..];
    Some((left, right))
}

/// Like `Iterator::position` but takes a pattern.
///
/// # Returns
///
/// An index up to but excluding pattern.
fn position_pattern<T: PartialEq>(me: &[T], pattern: &[T]) -> Option<usize> {
    me.windows(pattern.len())
        .position(|window| window == pattern)
}

#[cfg(test)]
mod tests {
    use crate::slice_ext::position_pattern;

    use super::{split_once_mut, split_pattern_mut};

    #[test]
    fn test_split_pattern_mut_multi() {
        let crlf = "\r\n";
        let mut v = format!("x{0}y{0}z", crlf).into_bytes();
        split_pattern_mut(&mut v, crlf.as_bytes()).for_each(|part| part.make_ascii_uppercase());
        assert_eq!(v, format!("X{0}Y{0}Z", crlf).into_bytes());
    }

    #[test]
    fn test_split_pattern_mut_none() {
        let crlf = "\r\n";
        let mut v = b"xyz".to_vec();
        let mut it = split_pattern_mut(&mut v, crlf.as_bytes());
        assert_eq!(it.next().unwrap(), b"xyz");
        assert!(it.next().is_none());
    }

    #[test]
    fn test_split_once_mut_rm_pattern() {
        let crlf = "\r\n";
        let mut v = format!("x{0}y{0}z", crlf).into_bytes();

        let (left, right) = split_once_mut(&mut v, crlf.as_bytes()).unwrap();
        assert!(!left.ends_with(crlf.as_bytes()));
        assert!(!right.starts_with(crlf.as_bytes()));
    }

    #[test]
    fn test_split_once_mut_multi() {
        let crlf = "\r\n";
        let mut v = format!("x{0}y{0}z", crlf).into_bytes();

        let (left, _) = split_once_mut(&mut v, crlf.as_bytes()).unwrap();
        left.make_ascii_uppercase();
        assert_eq!(v, format!("X{0}y{0}z", crlf).into_bytes());

        let (_, right) = split_once_mut(&mut v, crlf.as_bytes()).unwrap();
        right.make_ascii_uppercase();
        assert_eq!(v, format!("X{0}Y{0}Z", crlf).into_bytes());
    }

    #[test]
    fn test_split_once_mut_none() {
        let crlf = "\r\n";
        let mut v = b"xyz".to_vec();
        assert!(split_once_mut(&mut v, crlf.as_bytes()).is_none());
    }

    #[test]
    fn test_position_pattern() {
        let crlf = "\r\n";
        let v = format!("x{0}y{0}z", crlf).into_bytes();
        assert_eq!(position_pattern(&v, crlf.as_bytes()).unwrap(), 1);
    }
}
