// Local fields and p-adic numbers implementation

use crate::error::{Error, Result};
use num_bigint::{BigInt, BigUint};
use num_traits::{Zero, One, ToPrimitive};
use serde::{Serialize, Deserialize};
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::fmt;

/// Represents a local field (completion of a number field at a place)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalField {
    /// p-adic field Q_p
    Padic {
        prime: u32,
        precision: u32,
    },
    /// Archimedean completion (R or C)
    Archimedean {
        is_complex: bool,
    },
    /// Formal Laurent series field k((t))
    FormalLaurent {
        base_field: Box<LocalField>,
        variable: String,
    },
}

impl LocalField {
    /// Create a p-adic field Q_p
    pub fn p_adic(prime: u32, precision: u32) -> Self {
        LocalField::Padic { prime, precision }
    }

    /// Create the real numbers as a local field
    pub fn real() -> Self {
        LocalField::Archimedean { is_complex: false }
    }

    /// Create the complex numbers as a local field
    pub fn complex() -> Self {
        LocalField::Archimedean { is_complex: true }
    }

    /// Create a formal Laurent series field
    pub fn formal_laurent(base_field: LocalField, variable: &str) -> Self {
        LocalField::FormalLaurent {
            base_field: Box::new(base_field),
            variable: variable.to_string(),
        }
    }

    /// Get the residue field
    pub fn residue_field(&self) -> Result<LocalField> {
        match self {
            LocalField::Padic { prime, .. } => {
                // Residue field is F_p
                Ok(LocalField::finite_field(*prime))
            }
            LocalField::FormalLaurent { base_field, .. } => {
                // Residue field is the base field
                Ok((**base_field).clone())
            }
            LocalField::Archimedean { .. } => {
                Err(Error::InvalidInput("Archimedean fields have no residue field".into()))
            }
        }
    }

    /// Get the characteristic
    pub fn characteristic(&self) -> u32 {
        match self {
            LocalField::Padic { .. } => 0,
            LocalField::Archimedean { .. } => 0,
            LocalField::FormalLaurent { base_field, .. } => base_field.characteristic(),
        }
    }

    /// Check if the field is non-archimedean
    pub fn is_non_archimedean(&self) -> bool {
        !matches!(self, LocalField::Archimedean { .. })
    }

    /// Get ramification set
    pub fn ramification_set(&self) -> Vec<u32> {
        match self {
            LocalField::Padic { prime, .. } => vec![*prime],
            LocalField::FormalLaurent { .. } => vec![0], // Uniformizer ramifies
            LocalField::Archimedean { .. } => vec![],
        }
    }

    // Helper to create finite field (for residue fields)
    fn finite_field(p: u32) -> Self {
        // For now, represent as Z/pZ
        LocalField::Padic { prime: p, precision: 1 }
    }
}

impl Default for LocalField {
    fn default() -> Self {
        LocalField::p_adic(2, 32)
    }
}

/// Element of a local field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalFieldElement {
    /// p-adic number
    Padic(PadicNumber),
    /// Real or complex number
    Archimedean(ComplexNumber),
    /// Element of formal Laurent series
    FormalLaurent(LaurentSeries),
}

/// p-adic number representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadicNumber {
    /// The prime p
    pub prime: u32,
    /// Precision (number of digits)
    pub precision: u32,
    /// Valuation (power of p in denominator)
    pub valuation: i32,
    /// Digits in base p (least significant first)
    pub digits: Vec<u32>,
}

impl PadicNumber {
    /// Create a new p-adic number
    pub fn new(prime: u32, precision: u32, valuation: i32, digits: Vec<u32>) -> Self {
        // Ensure digits are in range [0, p)
        let normalized_digits: Vec<u32> = digits.iter()
            .map(|&d| d % prime)
            .collect();
        
        PadicNumber {
            prime,
            precision,
            valuation,
            digits: normalized_digits,
        }
    }

    /// Create p-adic zero
    pub fn zero(prime: u32, precision: u32) -> Self {
        PadicNumber {
            prime,
            precision,
            valuation: 0,
            digits: vec![0; precision as usize],
        }
    }

    /// Create p-adic one
    pub fn one(prime: u32, precision: u32) -> Self {
        let mut digits = vec![0; precision as usize];
        digits[0] = 1;
        PadicNumber {
            prime,
            precision,
            valuation: 0,
            digits,
        }
    }

