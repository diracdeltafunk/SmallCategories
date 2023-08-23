use itertools::Itertools;
use std::env;
use std::io::{prelude::*, BufReader, BufWriter};

fn invert(n: usize, perm: &Vec<usize>, size: usize) -> usize {
    if n == size {
        return size;
    }
    perm.iter().position(|x| (*x) == n).unwrap()
}

fn act(mat: &Vec<Vec<usize>>, perm: &Vec<usize>, size: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    for i in 0..size {
        let mut r = Vec::new();
        for j in 0..size {
            r.push(invert(mat[perm[i]][perm[j]], perm, size));
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // Accept arbitrary number of args, but at least 4:
    // $0 is command name
    // $1 is filename in
    // $2 is filename out
    // $3 is number of morphisms
    // $4.. are groupings (e.g. objects, non-id endos, etc.)
    if args.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Incorrect number of arguments",
        ));
    }
    let filename_in: &String = &args[1];
    let filename_out: &String = &args[2];
    let num_morphisms: usize = args[3].parse().unwrap();
    let groupings: Vec<usize> = args[4..]
        .iter()
        .map(|x| x.parse().unwrap())
        .collect::<Vec<usize>>();
    // let num_objects: usize = args[2].parse().unwrap();

    let file_in = std::fs::File::open(filename_in)?;
    let reader = BufReader::new(file_in);

    let perms: Vec<Vec<usize>> = groupings
        .iter()
        .map(|x| (0..*x).permutations(*x))
        .multi_cartesian_product()
        .map(|l| l.concat())
        .collect();

    let mut uniques: Vec<Vec<Vec<usize>>> = Vec::new();

    for line in reader.lines() {
        // The line will consist of num_morphisms*num_morphisms many integers, separated by spaces
        // We parse the line into a num_morphisms x num_morphisms array of integers
        let mat: Vec<Vec<usize>> = line?
            .split_whitespace()
            .map(|x| x.parse().unwrap())
            .collect::<Vec<usize>>()
            .chunks_exact(num_morphisms)
            .map(Vec::from)
            .collect();
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
