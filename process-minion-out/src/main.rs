use itertools::Itertools;
use std::env;
use std::io::{prelude::*, BufReader, BufWriter};

// Generates the permutation group S_{n_1} × S_{n_2} × ... × S_{n_k}
// as standardly embedded in S_{n_1 + n_2 + ... + n_k},
// where groupings = [n_1, n_2, ..., n_k]
fn perm_group(groupings: &Vec<usize>) -> Vec<Vec<usize>> {
    groupings
        .iter()
        .scan(0, |i, &x| {
            *i += x;
            Some((*i - x..*i).permutations(x))
        })
        .multi_cartesian_product()
        .map(|l| l.concat())
        .collect()
}

// Assumes perm contains every integer from 0 to size-1 exactly once,
// i.e. is a permutation of (0..size)
fn invert_perm(perm: &Vec<usize>, size: usize) -> Vec<usize> {
    (0..size)
        .map(|x| perm.iter().position(|y| *y == x).unwrap())
        .collect()
}

// Acts on mat by perm, i.e. pushes the binary operation encoded by mat
// along the bijection encoded by perm
fn act(mat: &Vec<Vec<usize>>, perm: &Vec<usize>, size: usize) -> Vec<Vec<usize>> {
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
    result.push_str("]");
    return result;
}

fn canonical_form(mat: &Vec<Vec<usize>>, perms: &Vec<Vec<usize>>, size: usize) -> Vec<Vec<usize>> {
    let mut min = mat.clone();
    for p in perms {
        let candidate = act(mat, &p, size);
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
       $4.. are groupings (e.g. objects, non-id endos, etc.)
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
    let mut groupings: Vec<usize> = args[4..]
        .iter()
        .map(|x| x.parse().unwrap())
        .collect::<Vec<usize>>();
    assert!(num_morphisms >= groupings.iter().sum());
    groupings.push(num_morphisms - groupings.iter().sum::<usize>());

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
        let coarse_perms = perm_group(&vec![groupings[0], num_morphisms - groupings[0]]);
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
        writer.write(to_py_list(mat.into_iter().map(|r| to_py_list(r.into_iter()))).as_bytes())?;
        writer.write(b"\n")?;
        num_uniques += 1;
    }

    println!("{}", num_uniques);

    Ok(())
}
