//! Mock implementations for testing

use crate::core::{AlgebraicVariety, Field, Group, LieAlgebra, Scheme, ModuliSpace};
use crate::representation::Representation;
use crate::sheaf::{LocalSystem, HiggsBundle, CoherentSheaf, SpectralCurve};
use crate::error::{Error, Result};
use num_complex::Complex64;
use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone)]
pub struct MockGroup {
    pub name: String,
    pub dim: usize,
}

impl MockGroup {
    pub fn new(name: String, dim: usize) -> Self {
        Self { name, dim }
    }
}

impl Group for MockGroup {
    fn dimension(&self) -> usize {
        self.dim
    }
    
    fn rank(&self) -> usize {
        self.dim / 2 // Simplified
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn identity(&self) -> Vec<f64> {
        vec![1.0; self.dim]
    }
    
    fn lie_algebra(&self) -> Result<Box<dyn LieAlgebra>> {
        Ok(Box::new(MockLieAlgebra {
            dim: self.dim,
            name: format!("Lie({})", self.name),
        }))
    }
    
    fn langlands_dual(&self) -> Result<Box<dyn Group>> {
        Ok(Box::new(MockGroup::new(
            format!("{}^L", self.name),
            self.dim,
        )))
    }
    
    fn fundamental_representation(&self) -> Result<Box<dyn Representation>> {
        Ok(Box::new(MockRepresentation::new(self.dim.min(4))))
    }
    
    fn character_variety(&self, curve: &dyn AlgebraicVariety) -> Result<Box<dyn AlgebraicVariety>> {
        Ok(Box::new(MockVariety::new(
            curve.dimension() + self.rank(),
            curve.genus().unwrap_or(0),
        )))
    }
    
    fn representation_variety(&self, curve: &dyn AlgebraicVariety) -> Result<Box<dyn AlgebraicVariety>> {
        Ok(Box::new(MockVariety::new(
            curve.dimension() + self.rank(),
            curve.genus().unwrap_or(0),
        )))
    }
}

#[derive(Debug, Clone)]
pub struct MockLieAlgebra {
    pub dim: usize,
    pub name: String,
}

impl LieAlgebra for MockLieAlgebra {
    fn dimension(&self) -> usize {
        self.dim
    }
    
    fn root_system(&self) -> String {
        format!("A{}", self.dim - 1)
    }
    
    fn cartan_subalgebra(&self) -> Result<DMatrix<Complex64>> {
        Ok(DMatrix::identity(self.dim.min(3), self.dim.min(3)))
    }
}

#[derive(Debug, Clone)]
pub struct MockVariety {
    pub dim: usize,
    pub genus: i32,
}

impl MockVariety {
    pub fn new(dim: usize, genus: i32) -> Self {
        Self { dim, genus }
    }
}

impl AlgebraicVariety for MockVariety {
    fn dimension(&self) -> usize {
        self.dim
    }
    
    fn is_smooth(&self) -> bool {
        true
    }
    
    fn is_complete(&self) -> bool {
        true
    }
    
    fn genus(&self) -> Result<i32> {
        Ok(self.genus)
    }
    
    fn volume(&self) -> Result<f64> {
        Ok(1.0)
    }
    
    fn intersection(&self, other: &dyn AlgebraicVariety) -> Result<Vec<Vec<f64>>> {
        Ok(vec![vec![0.0; self.dim]; 2]) // Two intersection points
    }
    
    fn mirror_variety(&self) -> Result<Box<dyn AlgebraicVariety>> {
        Ok(Box::new(MockVariety::new(self.dim, self.genus)))
    }
    
    fn moduli_space(&self) -> Result<Box<dyn AlgebraicVariety>> {
        Ok(Box::new(MockVariety::new(3 * self.dim, 0)))
    }
    
    fn holomorphic_curves(&self) -> Result<Vec<HolomorphicCurve>> {
        Ok(vec![
            HolomorphicCurve { area: 1.0, homology_class: vec![1, 0] },
            HolomorphicCurve { area: 2.0, homology_class: vec![0, 1] },
        ])
    }
    
    fn lagrangian_submanifolds(&self) -> Result<Vec<()>> {
        // Temporarily disabled due to circular import
        Ok(Vec::new())
    }
    
    fn codimension(&self) -> Result<usize> {
        Ok(1) // Simplified
    }
    
    fn todd_class(&self) -> Result<Vec<f64>> {
        Ok(vec![1.0, 0.0, 0.0]) // Simplified Todd class
    }
}

#[derive(Debug, Clone)]
pub struct HolomorphicCurve {
    pub area: f64,
    pub homology_class: Vec<i32>,
}

impl HolomorphicCurve {
    pub fn area(&self) -> Result<f64> {
        Ok(self.area)
    }
    
    pub fn homology_class(&self) -> Result<Vec<i32>> {
        Ok(self.homology_class.clone())
    }
}

#[derive(Debug, Clone)]
pub struct MockRepresentation {
    pub dim: usize,
}

impl MockRepresentation {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl Representation for MockRepresentation {
    fn dimension(&self) -> usize {
        self.dim
    }
    
    fn character(&self, _element: &[f64]) -> f64 {
        self.dim as f64
    }
    
    fn weight_space(&self, _weight: &[i32]) -> Result<DVector<Complex64>> {
        Ok(DVector::from_element(self.dim, Complex64::new(1.0, 0.0)))
    }
    
    fn highest_weight(&self) -> Vec<i32> {
        vec![1; self.dim.min(3)]
    }
}

#[derive(Debug, Clone)]
pub struct TrivialLocalSystem {
    rank: usize,
}

impl TrivialLocalSystem {
    pub fn new(rank: usize) -> Self {
        Self { rank }
    }
}

impl crate::sheaf::LocalSystem for TrivialLocalSystem {
    fn rank(&self) -> usize {
        self.rank
    }
    
    fn holonomy(&self, _path: &[f64]) -> Result<Complex64> {
        Ok(Complex64::new(1.0, 0.0))
    }
    
    fn monodromy_representation(&self) -> Result<DMatrix<Complex64>> {
        Ok(DMatrix::identity(self.rank, self.rank))
    }
    
    fn connection_form(&self) -> Result<DMatrix<Complex64>> {
        Ok(DMatrix::zeros(self.rank, self.rank))
    }
}