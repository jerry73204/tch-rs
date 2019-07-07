use std::ops::{
    RangeBounds,
    Range,
    RangeFrom,
    RangeFull,
    RangeInclusive,
    RangeTo,
    RangeToInclusive,
    Bound,
};
use super::Tensor;

// https://docs.scipy.org/doc/numpy/reference/arrays.indexing.html

pub struct NewAxis;

pub enum TensorIndexer {
    Select(i64),
    Narrow(Bound<i64>, Bound<i64>),
    IndexSelect(Tensor),
    InsertNewAxis,
}

impl From<NewAxis> for TensorIndexer {
    fn from(_index: NewAxis) -> Self {
        TensorIndexer::InsertNewAxis
    }
}

impl From<i64> for TensorIndexer {
    fn from(index: i64) -> Self {
        TensorIndexer::Select(index)
    }
}

impl From<&[i64]> for TensorIndexer {
    fn from(index: &[i64]) -> Self {
        let tensor = index.into();
        TensorIndexer::IndexSelect(tensor)
    }
}

impl From<&Tensor> for TensorIndexer {
    fn from(tensor: &Tensor) -> Self {
        use super::Kind::*;

        assert!(
            tensor.size().len() == 1,
            "Multi-dimensional tensor is not supported for indexing",
        );

        match tensor.kind() {
            Int64 =>
                TensorIndexer::IndexSelect(tensor.shallow_clone()),
            Int16 =>
                TensorIndexer::IndexSelect(tensor.shallow_clone()),
            Int8 =>
                TensorIndexer::IndexSelect(tensor.shallow_clone()),
            Int =>
                TensorIndexer::IndexSelect(tensor.shallow_clone()),
            _ => {
                panic!(
                    "the kind of tensors used as indices must be one of {:?}, {:?}, {:?}, {:?}",
                    Int64,
                    Int16,
                    Int8,
                    Int,
                );
            }
        }

    }
}

macro_rules! impl_from_range {
    ($range_type:ty) => {
        impl From<$range_type> for TensorIndexer {
            fn from(range: $range_type) -> Self {
                use std::ops::Bound::*;

                let start = match range.start_bound() {
                    Included(idx) => Included(*idx),
                    Excluded(idx) => Excluded(*idx),
                    Unbounded => Unbounded,
                };

                let end = match range.end_bound() {
                    Included(idx) => Included(*idx),
                    Excluded(idx) => Excluded(*idx),
                    Unbounded => Unbounded,
                };

                TensorIndexer::Narrow(start, end)
            }
        }
    }
}

impl_from_range!(Range<i64>);
impl_from_range!(RangeFrom<i64>);
impl_from_range!(RangeFull);
impl_from_range!(RangeInclusive<i64>);
impl_from_range!(RangeTo<i64>);
impl_from_range!(RangeToInclusive<i64>);

trait IndexOp<T> {
    fn i(&self, index: T) -> Tensor;
}

impl<A> IndexOp<A> for Tensor where
    A: Into<TensorIndexer>,
{
    fn i(&self, index: A) -> Tensor {
        self.indexer(&[index.into()])
    }
}

impl<A> IndexOp<(A,)> for Tensor where
    A: Into<TensorIndexer>,
{
    fn i(&self, index: (A,)) -> Tensor {
        let a = index.0.into();
        self.indexer(&[a])
    }
}

impl<A, B> IndexOp<(A, B)> for Tensor where
    A: Into<TensorIndexer>,
    B: Into<TensorIndexer>,
{
    fn i(&self, index: (A, B)) -> Tensor {
        let a = index.0.into();
        let b = index.1.into();
        self.indexer(&[a, b])
    }
}

impl<A, B, C> IndexOp<(A, B, C)> for Tensor where
    A: Into<TensorIndexer>,
    B: Into<TensorIndexer>,
    C: Into<TensorIndexer>,
{
    fn i(&self, index: (A, B, C)) -> Tensor {
        let a = index.0.into();
        let b = index.1.into();
        let c = index.2.into();
        self.indexer(&[a, b, c])
    }
}

