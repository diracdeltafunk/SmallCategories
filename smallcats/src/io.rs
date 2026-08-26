//! Reading and writing cell files.
//!
//! Cells are stored gzipped.  The tables are enormously repetitive — the
//! undefined sentinel alone accounts for most of a sparse table — so they
//! compress by a factor of sixty or more, which is the difference between a
//! 1.8 GB file and a 15 MB one.  GitHub refuses a blob over 100 MB whatever git
//! does with it internally, so this is not an optimisation but a requirement.
//! Read one with `zcat`, or search with `zgrep`.

use crate::canonical::canonicalize;
use crate::table::Table;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use std::io::{Read, Write};
use std::collections::HashMap;
use crate::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn serialize_tables(tables: &[Table]) -> Vec<u8> {
    let mut output = String::new();
    for table in tables {
        output.push_str(&table.format_line());
        output.push('\n');
    }
    output.into_bytes()
}

pub fn write_new(path: &Path, data: &[u8]) -> Result<()> {
    if path.exists() {
        return Err(format!("refusing to overwrite {}", path.display()));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            parent.display()
        ));
    }
    let temporary = temporary_path(path);
    if temporary.exists() {
        return Err(format!(
            "temporary output already exists: {}",
            temporary.display()
        ));
    }
    fs::write(&temporary, data).map_err(|error| format!("{}: {error}", temporary.display()))?;
    if let Err(error) = fs::hard_link(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("{}: {error}", path.display()));
    }
    fs::remove_file(&temporary).map_err(|error| format!("{}: {error}", temporary.display()))?;
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".partial-{}", std::process::id()));
    path.with_file_name(name)
}

#[derive(Clone, Debug)]
pub struct Verification {
    pub tables: usize,
    pub canonical_permutations: u64,
    pub bytes: usize,
}

/// Where a cell's tables live.  The extension is defined here and nowhere else.
pub fn cell_path(database: &Path, morphisms: usize, objects: usize) -> PathBuf {
    database.join(format!("cats{morphisms}-{objects}.txt.gz"))
}

/// Where a cell's proposition masks live.
pub fn propositions_path(database: &Path, morphisms: usize, objects: usize) -> PathBuf {
    database.join(format!("props{morphisms}-{objects}.txt.gz"))
}

/// Read a gzipped text file.
pub fn read_gzipped(path: &Path) -> Result<String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut text = String::new();
    GzDecoder::new(file)
        .read_to_string(&mut text)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(text)
}

/// Write a gzipped text file.
pub fn write_gzipped(path: &Path, data: &[u8]) -> Result<()> {
    let file = fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = GzEncoder::new(file, Compression::best());
    encoder
        .write_all(data)
        .and_then(|()| encoder.finish().map(|_| ()))
        .map_err(|error| format!("{}: {error}", path.display()))
}

/// One proposition mask per line, in the same order as the cell's tables.
pub fn serialize_masks(masks: &[u32]) -> Vec<u8> {
    let mut output = String::new();
    for mask in masks {
        output.push_str(&mask.to_string());
        output.push('\n');
    }
    output.into_bytes()
}

/// Read a proposition file back.
pub fn read_masks(path: &Path) -> Result<Vec<u32>> {
    let text = read_gzipped(path)?;
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            line.parse()
                .map_err(|_| format!("{}:{}: not a proposition mask", path.display(), index + 1))
        })
        .collect()
}

/// Check a cell file as mathematics, not as bytes: every line a valid category of
/// the right dimensions, the lines strictly increasing so the file is a set, and —
/// the part that matters — no two lines describing the same category.
///
/// The generator never computes a canonical form; it does not need one, because
/// the search emits a single table per isomorphism class.  This is where the
/// expensive form earns its keep, as an independent judge.
pub fn verify_file(path: &Path, morphisms: usize, objects: usize) -> Result<Verification> {
    let text = read_gzipped(path)?;
    if !text.is_empty() && !text.ends_with('\n') {
        return Err(format!("{}: missing final newline", path.display()));
    }
    let bytes = text.len();

    let mut tables = Vec::new();
    let mut previous: Option<Table> = None;
    for (line_index, line) in text.lines().enumerate() {
        let at = |error: String| format!("{}:{}: {error}", path.display(), line_index + 1);
        if line.is_empty() {
            return Err(at("blank line".into()));
        }
        let table = Table::parse(line, morphisms, objects).map_err(&at)?;
        table.validate().map_err(&at)?;
        if previous.as_ref().is_some_and(|prior| prior >= &table) {
            return Err(at("tables are not strictly increasing".into()));
        }
        previous = Some(table.clone());
        tables.push(table);
    }

    let classes = canonical_classes(&tables)?;
    let permutations = classes.iter().map(|(_, count)| count).sum();
    let mut seen: HashMap<&Table, usize> = HashMap::new();
    for (index, (canonical, _)) in classes.iter().enumerate() {
        if let Some(earlier) = seen.insert(canonical, index + 1) {
            return Err(format!(
                "{}:{}: describes the same category as line {earlier}",
                path.display(),
                index + 1
            ));
        }
    }

    Ok(Verification {
        tables: tables.len(),
        canonical_permutations: permutations,
        bytes,
    })
}

/// The isomorphism class of every table, as its least relabelling.
pub fn canonical_classes(tables: &[Table]) -> Result<Vec<(Table, u64)>> {
    tables
        .par_iter()
        .map(|table| canonicalize(table).map(|(canonical, stats)| (canonical, stats.candidate_permutations)))
        .collect()
}

/// Read a cell file into tables, without any checking beyond parsing.
pub fn read_cell(path: &Path, morphisms: usize, objects: usize) -> Result<Vec<Table>> {
    let text = read_gzipped(path)?;
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            Table::parse(line, morphisms, objects)
                .map_err(|error| format!("{}:{}: {error}", path.display(), index + 1))
        })
        .collect()
}