    /// Create from integer
    pub fn from_int(n: i64, prime: u32, precision: u32) -> Self {
        let mut digits = Vec::new();
        let mut val = n.abs() as u64;
        let mut valuation = 0;

        // Extract powers of p
        while val > 0 && val % prime as u64 == 0 {
            val /= prime as u64;
            valuation += 1;
        }

        // Convert to base p
        while val > 0 && digits.len() < precision as usize {
            digits.push((val % prime as u64) as u32);
            val /= prime as u64;
        }

        // Pad with zeros
        while digits.len() < precision as usize {
            digits.push(0);
        }

        // Handle negative numbers using p-adic representation
        if n < 0 {
            // Compute p-adic negative: -x = p^k - x where k is large enough
            let mut carry = 1;
            for digit in &mut digits {
                let sum = (prime - *digit) + carry;
                *digit = sum % prime;
                carry = sum / prime;
            }
        }

        PadicNumber {
            prime,
            precision,
            valuation: if n == 0 { i32::MAX } else { valuation },
            digits,
        }
    }

    /// Compute norm |x|_p = p^(-valuation)
    pub fn norm(&self) -> f64 {
        if self.is_zero() {
            0.0
        } else {
            (self.prime as f64).powf(-self.valuation as f64)
        }
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0)
    }

    /// Get the unit part (remove powers of p)
    pub fn unit_part(&self) -> Self {
        PadicNumber {
            prime: self.prime,
            precision: self.precision,
            valuation: 0,
            digits: self.digits.clone(),
        }
    }

    /// Hensel lifting for square roots
    pub fn sqrt(&self) -> Result<Self> {
        // Check if square root exists modulo p
        if self.is_zero() {
            return Ok(Self::zero(self.prime, self.precision));
        }

        // For odd valuation, no square root
        if self.valuation % 2 != 0 {
            return Err(Error::InvalidInput("No square root: odd valuation".into()));
        }

        // Initial approximation
        let mut x = Self::one(self.prime, self.precision);
        
        // Hensel's lemma iteration
        for _ in 0..self.precision {
            // x_{n+1} = x_n - (x_n^2 - a) / (2*x_n)
            let x_squared = x.clone() * x.clone();
            let diff = x_squared - self.clone();
            let two_x = x.clone() + x.clone();
            let correction = diff / two_x;
            x = x - correction;
        }

        // Adjust valuation
        x.valuation = self.valuation / 2;
        Ok(x)
    }

    /// Teichmüller lift
    pub fn teichmuller_lift(&self) -> Self {
        if self.is_zero() {
            return self.clone();
        }

        // Start with residue class representative
        let mut lift = Self::from_int(self.digits[0] as i64, self.prime, self.precision);
        
        // Apply Hensel lifting to find (p-1)th root of unity
        let p_minus_1 = self.prime - 1;
        for _ in 0..self.precision {
            let power = lift.pow(p_minus_1);
            let diff = power - Self::one(self.prime, self.precision);
            let derivative = Self::from_int(p_minus_1 as i64, self.prime, self.precision) * lift.pow(p_minus_1 - 1);
            let correction = diff / derivative;
            lift = lift - correction;
        }

        lift
    }

    /// Compute logarithm (for |x-1|_p < 1)
    pub fn log(&self) -> Result<Self> {
        // Check convergence condition
        let one = Self::one(self.prime, self.precision);
        let diff = self.clone() - one.clone();
        if diff.valuation <= 0 {
            return Err(Error::InvalidInput("log series does not converge".into()));
        }

        // Compute log(x) = log(1 + (x-1)) = sum_{n=1}^∞ (-1)^{n+1} (x-1)^n / n
        let mut result = Self::zero(self.prime, self.precision);
        let mut term = diff.clone();
        let mut sign = 1;
        
        for n in 1..self.precision {
            let n_inv = Self::from_int(sign * n as i64, self.prime, self.precision).inverse()?;
            result = result + term.clone() * n_inv;
            term = term * diff.clone();
            sign = -sign;
            
            // Check convergence
            if term.valuation > self.precision as i32 {
                break;
            }
        }

        Ok(result)
    }

    /// Compute exponential (for |x|_p < p^(-1/(p-1)))
    pub fn exp(&self) -> Result<Self> {
        // Check convergence condition
        let threshold = -1.0 / (self.prime as f64 - 1.0);
        if self.valuation as f64 > threshold {
            return Err(Error::InvalidInput("exp series does not converge".into()));
        }

        // Compute exp(x) = sum_{n=0}^∞ x^n / n!
        let mut result = Self::one(self.prime, self.precision);
        let mut term = Self::one(self.prime, self.precision);
        
        for n in 1..self.precision {
            term = term * self.clone();
            let n_factorial = Self::factorial(n, self.prime, self.precision);
            let n_factorial_inv = n_factorial.inverse()?;
            result = result + term.clone() * n_factorial_inv;
            
            // Check convergence
            if term.valuation > self.precision as i32 {
                break;
            }
        }

        Ok(result)
    }

    // Helper: compute n! as p-adic number
    fn factorial(n: u32, prime: u32, precision: u32) -> Self {
        let mut result = Self::one(prime, precision);
        for i in 2..=n {
            result = result * Self::from_int(i as i64, prime, precision);
        }
        result
    }

    // Helper: compute multiplicative inverse
    fn inverse(&self) -> Result<Self> {
        if self.is_zero() {
            return Err(Error::InvalidInput("Cannot invert zero".into()));
        }

        // Extended Euclidean algorithm to find inverse mod p^precision
        let mut a = self.unit_part();
        let mut result = Self::one(self.prime, self.precision);
        
        // Newton's method: x_{n+1} = x_n * (2 - a * x_n)
        for _ in 0..self.precision {
            let ax = a.clone() * result.clone();
            let two = Self::from_int(2, self.prime, self.precision);
            let factor = two - ax;
            result = result * factor;
        }

        // Adjust valuation
        result.valuation = -self.valuation;
        Ok(result)
    }

    fn pow(&self, n: u32) -> Self {
        let mut result = Self::one(self.prime, self.precision);
        let mut base = self.clone();
        let mut exp = n;

        while exp > 0 {
            if exp % 2 == 1 {
                result = result * base.clone();
            }
            base = base.clone() * base.clone();
            exp /= 2;
        }

        result
    }
}