impl<A, B, C, D> IndexOp<(A, B, C, D)> for Tensor where
    A: Into<TensorIndexer>,
    B: Into<TensorIndexer>,
    C: Into<TensorIndexer>,
    D: Into<TensorIndexer>,
{
    fn i(&self, index: (A, B, C, D)) -> Tensor {
        let a = index.0.into();
        let b = index.1.into();
        let c = index.2.into();
        let d = index.3.into();
        self.indexer(&[a, b, c, d])
    }
}

impl<A, B, C, D, E> IndexOp<(A, B, C, D, E)> for Tensor where
    A: Into<TensorIndexer>,
    B: Into<TensorIndexer>,
    C: Into<TensorIndexer>,
    D: Into<TensorIndexer>,
    E: Into<TensorIndexer>,
{
    fn i(&self, index: (A, B, C, D, E)) -> Tensor {
        let a = index.0.into();
        let b = index.1.into();
        let c = index.2.into();
        let d = index.3.into();
        let e = index.4.into();
        self.indexer(&[a, b, c, d, e])
    }
}

impl<A, B, C, D, E, F> IndexOp<(A, B, C, D, E, F)> for Tensor where
    A: Into<TensorIndexer>,
    B: Into<TensorIndexer>,
    C: Into<TensorIndexer>,
    D: Into<TensorIndexer>,
    E: Into<TensorIndexer>,
    F: Into<TensorIndexer>,
{
    fn i(&self, index: (A, B, C, D, E, F)) -> Tensor {
        let a = index.0.into();
        let b = index.1.into();
        let c = index.2.into();
        let d = index.3.into();
        let e = index.4.into();
        let f = index.5.into();
        self.indexer(&[a, b, c, d, e, f])
    }
}

impl Tensor {
    fn indexer(&self, index_spec: &[TensorIndexer]) -> Tensor {
        use std::ops::Bound::*;
        use TensorIndexer::*;

        assert!(
            index_spec.len() <= self.size().len(),
            format!("too many indices for tensor of dimension {}", self.size().len())
        );

        let mut curr_tensor = self.shallow_clone();
        let mut curr_idx: i64 = 0;

        for (_spec_idx, spec) in index_spec.iter().enumerate() {
            let dim_len = curr_tensor.size()[curr_idx as usize] as i64;

            let (next_tensor, next_idx) = match spec {
                InsertNewAxis => (
                    curr_tensor.unsqueeze(curr_idx),
                    curr_idx + 1,
                ),
                Select(index) => (
                    curr_tensor.select(curr_idx, *index),
                    curr_idx,  // curr_idx is not advanced because select() sequeezes dimension
                ),
                Narrow(Unbounded, Unbounded) => (
                    curr_tensor,
                    curr_idx + 1,
                ),
                Narrow(Included(start), Unbounded) => (
                    curr_tensor.narrow(curr_idx, *start, dim_len - *start),
                    curr_idx + 1,
                ),
                Narrow(Excluded(start), Unbounded) => (
                    curr_tensor.narrow(curr_idx, *start + 1, dim_len - *start - 1),
                    curr_idx + 1,
                ),
                Narrow(Unbounded, Included(end)) => (
                    curr_tensor.narrow(curr_idx, 0, *end + 1),
                    curr_idx + 1,
                ),
                Narrow(Unbounded, Excluded(end)) => (
                    curr_tensor.narrow(curr_idx, 0, *end),
                    curr_idx + 1,
                ),
                Narrow(Included(start), Included(end)) => (
                    curr_tensor.narrow(curr_idx, *start, *end - *start + 1),
                    curr_idx + 1,
                ),
                Narrow(Included(start), Excluded(end)) => (
                    curr_tensor.narrow(curr_idx, *start, *end - *start),
                    curr_idx + 1,
                ),
                Narrow(Excluded(start), Included(end)) => (
                    curr_tensor.narrow(curr_idx, *start + 1, *end - *start),
                    curr_idx + 1,
                ),
                Narrow(Excluded(start), Excluded(end)) => (
                    curr_tensor.narrow(curr_idx, *start + 1, *end - *start - 1),
                    curr_idx + 1,
                ),
                IndexSelect(index_tensor) => (
                    curr_tensor.index_select(curr_idx, index_tensor),
                    curr_idx + 1,
                )
            };

            curr_tensor = next_tensor;
            curr_idx = next_idx;
        }

        curr_tensor
    }
}
