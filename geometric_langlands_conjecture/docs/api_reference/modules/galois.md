# Galois Module API Reference

## 📚 Overview

The `galois` module implements Galois representations, local systems, and l-adic sheaves - the "Galois side" of the Langlands correspondence.

## 🎯 Module Structure

```rust
pub mod galois {
    pub mod representations;  // Galois representations
    pub mod local_systems;   // Local systems on curves
    pub mod l_adic;         // l-adic sheaves and étale cohomology
    pub mod fundamental;    // Fundamental groups
    pub mod ramification;   // Ramification theory
    pub mod weil_deligne;   // Weil-Deligne representations
}
```

## 🔷 Core Types

### GaloisRepresentation

Represents a continuous homomorphism from a Galois group.

```rust
pub struct GaloisRepresentation<F: Field, G: Group> {
    pub base_field: F,
    pub target_group: G,
    pub dimension: usize,
    pub l_adic_prime: Prime,
    representation: Box<dyn Fn(&GaloisElement) -> G::Element>,
}

impl<F: Field, G: Group> GaloisRepresentation<F, G> {
    /// Create a new Galois representation
    pub fn new(
        base_field: F,
        target_group: G,
        l: Prime
    ) -> Result<Self, Error>;
    
    /// Evaluate representation at Galois element
    pub fn evaluate(&self, g: &GaloisElement) -> G::Element;
    
    /// Character of the representation
    pub fn character(&self) -> Character<F, G>;
    
    /// Check if representation is unramified at prime
    pub fn is_unramified_at(&self, p: &Prime) -> bool;
    
    /// Frobenius element at unramified prime
    pub fn frobenius_at(&self, p: &Prime) -> Result<G::Element, Error>;
    
    /// L-factor at prime p
    pub fn l_factor(&self, p: &Prime, t: Polynomial) -> Polynomial;
    
    /// Conductor of the representation
    pub fn conductor(&self) -> Ideal<F>;
    
    /// Restriction to decomposition group
    pub fn restrict_to_decomposition(&self, p: &Prime) 
        -> LocalGaloisRep<F, G>;
}
```

#### Example: Tate Module Representation

```rust
use geometric_langlands::galois::{GaloisRepresentation, TateModule};

// Galois representation from elliptic curve
let curve = EllipticCurve::from_weierstrass([0, 0, 0, -1, 0]); // y² = x³ - x
let field = NumberField::rationals();
let l = Prime::new(5);

// l-adic Tate module gives 2-dimensional representation
let tate_rep = GaloisRepresentation::from_tate_module(
    &curve,
    field,
    l
)?;

// Check Frobenius eigenvalues at good primes
for p in primes_up_to(100) {
    if p != l && curve.has_good_reduction_at(p) {
        let frob = tate_rep.frobenius_at(&p)?;
        let char_poly = frob.characteristic_polynomial();
        
        // Verify Hasse bound: |a_p| ≤ 2√p
        let a_p = -char_poly.coefficient(1);
        assert!(a_p.abs() <= 2.0 * (p as f64).sqrt());
    }
}
```

### LocalSystem

Local systems on algebraic curves - geometric incarnation of Galois representations.

```rust
pub struct LocalSystem<C: Curve, G: Group> {
    pub curve: C,
    pub rank: usize,
    pub group: G,
    pub monodromy: MonodromyData<C, G>,
    pub singularities: Vec<(Point<C>, LocalMonodromy<G>)>,
}

impl<C: Curve, G: Group> LocalSystem<C, G> {
    /// Create local system from monodromy representation
    pub fn from_monodromy(
        curve: C,
        base_point: Point<C>,
        monodromy: FundamentalGroupRep<C, G>
    ) -> Self;
    
    /// Create local system with prescribed ramification
    pub fn with_ramification(
        curve: C,
        group: G,
        ramification_data: Vec<(Point<C>, ConjugacyClass<G>)>
    ) -> Result<Self, Error>;
    
    /// Monodromy around point
    pub fn monodromy_at(&self, p: &Point<C>) -> G::Element;
    
    /// Check if unramified at point
    pub fn is_unramified_at(&self, p: &Point<C>) -> bool;
    
    /// Euler characteristic
    pub fn euler_characteristic(&self) -> isize;
    
    /// Cohomology groups H^i(C, L)
    pub fn cohomology(&self, degree: usize) -> CohomologyGroup<G>;
    
    /// Tensor product with another local system
    pub fn tensor(&self, other: &Self) -> LocalSystem<C, G>;
    
    /// Dual local system
    pub fn dual(&self) -> LocalSystem<C, G>;
}
```