// Arithmetic operations for p-adic numbers
impl Add for PadicNumber {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(self.prime, other.prime, "Cannot add p-adic numbers with different primes");
        
        // Align valuations
        let min_val = self.valuation.min(other.valuation);
        let self_shift = (self.valuation - min_val) as usize;
        let other_shift = (other.valuation - min_val) as usize;
        
        let mut result_digits = vec![0; self.precision as usize + 1];
        let mut carry = 0;
        
        for i in 0..self.precision as usize {
            let mut sum = carry;
            if i >= self_shift && i - self_shift < self.digits.len() {
                sum += self.digits[i - self_shift];
            }
            if i >= other_shift && i - other_shift < other.digits.len() {
                sum += other.digits[i - other_shift];
            }
            
            result_digits[i] = sum % self.prime;
            carry = sum / self.prime;
        }
        
        // Normalize (remove trailing zeros and adjust valuation)
        let mut actual_val = min_val;
        while !result_digits.is_empty() && result_digits[0] == 0 {
            result_digits.remove(0);
            actual_val += 1;
        }
        
        // Truncate to precision
        result_digits.truncate(self.precision as usize);
        
        PadicNumber {
            prime: self.prime,
            precision: self.precision,
            valuation: if result_digits.is_empty() { i32::MAX } else { actual_val },
            digits: result_digits,
        }
    }
}

impl Sub for PadicNumber {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl Neg for PadicNumber {
    type Output = Self;

    fn neg(self) -> Self {
        if self.is_zero() {
            return self;
        }

        // p-adic negative: -x = p^k - x for large k
        let mut digits = self.digits.clone();
        let mut carry = 1;
        
        for digit in &mut digits {
            let diff = (self.prime as i32 - *digit as i32 + carry - 1) as u32;
            *digit = diff % self.prime;
            carry = if diff >= self.prime { 1 } else { 0 };
        }

        PadicNumber {
            prime: self.prime,
            precision: self.precision,
            valuation: self.valuation,
            digits,
        }
    }
}

impl Mul for PadicNumber {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        assert_eq!(self.prime, other.prime, "Cannot multiply p-adic numbers with different primes");
        
        if self.is_zero() || other.is_zero() {
            return PadicNumber::zero(self.prime, self.precision);
        }

        // Multiply valuations
        let result_val = self.valuation.saturating_add(other.valuation);
        
        // Multiply digits (convolution)
        let mut result_digits = vec![0u64; (self.precision * 2) as usize];
        
        for i in 0..self.digits.len() {
            for j in 0..other.digits.len() {
                if i + j < result_digits.len() {
                    result_digits[i + j] += self.digits[i] as u64 * other.digits[j] as u64;
                }
            }
        }
        
