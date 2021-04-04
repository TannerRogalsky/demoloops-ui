use std::ops::{Range, RangeBounds};

struct Tracking<'a> {
    parent_range: &'a mut Range<usize>,
    total_consumed: &'a mut usize,
}

pub struct InputStack<'a, T> {
    inner: &'a mut Vec<T>,
    range: Range<usize>,
    total_consumed: usize,
    tracking: Option<Tracking<'a>>,
}

impl<'a, T> InputStack<'a, T> {
    pub fn new<R: RangeBounds<usize>>(inner: &'a mut Vec<T>, range: R) -> Self {
        Self::with_parent(inner, range, None)
    }

    fn with_parent<R: RangeBounds<usize>>(
        inner: &'a mut Vec<T>,
        range: R,
        tracking: Option<Tracking<'a>>,
    ) -> Self {
        let mut range = range_assert(range, ..inner.len());
        if let Some(tracking) = &tracking {
            range.start += tracking.parent_range.start;
            range.end += tracking.parent_range.start;
        }
        Self {
            inner,
            range,
            total_consumed: 0,
            tracking,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.inner[self.range.clone()]
    }

    pub fn consume(self) -> std::vec::Drain<'a, T> {
        let drain = self.inner.drain(self.range);
        if let Some(tracking) = self.tracking {
            let len = drain.as_slice().len();
            *tracking.total_consumed += len;
            tracking.parent_range.end -= *tracking.total_consumed;
        }
        drain
    }

    pub fn sub<R: RangeBounds<usize>>(&mut self, range: R) -> InputStack<'_, T> {
        let tracking = match &mut self.tracking {
            None => Tracking {
                parent_range: &mut self.range,
                total_consumed: &mut self.total_consumed,
            },
            Some(tracking) => Tracking {
                parent_range: &mut self.range,
                total_consumed: &mut tracking.total_consumed,
            },
        };
        InputStack::with_parent(&mut self.inner, range, Some(tracking))
    }
}

pub type DerefIter<'a> = std::iter::Map<
    std::slice::Iter<'a, Box<dyn std::any::Any>>,
    fn(&Box<dyn std::any::Any>) -> &dyn std::any::Any,
>;
impl InputStack<'_, Box<dyn std::any::Any>> {
    pub fn deref_iter(&self) -> DerefIter<'_> {
        self.as_slice().iter().map(std::ops::Deref::deref)
    }
}

impl<T> AsRef<[T]> for InputStack<'_, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

fn range_assert<R>(range: R, bounds: std::ops::RangeTo<usize>) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let len = bounds.end;

    let start: std::ops::Bound<&usize> = range.start_bound();
    let start = match start {
        std::ops::Bound::Included(&start) => start,
        std::ops::Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| panic!("attempted to index slice from after maximum usize")),
        std::ops::Bound::Unbounded => 0,
    };

    let end: std::ops::Bound<&usize> = range.end_bound();
    let end = match end {
        std::ops::Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| panic!("attempted to index slice from after maximum usize")),
        std::ops::Bound::Excluded(&end) => end,
        std::ops::Bound::Unbounded => len,
    };

    if start > end {
        panic!("slice index starts at {} but ends at {}", start, end);
    }
    if end > len {
        panic!(
            "range end index {} out of range for slice of length {}",
            end, len
        );
    }

    std::ops::Range { start, end }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        let mut stack = vec![0, 1, 2, 3];
        let substack = InputStack::new(&mut stack, 2..);

        assert_eq!(substack.as_slice().iter().sum::<i32>(), 5);
        let mut iter = substack.consume();
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn sub_test() {
        let mut stack = vec![-1, 0, 1, 2, 3, 4];
        let mut substack1 = InputStack::new(&mut stack, 1..5);
        assert_eq!(substack1.as_slice(), &[0, 1, 2, 3]);

        {
            let mut substack2 = substack1.sub(1..=2);
            assert_eq!(substack2.as_slice(), &[1, 2]);

            {
                let substack3 = substack2.sub(..1);
                assert_eq!(substack3.consume().collect::<Vec<_>>(), vec![1]);
            }

            assert_eq!(substack2.consume().collect::<Vec<_>>(), vec![2]);
        }

        assert_eq!(substack1.consume().collect::<Vec<_>>(), vec![0, 3]);
        assert_eq!(stack, vec![-1, 4]);
    }

    #[test]
    fn box_test() {
        use std::any::Any;
        let mut stack: Vec<Box<dyn Any>> = vec![Box::new(5), Box::new("x")];
        let substack = InputStack::new(&mut stack, ..);

        let count = substack
            .deref_iter()
            .filter(|v| v.is::<&'static str>())
            .count();
        assert_eq!(count, 1);
    }
}