#### Example: Kummer Local System

```rust
// Local system on P¹ \ {0, 1, ∞} from Kummer cover
let curve = ProjectiveLine::new(ComplexField);
let ramification_points = vec![
    Point::zero(),
    Point::one(), 
    Point::infinity()
];

// Create rank-2 local system with prescribed monodromy
let monodromy_data = vec![
    (ramification_points[0], Matrix2::from_eigenvalues(omega, omega.conj())),
    (ramification_points[1], Matrix2::from_eigenvalues(omega.pow(2), omega.pow(-2))),
    (ramification_points[2], Matrix2::identity()),
];

let local_system = LocalSystem::with_ramification(
    curve,
    GL2::over(ComplexField),
    monodromy_data
)?;

// Compute cohomology
let h0 = local_system.cohomology(0);
let h1 = local_system.cohomology(1);
println!("H⁰ dimension: {}", h0.dimension());
println!("H¹ dimension: {}", h1.dimension());

// Verify Euler characteristic
let chi = h0.dimension() as isize - h1.dimension() as isize;
assert_eq!(chi, local_system.euler_characteristic());
```

## 🔢 L-adic Sheaves

### LAdicSheaf

Implementation of l-adic étale sheaves.

```rust
pub struct LAdicSheaf<X: Scheme> {
    pub base_scheme: X,
    pub l: Prime,
    pub rank: usize,
    pub constructible: bool,
    stalks: StalkData<X>,
}

impl<X: Scheme> LAdicSheaf<X> {
    /// Constant sheaf
    pub fn constant(scheme: X, l: Prime, rank: usize) -> Self;
    
    /// Sheaf from local system (for curves)
    pub fn from_local_system<C: Curve>(
        local_system: &LocalSystem<C, GLn>
    ) -> Self where X: From<C>;
    
    /// Stalk at geometric point
    pub fn stalk_at(&self, x: &GeometricPoint<X>) -> LAdicModule;
    
    /// Étale cohomology H^i_ét(X, F)
    pub fn etale_cohomology(&self, degree: usize) -> LAdicModule;
    
    /// Frobenius action on cohomology
    pub fn frobenius_on_cohomology(&self, degree: usize) 
        -> Endomorphism<LAdicModule>;
    
    /// Direct image f_* F
    pub fn direct_image<Y: Scheme>(&self, f: &Morphism<X, Y>) 
        -> LAdicSheaf<Y>;
    
    /// Inverse image f^* F
    pub fn inverse_image<Y: Scheme>(&self, f: &Morphism<Y, X>) 
        -> LAdicSheaf<Y>;
    
    /// Perverse cohomology
    pub fn perverse_cohomology(&self) -> PerverseSheaf<X>;
}
```

### WeilDeligneRepresentation

Representations of the Weil-Deligne group.

```rust
pub struct WeilDeligneRep<F: LocalField> {
    pub field: F,
    pub dimension: usize,
    pub weil_rep: WeilGroupRep<F>,
    pub monodromy_operator: NilpotentOperator,
}

impl<F: LocalField> WeilDeligneRep<F> {
    /// Create from Galois representation
    pub fn from_galois_rep(
        rep: &LocalGaloisRep<F, GLn>
    ) -> Self;
    
    /// Frobenius semisimplification
    pub fn frobenius_semisimple(&self) -> WeilGroupRep<F>;
    
    /// Monodromy filtration
    pub fn monodromy_filtration(&self) -> Filtration;
    
    /// L-factor
    pub fn l_factor(&self, s: Complex) -> Complex;
    
    /// ε-factor
    pub fn epsilon_factor(&self, s: Complex, psi: &Character<F>) -> Complex;
    
    /// Check if representation is pure
    pub fn is_pure(&self) -> bool;
    
    /// Weight of pure representation
    pub fn weight(&self) -> Option<isize>;
}
```

