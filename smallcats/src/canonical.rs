//! The canonical form of a table: the lexicographic least relabelling of it.
//!
//! Two tables describe the same category exactly when one is a relabelling of the
//! other under `S_k × S_(n-k)`, so reducing every table to the least member of its
//! orbit gives one representative per isomorphism class.  This is the form the
//! database stores.

use crate::table::{Endpoints, Entry, Table};
use crate::Result;

#[derive(Clone, Copy, Debug, Default)]
pub struct CanonicalStats {
    pub candidate_permutations: u64,
}

/// Return the exact lexicographically least table under S_k × S_(n-k).
///
/// This search never materializes a permutation group or allocates a table per
/// permutation.  Identity rows force two safe reductions.  In a least table,
/// objects occur in nonincreasing order of their number of incoming
/// nonidentity morphisms, and nonidentity labels form consecutive codomain
/// blocks in object order.  The search enumerates only ties and permutations
/// inside those blocks, reusing one candidate buffer.
pub fn canonicalize(table: &Table) -> Result<(Table, CanonicalStats)> {
    let endpoints = table.validate()?;
    if table.morphisms() == 0 {
        return Ok((table.clone(), CanonicalStats::default()));
    }
    let mut search = CanonicalSearch::new(table, endpoints);
    search.search_object_buckets(0);
    let best = search
        .best
        .ok_or_else(|| "canonical search produced no permutation".to_string())?;
    Ok((
        Table::from_cells(table.morphisms(), table.objects(), best)?,
        CanonicalStats {
            candidate_permutations: search.leaves,
        },
    ))
}

struct CanonicalSearch<'a> {
    table: &'a Table,
    endpoints: Endpoints,
    buckets: Vec<Vec<usize>>,
    isolated: Vec<bool>,
    permutation: Vec<usize>,
    inverse: Vec<usize>,
    scratch: Vec<Entry>,
    best: Option<Vec<Entry>>,
    leaves: u64,
}

impl<'a> CanonicalSearch<'a> {
    fn new(table: &'a Table, endpoints: Endpoints) -> Self {
        let n = table.morphisms();
        let k = table.objects();
        let mut incoming = vec![0; k];
        let mut isolated = vec![true; k];
        for arrow in k..n {
            incoming[endpoints.codomain[arrow]] += 1;
            isolated[endpoints.domain[arrow]] = false;
            isolated[endpoints.codomain[arrow]] = false;
        }
        let mut buckets = Vec::new();
        for count in (0..=n - k).rev() {
            let bucket: Vec<_> = (0..k).filter(|&object| incoming[object] == count).collect();
            if !bucket.is_empty() {
                buckets.push(bucket);
            }
        }
        Self {
            table,
            endpoints,
            buckets,
            isolated,
            permutation: Vec::with_capacity(n),
            inverse: vec![0; n],
            scratch: vec![Entry::Undef; n * n],
            best: None,
            leaves: 0,
        }
    }

    fn search_object_buckets(&mut self, bucket_index: usize) {
        if bucket_index == self.buckets.len() {
            self.search_arrows();
            return;
        }
        let items = self.buckets[bucket_index].clone();
        let mut used = vec![false; items.len()];
        let mut chosen = Vec::with_capacity(items.len());
        self.permute_object_bucket(bucket_index, &items, &mut used, &mut chosen);
    }

    fn permute_object_bucket(
        &mut self,
        bucket_index: usize,
        items: &[usize],
        used: &mut [bool],
        chosen: &mut Vec<usize>,
    ) {
        if chosen.len() == items.len() {
            let original_len = self.permutation.len();
            self.permutation.extend(chosen.iter().copied());
            self.search_object_buckets(bucket_index + 1);
            self.permutation.truncate(original_len);
            return;
        }

        // Isolated objects are genuinely indistinguishable; choose only one
        // unused isolated label at each depth while still allowing its position
        // relative to nonisolated objects to vary.
        let mut chose_isolated = false;
        for index in 0..items.len() {
            if used[index] {
                continue;
            }
            let object = items[index];
            if self.isolated[object] {
                if chose_isolated {
                    continue;
                }
                chose_isolated = true;
            }
            used[index] = true;
            chosen.push(object);
            self.permute_object_bucket(bucket_index, items, used, chosen);
            chosen.pop();
            used[index] = false;
        }
    }

    fn search_arrows(&mut self) {
        let k = self.table.objects();
        debug_assert_eq!(self.permutation.len(), k);
        let mut groups = Vec::with_capacity(k);
        for &old_object in &self.permutation[..k] {
            groups.push(
                (k..self.table.morphisms())
                    .filter(|&arrow| self.endpoints.codomain[arrow] == old_object)
                    .collect::<Vec<_>>(),
            );
        }
        self.permute_arrow_groups(&mut groups, 0, 0);
    }

    fn permute_arrow_groups(
        &mut self,
        groups: &mut [Vec<usize>],
        group_index: usize,
        position: usize,
    ) {
        if group_index == groups.len() {
            self.consider_current_permutation();
            return;
        }
        if position == groups[group_index].len() {
            let original_len = self.permutation.len();
            self.permutation.extend(groups[group_index].iter().copied());
            self.permute_arrow_groups(groups, group_index + 1, 0);
            self.permutation.truncate(original_len);
            return;
        }
        for index in position..groups[group_index].len() {
            groups[group_index].swap(position, index);
            self.permute_arrow_groups(groups, group_index, position + 1);
            groups[group_index].swap(position, index);
        }
    }

    fn consider_current_permutation(&mut self) {
        let n = self.table.morphisms();
        debug_assert_eq!(self.permutation.len(), n);
        for (new, &old) in self.permutation.iter().enumerate() {
            self.inverse[old] = new;
        }
        for new_left in 0..n {
            for new_right in 0..n {
                self.scratch[new_left * n + new_right] = self
                    .table
                    .at(self.permutation[new_left], self.permutation[new_right])
                    .relabel(&self.inverse);
            }
        }
        self.leaves += 1;
        if self
            .best
            .as_ref()
            .is_none_or(|best| self.scratch.as_slice() < best.as_slice())
        {
            self.best = Some(self.scratch.clone());
        }
    }
}
