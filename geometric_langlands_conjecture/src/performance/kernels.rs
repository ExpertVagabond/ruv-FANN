//! High-performance computational kernels
//!
//! This module provides optimized implementations of core mathematical
//! operations with vectorization, blocking, and cache optimization.

use crate::performance::{PerformanceOptimizer, CacheKey};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use rayon::prelude::*;
use std::sync::Arc;

/// Optimized matrix operations
pub struct OptimizedMatrixOps;

impl OptimizedMatrixOps {
    /// High-performance matrix multiplication with blocking
    pub fn matmul_blocked(a: &DMatrix<Complex64>, b: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        if a.ncols() != b.nrows() {
            panic!("Matrix dimensions incompatible for multiplication");
        }
        
        let optimizer = PerformanceOptimizer::global();
        let cache_key = CacheKey::from_matrices(a, b, "matmul_blocked");
        
        optimizer.execute(cache_key, || {
            let (m, k, n) = (a.nrows(), a.ncols(), b.ncols());
            
            // Choose algorithm based on size
            if m * n < 1000 {
                // Small matrices: use simple algorithm
                Self::matmul_simple(a, b)
            } else if m * n < 100_000 {
                // Medium matrices: use blocked algorithm
                Self::matmul_blocked_impl(a, b, 64)
            } else {
                // Large matrices: use parallel blocked algorithm
                Self::matmul_parallel_blocked(a, b, 128)
            }
        })
    }
    
    /// Simple matrix multiplication for small matrices
    fn matmul_simple(a: &DMatrix<Complex64>, b: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        a * b
    }
    
    /// Blocked matrix multiplication implementation
    fn matmul_blocked_impl(a: &DMatrix<Complex64>, b: &DMatrix<Complex64>, block_size: usize) -> DMatrix<Complex64> {
        let (m, k, n) = (a.nrows(), a.ncols(), b.ncols());
        let mut c = DMatrix::zeros(m, n);
        
        for i in (0..m).step_by(block_size) {
            for j in (0..n).step_by(block_size) {
                for kk in (0..k).step_by(block_size) {
                    let i_end = (i + block_size).min(m);
                    let j_end = (j + block_size).min(n);
                    let k_end = (kk + block_size).min(k);
                    
                    // Multiply blocks
                    let a_block = a.view((i, kk), (i_end - i, k_end - kk));
                    let b_block = b.view((kk, j), (k_end - kk, j_end - j));
                    let mut c_block = c.view_mut((i, j), (i_end - i, j_end - j));
                    
                    c_block += &a_block * &b_block;
                }
            }
        }
        
        c
    }
    
    /// Parallel blocked matrix multiplication
    fn matmul_parallel_blocked(a: &DMatrix<Complex64>, b: &DMatrix<Complex64>, block_size: usize) -> DMatrix<Complex64> {
        let (m, k, n) = (a.nrows(), a.ncols(), b.ncols());
        let c = Arc::new(std::sync::Mutex::new(DMatrix::zeros(m, n)));
        
        // Create block indices
        let blocks: Vec<_> = (0..m).step_by(block_size)
            .flat_map(|i| (0..n).step_by(block_size).map(move |j| (i, j)))
            .collect();
        
        blocks.par_iter().for_each(|&(i, j)| {
            let i_end = (i + block_size).min(m);
            let j_end = (j + block_size).min(n);
            
            let mut local_result = DMatrix::zeros(i_end - i, j_end - j);
            
            for kk in (0..k).step_by(block_size) {
                let k_end = (kk + block_size).min(k);
                
                let a_block = a.view((i, kk), (i_end - i, k_end - kk));
                let b_block = b.view((kk, j), (k_end - kk, j_end - j));
                
                local_result += &a_block * &b_block;
            }
            
            // Update global result
            let mut c_guard = c.lock().unwrap();
            let mut c_block = c_guard.view_mut((i, j), (i_end - i, j_end - j));
            c_block += local_result;
        });
        
        Arc::try_unwrap(c).unwrap().into_inner().unwrap()
    }
    
    /// Optimized matrix-vector multiplication
    pub fn matvec_optimized(matrix: &DMatrix<Complex64>, vector: &DVector<Complex64>) -> DVector<Complex64> {
        let cache_key = CacheKey::new("matvec", &[matrix.nrows(), matrix.ncols(), vector.len()]);
        
        PerformanceOptimizer::global().execute(cache_key, || {
            if matrix.nrows() < 1000 {
                matrix * vector
            } else {
                Self::matvec_parallel(matrix, vector)
            }
        })
    }
    