## 🌀 Fundamental Groups

### FundamentalGroup

Étale and topological fundamental groups.

```rust
pub struct FundamentalGroup<X: Scheme> {
    pub base_scheme: X,
    pub base_point: GeometricPoint<X>,
    pub profinite: bool,
}

impl<X: Scheme> FundamentalGroup<X> {
    /// Étale fundamental group
    pub fn etale(scheme: X, base_point: GeometricPoint<X>) -> Self;
    
    /// Topological fundamental group (for complex varieties)
    pub fn topological(variety: X) -> Self where X: ComplexVariety;
    
    /// Finite quotient π₁(X) → G
    pub fn finite_quotient<G: FiniteGroup>(&self) -> GroupHomomorphism<Self, G>;
    
    /// Abelianization π₁(X)^ab
    pub fn abelianization(&self) -> AbelianGroup;
    
    /// Pro-l completion
    pub fn pro_l_completion(&self, l: Prime) -> ProLGroup;
    
    /// Representation in GL_n(Q_l)
    pub fn l_adic_representation(&self, n: usize, l: Prime) 
        -> GaloisRepresentation<X::Field, GLn<QlField>>;
}
```

#### Example: Fundamental Group of Punctured Line

```rust
// π₁(P¹ \ {0, 1, ∞})
let line = ProjectiveLine::new(ComplexField);
let punctures = vec![Point::zero(), Point::one(), Point::infinity()];
let punctured_line = line.remove_points(&punctures);

let pi1 = FundamentalGroup::topological(punctured_line);

// Generators and relations
let gens = pi1.generators(); // 2 generators
let relations = pi1.relations(); // 1 relation: [a,b] = 1

// Map to SL₂
let rep = pi1.representation_to_sl2(ComplexField);
let a = rep.evaluate(&gens[0]);
let b = rep.evaluate(&gens[1]);

// Verify relation
assert_eq!(a.commutator(&b), Matrix2::identity());
```

## 🔀 Ramification Theory

### RamificationData

Describes ramification of Galois representations.

```rust
pub struct RamificationData<F: Field> {
    pub prime: Prime,
    pub inertia_type: InertiaType,
    pub swan_conductor: usize,
    pub breaks: Vec<Rational>,
}

impl<F: Field> RamificationData<F> {
    /// Tame ramification
    pub fn tame(prime: Prime, character: Character<F>) -> Self;
    
    /// Wild ramification with given Swan conductor
    pub fn wild(prime: Prime, swan: usize) -> Self;
    
    /// Compute Artin conductor
    pub fn artin_conductor(&self) -> usize;
    
    /// Higher ramification groups
    pub fn higher_ramification_groups(&self) -> Vec<RamificationGroup>;
    
    /// Break decomposition
    pub fn break_decomposition(&self) -> BreakDecomposition;
}
```

### IrregularConnections

Local systems with irregular singularities.

```rust
pub struct IrregularLocalSystem<C: Curve> {
    pub curve: C,
    pub regular_part: LocalSystem<C, GLn>,
    pub irregular_data: Vec<(Point<C>, IrregularPart)>,
}

pub struct IrregularPart {
    pub formal_type: FormalConnection,
    pub stokes_data: StokesMatrices,
    pub exponential_factors: Vec<FormalSeries>,
}

impl<C: Curve> IrregularLocalSystem<C> {
    /// Create with prescribed irregular singularities
    pub fn with_irregular_singularities(
        curve: C,
        singularities: Vec<(Point<C>, FormalType)>
    ) -> Result<Self, Error>;
    
    /// Formal monodromy at irregular point
    pub fn formal_monodromy(&self, p: &Point<C>) -> FormalMonodromy;
    
    /// Stokes matrices
    pub fn stokes_matrices(&self, p: &Point<C>) -> StokesData;
    
    /// de Rham cohomology with irregular coefficients
    pub fn de_rham_cohomology(&self) -> IrregularHodgeStructure;
}
```

## ⚡ Computational Methods

### Character Tables

