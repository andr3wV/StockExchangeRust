use num::{traits::ToPrimitive, Float};
use std::ops::Deref;

pub fn kahan_sigma_return_counter<E, I: Iterator<Item = E>, F, Dtype>(
    element_iterator: I,
    op: F,
) -> (Dtype, usize)
where
    F: Fn(E) -> Dtype,
    Dtype: Float, {
    let mut count = 0usize;
    // Kahan summation algorithm
    let mut sum = Dtype::zero();
    let mut lower_bits = Dtype::zero();
    for a in element_iterator {
        count += 1;
        let y = op(a) - lower_bits;
        let new_sum = sum + y;
        lower_bits = (new_sum - sum) - y;
        sum = new_sum;
    }
    (sum, count)
}

#[inline]
pub fn mean<'a, A, T: Iterator<Item = &'a A>>(element_iterator: T) -> f64
where
    A: Copy + ToPrimitive + 'a,
    &'a A: Deref, {
    let (sum, count) =
        kahan_sigma_return_counter(element_iterator, |a| a.to_f64().unwrap());
    sum / count as f64
}

// `ddof` stands for delta degress of freedom, and the sum of squares will be
// divided by `count - ddof`, where `count` is the number of elements
// for population standard deviation, set `ddof` to 0
// for sample standard deviation, set `ddof` to 1
static DDOF: usize = 0;

#[inline]
pub fn variance<'a, T: Clone + Iterator<Item = &'a A>, A>(
    element_iterator: T,
) -> f64
where
    A: Copy + ToPrimitive + 'a,
    &'a A: Deref, {
    let mean = mean(element_iterator.clone());
    let (sum, count) = kahan_sigma_return_counter(element_iterator, move |a| {
        let a_f64 = a.to_f64().unwrap() - mean;
        a_f64 * a_f64
    });
    sum / (count - DDOF) as f64
}

#[inline]
pub fn standard_deviation<'a, T: Clone + Iterator<Item = &'a A>, A>(
    element_iterator: T,
) -> f64
where
    A: Copy + ToPrimitive + 'a,
    &'a A: Deref, {
    variance(element_iterator).sqrt()
}
