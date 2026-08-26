//! Shapes: the domain/codomain skeleton of a category, enumerated up to isomorphism.
//!
//! Every finite category determines a directed multigraph on its objects, with one
//! edge per nonidentity morphism.  We call the isomorphism class of that multigraph
//! the category's *shape*.  Two categories that share a fixed shape are isomorphic
//! exactly when some automorphism of the shape carries one to the other, so
//! enumerating shapes first replaces the relabelling group `S_k × S_(n-k)` by the
//! much smaller `Aut(shape)` — usually trivial.
//!
//! Morphisms are numbered with the identities first: `0..objects` are the
//! identities, and the nonidentity morphisms follow, sorted by `(domain, codomain)`.

use crate::{Entry, Result};

/// The number of shape automorphisms we are willing to hold in memory at once.
/// The group is `S_(n-k)` in the worst case (a one-object shape), so this caps
/// single-object cells at nine morphisms.  Larger cells need a canonical-labelling
/// search instead of an explicit group.
const AUTOMORPHISM_LIMIT: usize = 400_000;

/// A fixed assignment of domains and codomains to a numbered set of morphisms.
#[derive(Clone, Debug)]
pub struct Shape {
    pub morphisms: usize,
    pub objects: usize,
    pub domain: Vec<usize>,
    pub codomain: Vec<usize>,
    /// Every morphism permutation that preserves domains and codomains.  Always
    /// contains the identity.
    pub automorphisms: Vec<Automorphism>,
}

/// A relabelling of the morphisms that leaves the shape alone.
///
/// Both directions are kept because relabelling a table needs both: a cell moves
/// to the position its endpoints move to, but the *composite stored in it* is a
/// morphism, so it moves the other way.  Using `map` for both is only correct
/// when the permutation happens to be an involution.
#[derive(Clone, Debug)]
pub struct Automorphism {
    /// Morphism to its image; use this to index cells.
    pub map: Vec<usize>,
    /// The inverse; use this to relabel the value found in a cell.
    pub inverse: Vec<usize>,
}

impl Shape {
    /// Morphisms that can be the value of `left ∘ right`: those with the domain of
    /// `right` and the codomain of `left`.
    pub fn candidates(&self, left: usize, right: usize) -> Vec<Entry> {
        (0..self.morphisms)
            .filter(|&v| self.domain[v] == self.domain[right] && self.codomain[v] == self.codomain[left])
            .map(|v| Entry::Mor(v as u8))
            .collect()
    }

    pub fn composable(&self, left: usize, right: usize) -> bool {
        self.domain[left] == self.codomain[right]
    }
}

/// Enumerate every shape with `morphisms` morphisms and `objects` objects, one
/// representative per isomorphism class.  When `connected_only` is set, shapes
/// whose underlying graph is disconnected are skipped: a disconnected category is
/// reconstructed from its components instead of being searched for directly.
pub fn shapes(morphisms: usize, objects: usize, connected_only: bool) -> Result<Vec<Shape>> {
    if objects > morphisms {
        return Err(format!("{objects} objects need at least as many morphisms"));
    }
    if morphisms == 0 {
        return Ok(vec![Shape {
            morphisms: 0,
            objects: 0,
            domain: Vec::new(),
            codomain: Vec::new(),
            automorphisms: vec![Automorphism {
                map: Vec::new(),
                inverse: Vec::new(),
            }],
        }]);
    }
    if objects == 0 {
        return Ok(Vec::new());
    }

    let edges = morphisms - objects;
    // A connected shape on k objects needs at least k-1 edges to span them, so
    // below that there is nothing to enumerate.  Saying so here also keeps us
    // from building relabelling tables for object counts we never reach.
    if connected_only && objects > 1 && edges + 1 < objects {
        return Ok(Vec::new());
    }
    let relabellings = Relabellings::new(objects.max(edges))?;
    let mut found = Vec::new();
    let mut matrix = vec![0usize; objects * objects];
    collect(
        &mut matrix,
        0,
        edges,
        objects,
        connected_only,
        &relabellings,
        &mut found,
    );

    found
        .into_iter()
        .map(|matrix| build(&matrix, morphisms, objects, &relabellings))
        .collect()
}

/// Fill the multiplicity matrix cell by cell, keeping canonical representatives.
fn collect(
    matrix: &mut [usize],
    cell: usize,
    remaining: usize,
    objects: usize,
    connected_only: bool,
    relabellings: &Relabellings,
    found: &mut Vec<Vec<usize>>,
) {
    if cell == objects * objects {
        if remaining != 0 {
            return;
        }
        if connected_only && !connected(matrix, objects) {
            return;
        }
        if is_canonical(matrix, objects, relabellings) {
            found.push(matrix.to_vec());
        }
        return;
    }
    for count in 0..=remaining {
        matrix[cell] = count;
        collect(
            matrix,
            cell + 1,
            remaining - count,
            objects,
            connected_only,
            relabellings,
            found,
        );
    }
    matrix[cell] = 0;
}