```rust
/// Compute character tables for Galois groups
pub fn character_table<F: Field>(
    field: &F,
    degree: usize
) -> CharacterTable {
    let gal_group = field.galois_group();
    gal_group.character_table()
}

/// Find Galois representation with given character
pub fn representation_from_character<F: Field, G: Group>(
    field: &F,
    character: &Character<F, G>
) -> Option<GaloisRepresentation<F, G>> {
    // Use character theory to construct representation
}
```

### Frobenius Elements

```rust
/// Compute Frobenius elements efficiently
pub struct FrobeniusCache<F: NumberField> {
    field: F,
    cache: HashMap<Prime, GaloisElement>,
}

impl<F: NumberField> FrobeniusCache<F> {
    pub fn compute_frobenius(&mut self, p: &Prime) -> &GaloisElement {
        self.cache.entry(*p).or_insert_with(|| {
            self.field.frobenius_element(p)
        })
    }
    
    pub fn batch_compute(&mut self, primes: &[Prime]) -> Vec<&GaloisElement> {
        primes.par_iter()
            .map(|p| self.compute_frobenius(p))
            .collect()
    }
}
```

## 🧪 Examples and Tests

### Example: Galois Representation of CM Elliptic Curve

```rust
#[test]
fn test_cm_elliptic_curve() {
    // Elliptic curve with complex multiplication
    let curve = EllipticCurve::from_j_invariant(Complex::from(1728)); // j = 1728
    let field = NumberField::from_polynomial(x.pow(2) + 1); // Q(i)
    
    // Galois representation is reducible
    let l = Prime::new(7);
    let rho = GaloisRepresentation::from_tate_module(&curve, field, l)?;
    
    // Verify CM structure
    assert!(rho.is_reducible());
    
    // Decompose into characters
    let (chi1, chi2) = rho.decompose_into_characters()?;
    assert_eq!(chi1.order(), 4); // Order 4 character
}
```

### Example: Local System on Modular Curve

```rust
#[test]
fn test_modular_curve_local_system() {
    // Modular curve X(11)
    let curve = ModularCurve::new(11);
    
    // Universal elliptic curve gives local system
    let universal_curve = curve.universal_elliptic_curve();
    let local_system = LocalSystem::from_family(&universal_curve, Prime::new(3))?;
    
    // Compute cohomology
    let h1 = local_system.cohomology(1);
    
    // Should match dimension of cusp forms
    let cusp_forms = CuspForms::new(11, 2);
    assert_eq!(h1.dimension(), cusp_forms.dimension());
}
```

## 📊 Performance Optimization

### Parallel Frobenius Computation

```rust
pub fn parallel_l_function<F: NumberField>(
    rep: &GaloisRepresentation<F, GLn>,
    prime_bound: usize
) -> LFunction {
    let primes: Vec<_> = primes_up_to(prime_bound).collect();
    
    let local_factors: Vec<_> = primes
        .par_iter()
        .map(|p| {
            if rep.is_unramified_at(p) {
                let frob = rep.frobenius_at(p).unwrap();
                let char_poly = frob.characteristic_polynomial();
                (*p, char_poly)
            } else {
                (*p, rep.l_factor(p, Polynomial::variable()))
            }
        })
        .collect();
    
    LFunction::from_euler_factors(local_factors)
}
```

### Caching Galois Computations

```rust
thread_local! {
    static GALOIS_CACHE: RefCell<LruCache<(Field, Prime), GaloisGroup>> = 
        RefCell::new(LruCache::new(1000));
}

pub fn cached_galois_group(field: &Field, p: &Prime) -> GaloisGroup {
    GALOIS_CACHE.with(|cache| {
        cache.borrow_mut()
            .get(&(field.clone(), *p))
            .cloned()
            .unwrap_or_else(|| {
                let group = field.galois_group_at(p);
                cache.borrow_mut().put((field.clone(), *p), group.clone());
                group
            })
    })
}
```

## 📚 References

- Serre, J.P. "Abelian l-adic Representations and Elliptic Curves"
- Katz, N. "Gauss Sums, Kloosterman Sums, and Monodromy Groups"
- Deligne, P. "La Conjecture de Weil II"
- Laumon, G. "Transformation de Fourier, constantes d'équations fonctionnelles"

---

*This API reference covers the Galois module's functionality for working with Galois representations, local systems, and l-adic sheaves in the geometric Langlands program.*