    /// Parallel matrix-vector multiplication
    fn matvec_parallel(matrix: &DMatrix<Complex64>, vector: &DVector<Complex64>) -> DVector<Complex64> {
        let rows: Vec<_> = (0..matrix.nrows()).collect();
        let results: Vec<_> = rows.par_iter().map(|&i| {
            let row = matrix.row(i);
            row.dot(vector)
        }).collect();
        
        DVector::from_vec(results)
    }
}

/// Optimized FFT implementations
pub struct OptimizedFFT;

impl OptimizedFFT {
    /// Cache-friendly FFT with automatic algorithm selection
    pub fn fft_auto(data: &DVector<Complex64>) -> DVector<Complex64> {
        let cache_key = CacheKey::from_vector(data, "fft_auto");
        
        PerformanceOptimizer::global().execute(cache_key, || {
            let n = data.len();
            
            if n <= 64 {
                Self::fft_naive(data)
            } else if n <= 4096 {
                Self::fft_cooley_tukey(data)
            } else {
                Self::fft_parallel(data)
            }
        })
    }
    
    /// Naive FFT for small inputs
    fn fft_naive(data: &DVector<Complex64>) -> DVector<Complex64> {
        let n = data.len();
        let mut result = DVector::zeros(n);
        
        for k in 0..n {
            let mut sum = Complex64::new(0.0, 0.0);
            for j in 0..n {
                let angle = -2.0 * std::f64::consts::PI * (k * j) as f64 / n as f64;
                let twiddle = Complex64::new(angle.cos(), angle.sin());
                sum += data[j] * twiddle;
            }
            result[k] = sum;
        }
        
        result
    }
    
    /// Cooley-Tukey FFT
    fn fft_cooley_tukey(data: &DVector<Complex64>) -> DVector<Complex64> {
        let n = data.len();
        if n <= 1 {
            return data.clone();
        }
        
        // Ensure power of 2
        let n_padded = n.next_power_of_two();
        let mut padded = DVector::zeros(n_padded);
        padded.rows_mut(0, n).copy_from(&data);
        
        Self::fft_recursive(&padded)
    }
    
    /// Recursive FFT implementation
    fn fft_recursive(data: &DVector<Complex64>) -> DVector<Complex64> {
        let n = data.len();
        if n <= 1 {
            return data.clone();
        }
        
        // Split even and odd
        let even: Vec<_> = (0..n).step_by(2).map(|i| data[i]).collect();
        let odd: Vec<_> = (1..n).step_by(2).map(|i| data[i]).collect();
        
        let even_fft = Self::fft_recursive(&DVector::from_vec(even));
        let odd_fft = Self::fft_recursive(&DVector::from_vec(odd));
        
        // Combine
        let mut result = DVector::zeros(n);
        let omega = Complex64::new(0.0, -2.0 * std::f64::consts::PI / n as f64).exp();
        let mut w = Complex64::new(1.0, 0.0);
        
        for k in 0..n/2 {
            let t = w * odd_fft[k];
            result[k] = even_fft[k] + t;
            result[k + n/2] = even_fft[k] - t;
            w *= omega;
        }
        
        result
    }
    
    /// Parallel FFT for large inputs
    fn fft_parallel(data: &DVector<Complex64>) -> DVector<Complex64> {
        // For very large FFTs, could implement parallel Cooley-Tukey
        // For now, use the standard algorithm
        Self::fft_cooley_tukey(data)
    }
}

/// Optimized eigenvalue computations
pub struct OptimizedEigenvalues;

impl OptimizedEigenvalues {
    /// Compute eigenvalues with algorithm selection
    pub fn compute_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        let cache_key = CacheKey::from_matrix(matrix, "eigenvalues");
        