fn connected(matrix: &[usize], objects: usize) -> bool {
    if objects <= 1 {
        return true;
    }
    let mut parent: Vec<usize> = (0..objects).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for source in 0..objects {
        for target in 0..objects {
            if matrix[source * objects + target] > 0 {
                let (a, b) = (find(&mut parent, source), find(&mut parent, target));
                if a != b {
                    parent[a] = b;
                }
            }
        }
    }
    let root = find(&mut parent, 0);
    (1..objects).all(|object| find(&mut parent, object) == root)
}

/// Colour objects by iterated refinement, so that the canonical-form search only
/// has to consider orderings that are consistent with the colouring.
fn colours(matrix: &[usize], objects: usize) -> Vec<usize> {
    let degrees: Vec<[usize; 3]> = (0..objects)
        .map(|v| {
            let out: usize = (0..objects).map(|t| matrix[v * objects + t]).sum();
            let into: usize = (0..objects).map(|s| matrix[s * objects + v]).sum();
            [matrix[v * objects + v], out, into]
        })
        .collect();
    let mut colour = rank(&degrees);
    for _ in 0..2 {
        let refined: Vec<Vec<usize>> = (0..objects)
            .map(|v| {
                let mut signature = vec![colour[v]];
                let mut out: Vec<usize> = (0..objects)
                    .filter(|&t| t != v)
                    .map(|t| matrix[v * objects + t] * objects + colour[t])
                    .collect();
                let mut into: Vec<usize> = (0..objects)
                    .filter(|&s| s != v)
                    .map(|s| matrix[s * objects + v] * objects + colour[s])
                    .collect();
                out.sort_unstable();
                into.sort_unstable();
                signature.extend(out);
                signature.push(usize::MAX);
                signature.extend(into);
                signature
            })
            .collect();
        let next = rank(&refined);
        if next == colour {
            break;
        }
        colour = next;
    }
    colour
}

/// Replace each value by its index in the sorted list of distinct values, so that
/// colours stay small consecutive integers whatever they were computed from.
fn rank<T: Ord + Clone>(values: &[T]) -> Vec<usize> {
    let mut distinct: Vec<T> = values.to_vec();
    distinct.sort();
    distinct.dedup();
    values
        .iter()
        .map(|value| distinct.binary_search(value).expect("value is present"))
        .collect()
}

/// A matrix is canonical when its objects are listed in nondecreasing colour and
/// no colour-preserving reordering makes it smaller.
///
/// Both halves are needed.  Dropping the first would keep matrices that are not
/// among the orderings the minimum is taken over, so two isomorphic matrices
/// could both survive.
fn is_canonical(matrix: &[usize], objects: usize, relabellings: &Relabellings) -> bool {
    let colour = colours(matrix, objects);
    if (1..objects).any(|position| colour[position - 1] > colour[position]) {
        return false;
    }
    let mut order: Vec<usize> = (0..objects).collect();
    order.sort_by_key(|&v| colour[v]);
    // Objects sharing a colour may be listed in any order; try every choice.
    let mut classes: Vec<Vec<usize>> = Vec::new();
    for &object in &order {
        match classes.last() {
            Some(class) if colour[class[0]] == colour[object] => {
                classes.last_mut().unwrap().push(object)
            }
            _ => classes.push(vec![object]),
        }
    }
    let mut permutation = Vec::with_capacity(objects);
    !beats(matrix, objects, &classes, 0, relabellings, &mut permutation)
}

/// Search colour-consistent orderings for one that relabels `matrix` to something
/// strictly smaller.  Returns as soon as one is found.
fn beats(
    matrix: &[usize],
    objects: usize,
    classes: &[Vec<usize>],
    depth: usize,
    relabellings: &Relabellings,
    permutation: &mut Vec<usize>,
) -> bool {
    if depth == classes.len() {
        // `permutation[i]` is the object that ends up at position `i`.
        for i in 0..objects {
            for j in 0..objects {
                let candidate = matrix[permutation[i] * objects + permutation[j]];
                let current = matrix[i * objects + j];
                if candidate < current {
                    return true;
                }
                if candidate > current {
                    return false;
                }
            }
        }
        return false;
    }
    let class = &classes[depth];
    for arrangement in relabellings.of(class.len()) {
        let start = permutation.len();
        permutation.extend(arrangement.iter().map(|&index| class[index]));
        if beats(matrix, objects, classes, depth + 1, relabellings, permutation) {
            permutation.truncate(start);
            return true;
        }
        permutation.truncate(start);
    }
    false
}

