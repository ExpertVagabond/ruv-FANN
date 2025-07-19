# Sheaf Theory and D-Modules: A Comprehensive Guide

## 🌟 Introduction

Sheaf theory and D-modules form the geometric backbone of the Langlands correspondence. This guide provides a thorough treatment of these fundamental concepts, from basic definitions to their role in the geometric Langlands program.

## 📚 Table of Contents

1. [Sheaf Theory Foundations](#sheaf-theory-foundations)
2. [Constructible Sheaves](#constructible-sheaves)
3. [D-Modules Basics](#d-modules-basics)
4. [Holonomic D-Modules](#holonomic-d-modules)
5. [Riemann-Hilbert Correspondence](#riemann-hilbert-correspondence)
6. [D-Modules on Stacks](#d-modules-on-stacks)
7. [Computational Aspects](#computational-aspects)
8. [Applications to Langlands](#applications-to-langlands)

## 📦 Sheaf Theory Foundations

### Presheaves and Sheaves

#### Definition: Presheaf

A presheaf F on topological space X is:
- **Contravariant functor**: F: Open(X)^op → Sets
- For each open U ⊆ X, a set F(U) (sections over U)
- For V ⊆ U, restriction map ρ_{U,V}: F(U) → F(V)

#### Sheaf Axioms

A sheaf satisfies:

1. **Locality (Uniqueness)**:
   If {U_i} covers U and s, t ∈ F(U) with s|_{U_i} = t|_{U_i} for all i, then s = t.

2. **Gluing (Existence)**:
   If s_i ∈ F(U_i) with s_i|_{U_i ∩ U_j} = s_j|_{U_i ∩ U_j}, then ∃! s ∈ F(U) with s|_{U_i} = s_i.

#### Examples

```rust
// Structure sheaf
pub struct StructureSheaf<X: Variety> {
    variety: X,
}

impl<X: Variety> Sheaf for StructureSheaf<X> {
    type Section = RegularFunction;
    
    fn sections(&self, open: &OpenSet<X>) -> Ring<Self::Section> {
        // Regular functions on open set
        self.variety.regular_functions(open)
    }
    
    fn restrict(&self, f: &Self::Section, from: &OpenSet<X>, to: &OpenSet<X>) 
        -> Self::Section {
        f.restrict(to)
    }
}

// Constant sheaf
pub struct ConstantSheaf<X: Space, A: Set> {
    space: X,
    value: A,
}

// Skyscraper sheaf
pub struct SkyscraperSheaf<X: Space, A: Set> {
    point: Point<X>,
    stalk: A,
}
```

### Sheaf Operations

#### Direct Image

For f: X → Y and sheaf F on X:
```
(f_* F)(V) = F(f^{-1}(V))
```

#### Inverse Image

For f: X → Y and sheaf G on Y:
```
f^* G = sheafification of U ↦ lim_{V ⊃ f(U)} G(V)
```

#### Tensor Product

```
(F ⊗ G)(U) = F(U) ⊗ G(U)
```

#### Internal Hom

```
Hom(F, G)(U) = Hom_{Sh(U)}(F|_U, G|_U)
```

### Sheaf Cohomology

#### Čech Cohomology

For open cover U = {U_i}:

```rust
pub fn cech_complex<F: Sheaf>(sheaf: &F, cover: &OpenCover) -> Complex {
    let mut complex = Complex::new();
    
    // C^0 = ∏_i F(U_i)
    complex.add_term(0, product_sections(sheaf, cover.singles()));
    
    // C^1 = ∏_{i<j} F(U_i ∩ U_j)
    complex.add_term(1, product_sections(sheaf, cover.intersections()));
    
    // Differential: alternating restrictions
    complex.set_differential(cech_differential);
    
    complex
}
```

#### Derived Functor Cohomology

```
H^i(X, F) = R^i Γ(X, F)
```

Where Γ is global sections functor.

#### Key Properties

1. **Long exact sequence**: For 0 → F → G → H → 0:
   ```
   0 → H^0(X,F) → H^0(X,G) → H^0(X,H) → H^1(X,F) → ...
   ```

2. **Vanishing**: On affine varieties, H^i(X, F) = 0 for i > 0 and quasi-coherent F.

## 🏗️ Constructible Sheaves

### Stratifications

#### Definition

A stratification of X is decomposition:
```
X = ⊔_α X_α
```
where each X_α (stratum) is locally closed and satisfies frontier condition.

#### Whitney Stratification

Satisfies Whitney conditions (A) and (B) for smooth control.

### Constructible Sheaves

#### Definition

F is constructible with respect to stratification {X_α} if:
- F|_{X_α} is locally constant of finite rank
- Finite stratification

#### Examples

```rust
pub struct ConstructibleSheaf<X: Space> {
    stratification: Stratification<X>,
    local_systems: HashMap<StratumId, LocalSystem>,
}

impl<X: Space> ConstructibleSheaf<X> {
    // Intersection cohomology sheaf
    pub fn ic_sheaf(variety: &X, local_system: &LocalSystem) -> Self {
        let strat = variety.minimal_stratification();
        let mut local_systems = HashMap::new();
        
        // Place local system on smooth locus
        let smooth_stratum = strat.smooth_locus();
        local_systems.insert(smooth_stratum.id(), local_system.clone());
        
        // Extend by intermediate extension
        Self::intermediate_extension(strat, local_systems)
    }
}
```

### Perverse Sheaves

#### Perverse t-Structure

For stratification by complex dimension:

```
^p D^{≤0} = {F : dim_ℂ supp H^j(F) ≤ -j for all j}
^p D^{≥0} = {F : dim_ℂ supp H^j(D_X F) ≤ -j for all j}
```

#### Perverse Sheaves

```
Perv(X) = ^p D^{≤0} ∩ ^p D^{≥0}
```

Properties:
- Abelian category
- Simple objects = IC sheaves
- Finite length

## 📐 D-Modules Basics

### Definition and Motivation

#### Algebraic Definition

A (left) D-module on smooth variety X is:
- O_X-module M
- Action of derivations Der(O_X)
- Satisfying: ∂(fm) = f∂(m) + (∂f)m (Leibniz rule)

#### Geometric Interpretation

D-modules = Modules over differential operators = Systems of PDEs

### Ring of Differential Operators

#### Construction

```rust
pub struct DifferentialOperators<X: SmoothVariety> {
    variety: X,
}

impl<X: SmoothVariety> Ring for DifferentialOperators<X> {
    fn generate() -> Self {
        // D_X generated by O_X and T_X
        // Relations: [∂, f] = ∂(f) for ∂ ∈ T_X, f ∈ O_X
    }
}

// Filtered structure
pub struct FilteredDiffOps<X: SmoothVariety> {
    // F_0 D = O_X
    // F_1 D = O_X + T_X  
    // F_i D · F_j D ⊆ F_{i+j} D
}
```

#### Symbol Map

```
σ: F_i D / F_{i-1} D → Sym^i(T_X)
```

Leading symbol of differential operator.

### Categories of D-Modules

#### Coherent D-Modules

```rust
pub enum DModuleCategory {
    Coherent,    // Finitely generated
    Holonomic,   // Characteristic variety ≤ dim X
    Regular,     // Regular singularities
    Integrable,  // Integrable connections
}
```

#### Operations

```rust
// Pushforward
pub fn pushforward<X: Variety, Y: Variety>(
    f: &Morphism<X, Y>,
    m: &DModule<X>
) -> DModule<Y> {
    // D-module direct image
    // f_+ M = f_*(M ⊗_{D_X} D_{X→Y})
}

// Pullback
pub fn pullback<X: Variety, Y: Variety>(
    f: &Morphism<X, Y>,
    n: &DModule<Y>
) -> DModule<X> {
    // D-module inverse image
    // f^+ N = D_{Y→X} ⊗_{f^{-1}D_Y} f^{-1}N
}

// Tensor product
pub fn tensor_product<X: Variety>(
    m: &DModule<X>,
    n: &DModule<X>
) -> DModule<X> {
    // M ⊗^D N with diagonal D-action
}
```

### Connections

#### Flat Connection

Equivalent to D-module structure:

```rust
pub struct FlatConnection<X: Variety> {
    bundle: VectorBundle<X>,
    connection: Connection<X>,
}

impl<X: Variety> From<FlatConnection<X>> for DModule<X> {
    fn from(conn: FlatConnection<X>) -> DModule<X> {
        // (E, ∇) ↦ E with ∂(s) = ∇_∂(s)
    }
}
```

#### de Rham Complex

```rust
pub fn de_rham_complex<X: Variety>(conn: &FlatConnection<X>) -> Complex {
    // 0 → E → E ⊗ Ω¹ → E ⊗ Ω² → ...
    Complex::from_connection(conn)
}
```

## 🎯 Holonomic D-Modules

### Characteristic Variety

#### Definition

For coherent D-module M:
```
Ch(M) = Supp(gr M) ⊆ T*X
```
where gr M is associated graded module.

#### Properties

1. Ch(M) is involutive subvariety
2. dim Ch(M) ≥ dim X
3. Equality iff M holonomic

### Holonomic D-Modules

#### Definition

M is holonomic if:
- M coherent
- dim Ch(M) = dim X

#### Key Properties

```rust
impl<X: Variety> Holonomic<X> {
    // Finite length
    pub fn length(&self) -> usize {
        self.composition_series().len()
    }
    
    // Support has finite components
    pub fn singular_support(&self) -> LagrangianCycle {
        // SS(M) ⊆ T*X Lagrangian
    }
    
    // Preserved by proper direct image
    pub fn is_preserved_by_proper_pushforward(&self) -> bool {
        true
    }
}
```

### Regular Holonomic

#### Definition

Regular = holonomic + growth conditions at singular locus

#### Characterizations

1. Extends to good compactification
2. Solutions have moderate growth
3. Corresponds to perverse sheaves

## 🔄 Riemann-Hilbert Correspondence

### Classical Version

#### Statement

Equivalence of categories:
```
RH: D-mod_{reg}(X) ≃ Perv(X^{an})
```

Between:
- Regular holonomic D-modules on X
- Perverse sheaves on X^{an}

#### Construction

```rust
pub fn riemann_hilbert<X: SmoothVariety>(
    d_mod: &RegularHolonomic<X>
) -> PerverseSheaf<X> {
    // Sol(M) = RHom_{D_X}(M, O_X^{an})
    let solution_complex = d_mod.solution_sheaf();
    solution_complex.perverse_cohomology()
}

pub fn inverse_riemann_hilbert<X: SmoothVariety>(
    perv: &PerverseSheaf<X>  
) -> RegularHolonomic<X> {
    // M = RHom(Sol, O_X)[dim X]
    perv.de_rham_complex()
}
```

### Irregular Version

#### Wild Ramification

D-modules with irregular singularities:

```rust
pub struct IrregularDModule<X: Variety> {
    module: DModule<X>,
    formal_type: FormalDecomposition,
    stokes_data: StokesData,
}

// Stokes phenomenon
pub struct StokesData {
    sectors: Vec<AngularSector>,
    transition_matrices: HashMap<(Sector, Sector), Matrix>,
}
```

#### Enhanced RH

```
RH^{enh}: D-mod_{hol}(X) ≃ E(X)
```

Where E(X) = enhanced ind-sheaves (Kashiwara-Schapira).

## 🏛️ D-Modules on Stacks

### Crystals

#### Definition

Crystal on stack X/S = Compatible system of O_T-modules for all S-schemes T.

#### D-Modules as Crystals

```rust
pub struct Crystal<X: Stack> {
    // For each smooth atlas U → X
    atlas_modules: HashMap<Atlas, DModule>,
    // Descent data
    descent: DescentData,
}

impl<X: Stack> Crystal<X> {
    pub fn sections_over<S: Scheme>(&self, f: Morphism<S, X>) -> DModule<S> {
        // Pullback to S
        self.pullback_to_scheme(f)
    }
}
```

### D-Modules on Bun_G

#### Structure

```rust
pub struct DModBunG<G: ReductiveGroup, C: Curve> {
    group: G,
    curve: C,
    level: Option<Level>,
}

impl<G, C> DModBunG<G, C> {
    // Hecke operators
    pub fn hecke_operator(&self, x: Point<C>, lambda: Coweight<G>) 
        -> Functor<DModule, DModule> {
        // T_{x,λ}: DMod(Bun_G) → DMod(Bun_G)
    }
    
    // Hecke eigensheaves
    pub fn is_hecke_eigensheaf(&self, m: &DModule, e: &LocalSystem<G>) -> bool {
        // T_{x,λ}(M) ≃ M ⊗ V_λ(E_x)
    }
}
```

### Twisted D-Modules

#### Twisting by Line Bundle

```rust
pub struct TwistedDModule<X: Variety> {
    module: DModule<X>,
    twist: LineBundle<X>,
}

// Twisted differential operators
pub struct TwistedDiffOps<X: Variety> {
    base: X,
    twist: LineBundle<X>,
}
```

#### Critical Level

For Bun_G, critical twist = K_C^{1/2} ⊗ det^{-1/2}

## 💻 Computational Aspects

### Finite Field Computation

#### D-Modules in Characteristic p

```rust
pub struct FpDModule<X: Variety> {
    // Use divided powers
    // Crystalline cohomology
}

pub fn frobenius_action<X: Variety>(
    m: &FpDModule<X>
) -> Endomorphism {
    // F: M → F^* M
}
```

### Algorithmic D-Modules

#### Gröbner Bases

```rust
pub fn groebner_basis_dmodule(
    generators: Vec<DifferentialOperator>,
    ordering: TermOrdering
) -> Vec<DifferentialOperator> {
    // Buchberger's algorithm for D-modules
    // Weight vector (1,...,1,-1,...,-1)
}

// Integration (restriction)
pub fn integrate_dmodule(
    m: &DModule,
    subvariety: &Subvariety
) -> DModule {
    // Compute H^i(m ⊗ O_Y)
}
```

#### Bernstein-Sato Polynomial

```rust
pub fn bernstein_sato(f: &Polynomial) -> Polynomial {
    // b_f(s) minimal polynomial such that
    // ∃ P(s) ∈ D[s]: P(s)·f^{s+1} = b_f(s)·f^s
    
    // Algorithm:
    // 1. Compute ann(f^s) in D[s]
    // 2. Find minimal b(s)
}
```

### Numerical Methods

#### Connection Matrices

```rust
pub fn connection_matrix(
    bundle: &VectorBundle,
    path: &Path
) -> Matrix {
    // Parallel transport
    // Solve ∇_γ s = 0
    numerical_ode_solver(bundle.connection(), path)
}

// Monodromy
pub fn monodromy_representation(
    bundle: &VectorBundle,
    base_point: &Point
) -> GroupRepresentation {
    let loops = fundamental_group_generators(base_point);
    let matrices: Vec<Matrix> = loops.iter()
        .map(|loop| connection_matrix(bundle, loop))
        .collect();
    
    GroupRepresentation::from_generators(matrices)
}
```

## 🎯 Applications to Langlands

### Hecke Eigensheaves

#### Definition

D-module M on Bun_G is Hecke eigensheaf with eigenvalue E if:

```
T_{x,λ}(M) ≃ M ⊗ V_λ(E_x)
```

for all x ∈ C, λ ∈ Λ^+_Ĝ.

#### Construction

```rust
pub fn construct_hecke_eigensheaf(
    local_system: &LocalSystem<G>,
    curve: &Curve
) -> DModule<BunG> {
    // Geometric Eisenstein series
    // Or geometric theta series
    
    match local_system.rank() {
        1 => multiplicative_eigensheaf(local_system),
        _ => eisenstein_series(local_system, curve)
    }
}
```

### Geometric Langlands Functor

#### Construction

```rust
pub struct GeometricLanglandsFunctor<G: ReductiveGroup, C: Curve> {
    group: G,
    curve: C,
}

impl<G, C> Functor for GeometricLanglandsFunctor<G, C> {
    type Source = DModCategory<BunG>;
    type Target = IndCohCategory<LocG>;
    
    fn apply(&self, m: &DModule<BunG>) -> IndCoh<LocG> {
        // 1. Spectral decomposition by Hecke eigenvalues
        // 2. Fourier-Mukai transform
        // 3. Singular support condition
    }
}
```

### Opers and Miura Opers

#### Opers

```rust
pub struct Oper<G: ReductiveGroup, C: Curve> {
    bundle: GBundle<G, C>,
    connection: Connection,
    reduction: BorelReduction,
    generic_position: GenericityCondition,
}

// Space of opers = affine space
pub fn oper_space<G: ReductiveGroup, C: Curve>(
    group: &G,
    curve: &C
) -> AffineSpace {
    // Dimension = (rank G) × (genus(C) - 1)
    let dim = group.rank() * (curve.genus() - 1);
    AffineSpace::new(dim)
}
```

### Quantum Langlands

#### Difference Equations

```rust
pub struct QDModule<X: Variety> {
    module: Module,
    q_connection: QConnection,
}

// q-difference operator
pub fn q_difference_operator(q: Complex) -> impl Operator {
    move |f: Function| -> Function {
        |x| f(q * x)
    }
}
```

## 📚 References

### Sheaf Theory
1. Kashiwara-Schapira "Sheaves on Manifolds"
2. Iversen "Cohomology of Sheaves"
3. Dimca "Sheaves in Topology"

### D-Modules
1. Borel et al. "Algebraic D-modules"
2. Björk "Analytic D-modules"
3. Coutinho "A Primer of Algebraic D-modules"

### Geometric Langlands
1. Beilinson-Drinfeld "Chiral Algebras"
2. Frenkel-Ben-Zvi "Vertex Algebras and Algebraic Curves"
3. Gaitsgory-Rozenblyum "A Study in Derived Algebraic Geometry"

---

*This guide provides comprehensive coverage of sheaf theory and D-modules as they appear in the geometric Langlands program, with emphasis on computational realizability.*