        // Carry propagation
        let mut carry = 0u64;
        for digit in &mut result_digits {
            *digit += carry;
            carry = *digit / self.prime as u64;
            *digit %= self.prime as u64;
        }
        
        // Truncate to precision
        let final_digits: Vec<u32> = result_digits.iter()
            .take(self.precision as usize)
            .map(|&d| d as u32)
            .collect();

        PadicNumber {
            prime: self.prime,
            precision: self.precision,
            valuation: result_val,
            digits: final_digits,
        }
    }
}

impl Div for PadicNumber {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        assert_eq!(self.prime, other.prime, "Cannot divide p-adic numbers with different primes");
        
        if other.is_zero() {
            panic!("Division by zero");
        }

        if self.is_zero() {
            return PadicNumber::zero(self.prime, self.precision);
        }

        // Division: a/b = a * b^(-1)
        let inv = other.inverse().expect("Failed to compute inverse");
        self * inv
    }
}

impl fmt::Display for PadicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }

        // Display as sum of powers of p
        write!(f, "{}^{} * (", self.prime, self.valuation)?;
        
        for (i, &digit) in self.digits.iter().enumerate().take(5) {
            if i > 0 {
                write!(f, " + ")?;
            }
            write!(f, "{}*{}^{}", digit, self.prime, i)?;
        }
        
        if self.digits.len() > 5 {
            write!(f, " + ...")?;
        }
        
        write!(f, ")")
    }
}

/// Complex number for archimedean places
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexNumber {
    pub real: f64,
    pub imag: f64,
}

impl ComplexNumber {
    pub fn new(real: f64, imag: f64) -> Self {
        ComplexNumber { real, imag }
    }

    pub fn from_real(real: f64) -> Self {
        ComplexNumber { real, imag: 0.0 }
    }

    pub fn norm(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }

    pub fn arg(&self) -> f64 {
        self.imag.atan2(self.real)
    }

    pub fn conj(&self) -> Self {
        ComplexNumber {
            real: self.real,
            imag: -self.imag,
        }
    }
}

/// Laurent series for formal Laurent series fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaurentSeries {
    /// Coefficients indexed by power of t
    pub coefficients: Vec<LocalFieldElement>,
    /// Lowest power of t
    pub min_degree: i32,
    /// Variable name
    pub variable: String,
}

impl LaurentSeries {
    pub fn new(coefficients: Vec<LocalFieldElement>, min_degree: i32, variable: String) -> Self {
        LaurentSeries {
            coefficients,
            min_degree,
            variable,
        }
    }

    pub fn zero(variable: String) -> Self {
        LaurentSeries {
            coefficients: vec![],
            min_degree: 0,
            variable,
        }
    }

    pub fn valuation(&self) -> i32 {
        self.min_degree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padic_arithmetic() {
        let p = 5;
        let prec = 10;
        
        // Test addition
        let a = PadicNumber::from_int(17, p, prec);
        let b = PadicNumber::from_int(13, p, prec);
        let sum = a.clone() + b.clone();
        let expected = PadicNumber::from_int(30, p, prec);
        
        println!("17 + 13 = {} (p-adic)", sum);
        
        // Test multiplication
        let prod = a.clone() * b.clone();
        let expected_prod = PadicNumber::from_int(221, p, prec);
        println!("17 * 13 = {} (p-adic)", prod);
        
        // Test norm
        let x = PadicNumber::from_int(25, p, prec); // 25 = 5^2
        assert_eq!(x.valuation, 2);
        assert_eq!(x.norm(), 0.04); // 5^(-2)
    }

    #[test]
    fn test_padic_sqrt() {
        let p = 7;
        let prec = 10;
        
        // Test square root of 2 in Q_7
        let two = PadicNumber::from_int(2, p, prec);
        let sqrt2 = two.sqrt().unwrap();
        let check = sqrt2.clone() * sqrt2.clone();
        
        println!("sqrt(2) in Q_7 = {}", sqrt2);
        println!("sqrt(2)^2 = {}", check);
        
        // First few digits should give 2
        assert_eq!(check.digits[0], 2);
    }

    #[test]
    fn test_local_field_types() {
        let qp = LocalField::p_adic(5, 32);
        let r = LocalField::real();
        let c = LocalField::complex();
        
        assert!(qp.is_non_archimedean());
        assert!(!r.is_non_archimedean());
        assert!(!c.is_non_archimedean());
        
        let residue = qp.residue_field().unwrap();
        assert_eq!(qp.ramification_set(), vec![5]);
    }
}