# Category Theory for Geometric Langlands

## 🎯 Introduction

Category theory provides the natural language for the geometric Langlands correspondence. This guide presents the categorical foundations necessary to understand the correspondence as an equivalence of derived categories.

## 📚 Table of Contents

1. [Basic Categories](#basic-categories)
2. [Derived Categories](#derived-categories)
3. [Sheaves and Stacks](#sheaves-and-stacks)
4. [D-Modules](#d-modules)
5. [Perverse Sheaves](#perverse-sheaves)
6. [Six Operations](#six-operations)
7. [∞-Categories](#infinity-categories)
8. [Categorical Langlands](#categorical-langlands)

## 🔷 Basic Categories

### Categories and Functors

#### Definition: Category

A category **C** consists of:
- **Objects**: Ob(C)
- **Morphisms**: For each pair A, B ∈ Ob(C), a set Hom(A, B)
- **Composition**: ∘: Hom(B, C) × Hom(A, B) → Hom(A, C)
- **Identity**: id_A ∈ Hom(A, A) for each A

#### Axioms

1. **Associativity**: (h ∘ g) ∘ f = h ∘ (g ∘ f)
2. **Identity**: id_B ∘ f = f = f ∘ id_A for f: A → B

#### Examples in Langlands

```rust
// Category of vector bundles
pub struct VectBundles<X: Variety> {
    base: X,
}

impl<X: Variety> Category for VectBundles<X> {
    type Object = VectorBundle<X>;
    type Morphism = BundleMap;
    
    fn compose(f: &Self::Morphism, g: &Self::Morphism) -> Self::Morphism {
        // Composition of bundle maps
    }
}

// Category of D-modules
pub struct DMod<X: Variety> {
    base: X,
}
```

### Functors

#### Definition

A functor F: C → D consists of:
- **Object map**: F: Ob(C) → Ob(D)
- **Morphism map**: F: Hom_C(A, B) → Hom_D(F(A), F(B))

Preserving:
- **Composition**: F(g ∘ f) = F(g) ∘ F(f)
- **Identity**: F(id_A) = id_{F(A)}

#### Key Functors in Langlands

1. **Riemann-Hilbert**:
```
RH: DMod(X) → Perv(X)
D-modules → Perverse sheaves
```

2. **Hecke functors**:
```
T_{x,λ}: DMod(Bun_G) → DMod(Bun_G)
```

3. **Geometric Langlands functor**:
```
GL: DMod(Bun_G) → QCoh(Loc_Ĝ)
```

### Natural Transformations

#### Definition

A natural transformation α: F ⇒ G between functors F, G: C → D:
- For each A ∈ Ob(C), a morphism α_A: F(A) → G(A)
- **Naturality**: For f: A → B, we have G(f) ∘ α_A = α_B ∘ F(f)

#### Diagram
```
F(A) ---α_A---> G(A)
 |               |
F(f)|           |G(f)
 ↓               ↓
F(B) ---α_B---> G(B)
```

## 🔺 Derived Categories

### Complexes

#### Definition

A complex C• in an abelian category A:
```
... → C^{n-1} → C^n → C^{n+1} → ...
```
with d^n ∘ d^{n-1} = 0.

#### Cohomology

```
H^n(C•) = ker(d^n) / im(d^{n-1})
```

### Derived Category Construction

#### Homotopy Category

K(A) = Category of complexes up to homotopy:
- **Objects**: Complexes in A
- **Morphisms**: Chain maps modulo homotopy

#### Localization

D(A) = K(A)[quasi-isomorphisms^{-1}]

Where quasi-isomorphism = induces isomorphism on cohomology.

### Triangulated Structure

#### Distinguished Triangles

A triangle A → B → C → A[1] is distinguished if it's isomorphic to:
```
C• → Cone(f) → A[1] → C•[1]
```
for some f: C• → D•.

#### Axioms (TR1-TR4)

1. **TR1**: Every morphism embeds in distinguished triangle
2. **TR2**: Triangles isomorphic to distinguished are distinguished
3. **TR3**: Rotation axiom
4. **TR4**: Octahedral axiom

### Derived Functors

#### Construction

For F: A → B:
1. Choose resolution P• → A
2. Apply F to get F(P•)
3. RF(A) := F(P•) in D(B)

#### Examples

```rust
// Right derived functor of global sections
pub fn r_gamma<X: Scheme>(sheaf: &Sheaf<X>) -> Complex {
    let resolution = cech_resolution(sheaf);
    resolution.global_sections()
}

// Left derived tensor product
pub fn l_tensor(a: &Complex, b: &Complex) -> Complex {
    let flat_res_a = flat_resolution(a);
    tensor_complex(&flat_res_a, b)
}
```

## 🌐 Sheaves and Stacks

### Sheaves

#### Definition

A sheaf F on space X assigns:
- To each open U ⊆ X, an object F(U)
- To each inclusion V ⊆ U, a restriction F(U) → F(V)

Satisfying:
1. **Locality**: If s, t ∈ F(U) agree on covering, then s = t
2. **Gluing**: Compatible sections glue uniquely

#### Constructible Sheaves

Finite stratification X = ⊔ X_α such that F|_{X_α} is locally constant.

```rust
pub struct ConstructibleSheaf<X: Variety> {
    stratification: Stratification<X>,
    local_systems: Vec<LocalSystem>,
}
```

### Stacks

#### 2-Categories

Objects, 1-morphisms, and 2-morphisms:
```
A ==f==> B
‖   α   ‖
A ==g==> B
```

#### Definition: Stack

A stack X over category C:
1. **Fibered category** X → C
2. **Descent**: Sheaf condition for morphisms and objects

#### Moduli Stacks

```rust
// Stack of G-bundles
pub struct BunG<G: ReductiveGroup, C: Curve> {
    group: G,
    curve: C,
}

impl<G, C> Stack for BunG<G, C> {
    fn objects_over<S: Scheme>(&self, base: &S) -> Category {
        // G-bundles on C × S → S
    }
}
```

## 📐 D-Modules

### Definition

A D-module on smooth variety X is:
- Quasi-coherent O_X-module M
- Action of tangent sheaf T_X
- Satisfying Leibniz rule

#### Flat Connection

Equivalent data:
```
∇: M → M ⊗ Ω^1_X
```
with ∇² = 0 (integrability).

### Categories of D-Modules

#### Coherent D-Modules

D-mod_coh(X) = finitely generated D-modules

#### Holonomic D-Modules

D-mod_hol(X) ⊂ D-mod_coh(X)
- Characteristic variety has dimension ≤ dim X
- Regular singularities

### D-Module Operations

```rust
// Direct image
pub fn direct_image<X: Variety, Y: Variety>(
    f: &Morphism<X, Y>,
    m: &DModule<X>
) -> DModule<Y> {
    // f_+ M = f_* (M ⊗ ω_{X/Y})
}

// Inverse image  
pub fn inverse_image<X: Variety, Y: Variety>(
    f: &Morphism<X, Y>,
    n: &DModule<Y>
) -> DModule<X> {
    // f^+ N = O_X ⊗_{f^{-1}O_Y} f^{-1}N
}
```

## 🔶 Perverse Sheaves

### t-Structures

#### Definition

A t-structure on triangulated category D:
- Full subcategories D^{≤0}, D^{≥0}
- D^{≤0}[1] ⊆ D^{≤0}
- D^{≥0}[-1] ⊆ D^{≥0}
- Hom(D^{≤0}, D^{≥0}[-1]) = 0
- Every object has canonical triangle

#### Perverse t-Structure

```
D^{≤0}_p = {F : dim supp H^{-i}(F) ≤ i}
D^{≥0}_p = {F : dim supp H^{-i}(D(F)) ≤ i}
```

### Perverse Sheaves

#### Definition

Perv(X) = Heart of perverse t-structure = D^{≤0}_p ∩ D^{≥0}_p

#### Properties

1. **Abelian category**
2. **Artinian and Noetherian**
3. **Finite length objects**

#### Key Examples

```rust
// Intersection cohomology
pub fn ic_sheaf<X: Variety>(stratification: &Stratification<X>) -> PerverseSheaf<X> {
    let open_stratum = stratification.open_stratum();
    let local_system = constant_sheaf(&open_stratum);
    intermediate_extension(local_system)
}

// Constant perverse sheaf
pub fn constant_perverse<X: Variety>() -> PerverseSheaf<X> {
    shifted_constant_sheaf(dim(X))
}
```

## 🔧 Six Operations

### Grothendieck's Six Functors

For morphism f: X → Y:

1. **f^***: Inverse image (exact)
2. **f_***: Direct image (left exact)
3. **f_!**: Direct image with compact support
4. **f^!**: Exceptional inverse image
5. **⊗**: Tensor product
6. **Hom**: Internal hom

### Adjunctions

```
f^* ⊣ f_*    (f^* left adjoint to f_*)
f_! ⊣ f^!    (f_! left adjoint to f^!)
```

### Base Change

For cartesian square:
```
W ---g'---> X
|           |
f'|         |f
↓           ↓
Y ---g----> Z
```

Base change isomorphisms:
- g^* f_* ≃ f'_* g'^*
- g^! f_! ≃ f'_! g'^!

### Verdier Duality

Dualizing functor D:
```
D: D^b_c(X)^op → D^b_c(X)
```

Properties:
- D² ≃ id
- D(f_*) ≃ f_! D
- D(f^*) ≃ f^! D

## ∞ ∞-Categories

### Higher Categories

#### Definition

An (∞,1)-category (∞-category) has:
- Objects
- Morphisms (1-morphisms)
- 2-morphisms (homotopies)
- ...
- n-morphisms (higher homotopies)

With all k-morphisms invertible for k ≥ 2.

### Models

#### Quasi-Categories

Simplicial sets satisfying inner horn filling:
```
Λ^n_i → Δ^n has lift for 0 < i < n
```

#### Complete Segal Spaces

Simplicial spaces X_• with:
- X_0 = space of objects
- X_1 = space of morphisms
- Segal condition for composition

### ∞-Categorical Langlands

#### Statement

Equivalence of (∞,1)-categories:
```
IndCoh(Bun_G) ≃ QCoh(Loc_Ĝ)
```

Where:
- IndCoh = Ind-coherent sheaves
- Singular support condition

## 🎯 Categorical Langlands

### Spectral Side

#### IndCoh Category

```rust
pub struct IndCoh<X: DerivedStack> {
    base: X,
    t_structure: TStructure,
}

impl<X: DerivedStack> InfinityCategory for IndCoh<X> {
    // ∞-categorical structure
}
```

Properties:
- Self-dual
- Compactly generated
- Singular support

### Automorphic Side

#### D-Modules on Stacks

For Artin stack X:
```
DMod(X) = lim D-mod(S)
```
over smooth presentations S → X.

### The Functor

#### Construction

```
L: DMod(Bun_G) → IndCoh(Loc_Ĝ)
```

Via:
1. Hecke eigensheaves
2. Spectral decomposition
3. Singular support

#### Properties

1. **t-Exact**: Preserves t-structures
2. **Hecke-equivariant**: Intertwines Hecke actions
3. **Miraculous**: Many unexpected properties

### Ramification

#### Parabolic Structures

At points x_i ∈ C:
- Parabolic G-bundles
- Parabolic Ĝ-local systems

#### Wild Ramification

Irregular singularities:
- Stokes phenomena
- Exponential D-modules
- Irregular Riemann-Hilbert

## 💻 Computational Realization

### Finite Models

```rust
// Finite field approximation
pub struct FiniteLanglands<F: FiniteField> {
    field: F,
    curve: Curve<F>,
}

impl<F: FiniteField> CategoricalCorrespondence for FiniteLanglands<F> {
    type Automorphic = FiniteDMod<F>;
    type Spectral = FiniteIndCoh<F>;
    
    fn correspondence(&self) -> Equivalence<Self::Automorphic, Self::Spectral> {
        // Explicit finite computation
    }
}
```

### Derived Computation

```rust
// Computing in derived categories
pub fn derived_hom<C: Category>(
    a: &Complex<C>,
    b: &Complex<C>
) -> Complex<Ab> {
    let proj_res = projective_resolution(a);
    hom_complex(&proj_res, b)
}

// Spectral sequences
pub fn spectral_sequence<F: Functor>(
    functor: &F,
    complex: &Complex
) -> SpectralSequence {
    // E_2 page from composition of derived functors
}
```

### Verification

```rust
// Verify categorical equivalence
pub fn verify_equivalence<F: Functor, G: Functor>(
    f: &F,
    g: &G
) -> bool {
    // Check F ∘ G ≃ id and G ∘ F ≃ id
    let fg_id = natural_iso(compose(f, g), identity());
    let gf_id = natural_iso(compose(g, f), identity());
    
    fg_id.is_some() && gf_id.is_some()
}
```

## 📚 References

### Category Theory
1. Mac Lane, S. "Categories for the Working Mathematician"
2. Lurie, J. "Higher Topos Theory"
3. Riehl, E. "Category Theory in Context"

### Derived Categories
1. Gelfand-Manin "Methods of Homological Algebra"
2. Kashiwara-Schapira "Categories and Sheaves"
3. Weibel "Introduction to Homological Algebra"

### Geometric Langlands
1. Beilinson-Drinfeld "Quantization of Hitchin's System"
2. Gaitsgory "Outline of the proof of the geometric Langlands conjecture"
3. Ben-Zvi, Nadler "The Character Theory of a Complex Group"

---

*This guide provides the categorical foundations essential for understanding the geometric Langlands correspondence as a derived equivalence of categories.*