/// Choose, for each parallel class, a bijection onto its image class.
fn extend(
    parallel: &[Vec<usize>],
    targets: &[&Vec<usize>],
    choices: &[&[Vec<usize>]],
    depth: usize,
    permutation: &mut Vec<usize>,
    output: &mut Vec<Automorphism>,
) {
    if depth == parallel.len() {
        let mut inverse = vec![0; permutation.len()];
        for (morphism, &image) in permutation.iter().enumerate() {
            inverse[image] = morphism;
        }
        output.push(Automorphism {
            map: permutation.clone(),
            inverse,
        });
        return;
    }
    for arrangement in choices[depth] {
        for (position, &arrow) in parallel[depth].iter().enumerate() {
            permutation[arrow] = targets[depth][arrangement[position]];
        }
        extend(parallel, targets, choices, depth + 1, permutation, output);
    }
}

/// Every permutation of `0..s`, for each `s` up to some bound, computed once.
///
/// The canonicity test asks for these once per colour class per candidate matrix,
/// and there are millions of candidate matrices, so building them on demand
/// dominated shape enumeration.
pub struct Relabellings {
    by_size: Vec<Vec<Vec<usize>>>,
}

impl Relabellings {
    /// Permutations of every size up to `largest`.  Sizes come from two places:
    /// the objects, and the parallel classes of morphisms, which on a one-object
    /// shape hold all `n - 1` nonidentities.
    pub fn new(largest: usize) -> Result<Self> {
        let total: usize = (1..=largest).product::<usize>();
        if total > AUTOMORPHISM_LIMIT {
            return Err(format!(
                "this cell needs all {total} permutations of {largest} labels, over the \
                 {AUTOMORPHISM_LIMIT} we hold in memory; it needs canonical labelling \
                 rather than an explicit group"
            ));
        }
        Ok(Self {
            by_size: (0..=largest).map(permutations).collect(),
        })
    }

    pub fn of(&self, size: usize) -> &[Vec<usize>] {
        &self.by_size[size]
    }
}

/// Every permutation of `0..count`, in lexicographic order.
pub fn permutations(count: usize) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new()];
    for _ in 0..count {
        let mut next = Vec::new();
        for partial in &result {
            for value in 0..count {
                if !partial.contains(&value) {
                    let mut extended = partial.clone();
                    extended.push(value);
                    next.push(extended);
                }
            }
        }
        result = next;
    }
    result
}

/// Turn a multiplicity matrix into a shape, computing its automorphism group.
fn build(
    matrix: &[usize],
    morphisms: usize,
    objects: usize,
    relabellings: &Relabellings,
) -> Result<Shape> {
    let mut domain: Vec<usize> = (0..objects).collect();
    let mut codomain: Vec<usize> = (0..objects).collect();
    for source in 0..objects {
        for target in 0..objects {
            for _ in 0..matrix[source * objects + target] {
                domain.push(source);
                codomain.push(target);
            }
        }
    }
    debug_assert_eq!(domain.len(), morphisms);

    // Nonidentity morphisms with equal endpoints are interchangeable; group them.
    let mut parallel: Vec<Vec<usize>> = Vec::new();
    for arrow in objects..morphisms {
        match parallel.last() {
            Some(class)
                if domain[class[0]] == domain[arrow] && codomain[class[0]] == codomain[arrow] =>
            {
                parallel.last_mut().unwrap().push(arrow)
            }
            _ => parallel.push(vec![arrow]),
        }
    }

    let mut automorphisms: Vec<Automorphism> = Vec::new();
    for object_map in relabellings.of(objects) {
        // Keep only object permutations that preserve every multiplicity.
        let preserves = (0..objects).all(|s| {
            (0..objects)
                .all(|t| matrix[s * objects + t] == matrix[object_map[s] * objects + object_map[t]])
        });
        if !preserves {
            continue;
        }
        // Each parallel class maps onto the class with the permuted endpoints.
        let targets: Vec<&Vec<usize>> = parallel
            .iter()
            .map(|class| {
                let source = object_map[domain[class[0]]];
                let target = object_map[codomain[class[0]]];
                parallel
                    .iter()
                    .find(|other| domain[other[0]] == source && codomain[other[0]] == target)
                    .expect("multiplicities are preserved, so the image class exists")
            })
            .collect();

        let choices: Vec<&[Vec<usize>]> = targets
            .iter()
            .map(|class| relabellings.of(class.len()))
            .collect();
        let total: usize = choices.iter().map(|c| c.len()).product::<usize>();
        if automorphisms.len().saturating_add(total) > AUTOMORPHISM_LIMIT {
            return Err(format!(
                "shape on {objects} objects has more than {AUTOMORPHISM_LIMIT} automorphisms; \
                 this cell needs canonical labelling rather than an explicit group"
            ));
        }

        let mut permutation = vec![0usize; morphisms];
        for (object, &image) in object_map.iter().enumerate() {
            permutation[object] = image;
        }
        extend(
            &parallel,
            &targets,
            &choices,
            0,
            &mut permutation,
            &mut automorphisms,
        );
    }

    Ok(Shape {
        morphisms,
        objects,
        domain,
        codomain,
        automorphisms,
    })
}
