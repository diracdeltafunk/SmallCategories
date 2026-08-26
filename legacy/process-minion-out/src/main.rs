use itertools::Itertools;
use std::env;
use std::io::{prelude::*, BufReader, BufWriter};

#[derive(Clone, Copy)]
enum PermutationBlock {
    // Every label in the block may be permuted independently.
    Symmetric(usize),
    // The block contains adjacent inverse pairs. Pairs may be permuted, and
    // the two labels in each pair may be swapped, but pairs may not be broken.
    InversePairs(usize),
}

impl PermutationBlock {
    fn label_count(&self) -> usize {
        match self {
            Self::Symmetric(size) => *size,
            Self::InversePairs(pairs) => 2 * pairs,
        }
    }
}

fn block_permutations(block: PermutationBlock, start: usize) -> Vec<Vec<usize>> {
    match block {
        PermutationBlock::Symmetric(size) => (start..start + size).permutations(size).collect(),
        PermutationBlock::InversePairs(pairs) => {
            let orientations: Vec<Vec<usize>> = if pairs == 0 {
                vec![Vec::new()]
            } else {
                (0..pairs).map(|_| 0..2).multi_cartesian_product().collect()
            };
            let mut result = Vec::new();

            for pair_order in (0..pairs).permutations(pairs) {
                for orientation in &orientations {
                    let mut permutation = Vec::with_capacity(2 * pairs);
                    for (&pair, &swap) in pair_order.iter().zip(orientation) {
                        permutation.push(start + 2 * pair + swap);
                        permutation.push(start + 2 * pair + 1 - swap);
                    }
                    result.push(permutation);
                }
            }

            result
        }
    }
}

// Generates the direct product of the permutation actions for consecutive
// blocks, standardly embedded in the symmetric group on all labels.
fn perm_group(groupings: &[PermutationBlock]) -> Vec<Vec<usize>> {
    let mut start = 0;
    let mut result = vec![Vec::new()];

    for &block in groupings {
        let block_perms = block_permutations(block, start);
        start += block.label_count();
        result = result
            .into_iter()
            .cartesian_product(block_perms)
            .map(|(mut prefix, suffix)| {
                prefix.extend(suffix);
                prefix
            })
            .collect();
    }

    result
}

// Assumes perm contains every integer from 0 to size-1 exactly once,
// i.e. is a permutation of (0..size)
fn invert_perm(perm: &[usize], size: usize) -> Vec<usize> {
    (0..size)
        .map(|x| perm.iter().position(|y| *y == x).unwrap())
        .collect()
}

// Acts on mat by perm, i.e. pushes the binary operation encoded by mat
// along the bijection encoded by perm
fn act(mat: &[Vec<usize>], perm: &[usize], size: usize) -> Vec<Vec<usize>> {
    let inverted = invert_perm(perm, size);
    let mut result = Vec::new();
    for i in 0..size {
        let mut r = Vec::new();
        for j in 0..size {
            let pre = mat[perm[i]][perm[j]];
            r.push(if pre == size { size } else { inverted[pre] });
        }
        result.push(r);
    }
    result
}

fn to_py_list<T: std::fmt::Display, I: Iterator<Item = T>>(list: I) -> String {
    let mut result = String::from("[");
    result.push_str(&list.map(|x| format!("{}", x)).join(", "));
    result.push(']');
    result
}

fn canonical_form(mat: &[Vec<usize>], perms: &[Vec<usize>], size: usize) -> Vec<Vec<usize>> {
    let mut min = mat.to_vec();
    for p in perms {
        let candidate = act(mat, p, size);
        if candidate < min {
            min = candidate;
        }
    }
    min
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    /*
       Accept arbitrary number of args, but at least 4:
       $0 is command name
       $1 is filename in
       $2 is filename out
       $3 is number of morphisms
       $4.. are grouping specifications (e.g. objects, non-id endos, etc.)
           N means a symmetric block of N consecutive labels.
           pairs:N means N adjacent inverse pairs, acted on by S_2 wr S_N.
    */
    if args.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Incorrect number of arguments",
        ));
    }
    let filename_in: &String = &args[1];
    let filename_out: &String = &args[2];
    let num_morphisms: usize = args[3].parse().unwrap();
    let mut groupings: Vec<PermutationBlock> = args[4..]
        .iter()
        .map(|x| {
            if let Some(pairs) = x.strip_prefix("pairs:") {
                PermutationBlock::InversePairs(pairs.parse().unwrap())
            } else {
                PermutationBlock::Symmetric(x.parse().unwrap())
            }
        })
        .collect();
    let grouped_labels = groupings
        .iter()
        .map(PermutationBlock::label_count)
        .sum::<usize>();
    assert!(num_morphisms >= grouped_labels);
    groupings.push(PermutationBlock::Symmetric(num_morphisms - grouped_labels));

    let file_in = std::fs::File::open(filename_in)?;
    let reader = BufReader::new(file_in);

    let perms = perm_group(&groupings);

    let mut uniques: Vec<Vec<Vec<usize>>> = Vec::new();

    for line in reader.lines() {
        // The line will consist of num_morphisms*num_morphisms many integers, separated by spaces
        // We parse the line into a num_morphisms x num_morphisms array of integers
        let mat: Vec<Vec<usize>> = if num_morphisms > 0 {
            line?
                .split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect::<Vec<usize>>()
                .chunks_exact(num_morphisms)
                .map(Vec::from)
                .collect()
        } else {
            Vec::new()
        };
        let mut keep = true;
        for p in &perms {
            if mat > act(&mat, p, num_morphisms) {
                keep = false;
                break;
            }
        }
        if keep {
            uniques.push(mat);
        }
    }

    // If more than 2 groupings, we must canonize everything
    // via a larger symmetry group.
    if groupings.len() > 2 {
        let num_objects = groupings[0].label_count();
        let coarse_perms = perm_group(&[
            PermutationBlock::Symmetric(num_objects),
            PermutationBlock::Symmetric(num_morphisms - num_objects),
        ]);
        for mat in uniques.iter_mut() {
            *mat = canonical_form(mat, &coarse_perms, num_morphisms);
        }
    }

    // Sort the matrices
    uniques.sort_unstable();

    let file_out = std::fs::File::create(filename_out)?;
    let mut writer = BufWriter::new(file_out);
    let mut num_uniques: usize = 0;
    for mat in uniques {
        // Write mat as a python list to the file
        writer
            .write_all(to_py_list(mat.into_iter().map(|r| to_py_list(r.into_iter()))).as_bytes())?;
        writer.write_all(b"\n")?;
        num_uniques += 1;
    }

    println!("{}", num_uniques);

    Ok(())
}
