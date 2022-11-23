use dyn_stack::{DynStack, GlobalMemBuffer, ReborrowMut};
use faer_core::{Mat, Parallelism};
use ndarray_linalg::solve::Factorize;
use rand::random;
use std::time::Duration;
use timeit::*;

pub fn ndarray(sizes: &[usize]) -> Vec<Duration> {
    sizes
        .iter()
        .copied()
        .map(|n| {
            let mut c = ndarray::Array::<f64, _>::zeros((n, n));
            for i in 0..n {
                for j in 0..n {
                    c[(i, j)] = random();
                }
            }

            let time = timeit_loops! {10, {
                c.factorize().unwrap();
            }};

            time
        })
        .map(Duration::from_secs_f64)
        .collect()
}

pub fn nalgebra(sizes: &[usize]) -> Vec<Duration> {
    sizes
        .iter()
        .copied()
        .map(|n| {
            let mut c = nalgebra::DMatrix::<f64>::zeros(n, n);
            for i in 0..n {
                for j in 0..n {
                    c[(i, j)] = random();
                }
            }

            let time = timeit_loops! {10, {
                nalgebra::linalg::LU::new(c.clone());
            }};

            time
        })
        .map(Duration::from_secs_f64)
        .collect()
}

pub fn faer(sizes: &[usize], parallelism: Parallelism) -> Vec<Duration> {
    sizes
        .iter()
        .copied()
        .map(|n| {
            let mut c = Mat::<f64>::zeros(n, n);
            for i in 0..n {
                for j in 0..n {
                    c[(i, j)] = random();
                }
            }
            let mut lu = Mat::<f64>::zeros(n, n);
            let mut row_fwd = vec![0; n];
            let mut row_inv = vec![0; n];

            let mut mem = GlobalMemBuffer::new(
                faer_lu::partial_pivoting::lu_in_place_req::<f64>(n, n, parallelism).unwrap(),
            );
            let mut stack = DynStack::new(&mut mem);

            let mut block = || {
                lu.as_mut()
                    .cwise()
                    .zip(c.as_ref())
                    .for_each(|dst, src| *dst = *src);
                faer_lu::partial_pivoting::lu_in_place(
                    lu.as_mut(),
                    &mut row_fwd,
                    &mut row_inv,
                    parallelism,
                    stack.rb_mut(),
                );
            };

            let time = timeit_loops! {10, {
                block()
            }};

            let _ = c;

            time
        })
        .map(Duration::from_secs_f64)
        .collect()
}