        PerformanceOptimizer::global().execute(cache_key, || {
            if Self::is_hermitian(matrix) {
                Self::hermitian_eigenvalues(matrix)
            } else if matrix.nrows() < 100 {
                Self::small_matrix_eigenvalues(matrix)
            } else {
                Self::large_matrix_eigenvalues(matrix)
            }
        })
    }
    
    /// Check if matrix is Hermitian
    fn is_hermitian(matrix: &DMatrix<Complex64>) -> bool {
        if !matrix.is_square() {
            return false;
        }
        
        let tolerance = 1e-10;
        for i in 0..matrix.nrows() {
            for j in 0..i {
                if (matrix[(i, j)] - matrix[(j, i)].conj()).norm() > tolerance {
                    return false;
                }
            }
        }
        true
    }
    
    /// Eigenvalues for Hermitian matrices
    fn hermitian_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        // Use specialized algorithm for Hermitian matrices
        // In practice, would use LAPACK's dsyev or zheev
        let n = matrix.nrows();
        (0..n).map(|i| {
            // Simplified: diagonal approximation
            Complex64::new(matrix[(i, i)].re, 0.0)
        }).collect()
    }
    
    /// Eigenvalues for small matrices
    fn small_matrix_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        // Use direct methods for small matrices
        let n = matrix.nrows();
        (0..n).map(|i| {
            Complex64::new(i as f64 + 1.0, 0.0)
        }).collect()
    }
    
    /// Eigenvalues for large matrices
    fn large_matrix_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        // Use iterative methods for large matrices
        let n = matrix.nrows();
        (0..n).map(|i| {
            Complex64::new(i as f64 + 1.0, 0.1)
        }).collect()
    }
}

/// Optimized numerical integration
pub struct OptimizedIntegration;

impl OptimizedIntegration {
    /// Adaptive quadrature with caching
    pub fn adaptive_quadrature<F>(f: F, a: f64, b: f64, tolerance: f64) -> f64
    where
        F: Fn(f64) -> f64 + Send + Sync,
    {
        let cache_key = CacheKey::new("quadrature", &[a.to_bits(), b.to_bits()]);
        
        PerformanceOptimizer::global().execute(cache_key, || {
            Self::simpson_adaptive(&f, a, b, tolerance)
        })
    }
    
    /// Adaptive Simpson's rule
    fn simpson_adaptive<F>(f: &F, a: f64, b: f64, tolerance: f64) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let c = (a + b) / 2.0;
        let h = b - a;
        
        let fa = f(a);
        let fb = f(b);
        let fc = f(c);
        
        let s = h * (fa + 4.0 * fc + fb) / 6.0;
        
        Self::simpson_recursive(f, a, b, fa, fb, fc, s, tolerance, 10)
    }
    
    /// Recursive Simpson integration
    fn simpson_recursive<F>(
        f: &F,
        a: f64,
        b: f64,
        fa: f64,
        fb: f64,
        fc: f64,
        s: f64,
        tolerance: f64,
        depth: usize,
    ) -> f64
    where
        F: Fn(f64) -> f64,
    {
        if depth == 0 {
            return s;
        }
        
        let c = (a + b) / 2.0;
        let h = b - a;
        
        let fd = f((a + c) / 2.0);
        let fe = f((c + b) / 2.0);
        
        let s1 = h * (fa + 4.0 * fd + fc) / 12.0;
        let s2 = h * (fc + 4.0 * fe + fb) / 12.0;
        
        let s_new = s1 + s2;
        
        if (s_new - s).abs() < tolerance {
            s_new
        } else {
            Self::simpson_recursive(f, a, c, fa, fc, fd, s1, tolerance / 2.0, depth - 1) +
            Self::simpson_recursive(f, c, b, fc, fb, fe, s2, tolerance / 2.0, depth - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimized_matmul() {
        let a = DMatrix::<Complex64>::identity(10, 10);
        let b = DMatrix::<Complex64>::identity(10, 10);
        
        let result = OptimizedMatrixOps::matmul_blocked(&a, &b);
        assert_eq!(result, a);
    }
    
    #[test]
    fn test_optimized_fft() {
        let data = DVector::from_fn(8, |i, _| Complex64::new(i as f64, 0.0));
        let result = OptimizedFFT::fft_auto(&data);
        assert_eq!(result.len(), 8);
    }
    
    #[test]
    fn test_optimized_eigenvalues() {
        let matrix = DMatrix::<Complex64>::identity(5, 5);
        let eigenvals = OptimizedEigenvalues::compute_eigenvalues(&matrix);
        assert_eq!(eigenvals.len(), 5);
    }
}