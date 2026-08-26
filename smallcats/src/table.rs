//! The multiplication table of a finite category, and how it is read and written.
//!
//! Morphisms are numbered with the identities first: `0..objects` are the
//! identities, and `objects..morphisms` the rest.  Entry `(f, g)` is the
//! composite `f ∘ g`, or `Undef` when the two do not compose.

use crate::Result;
use std::cmp::Ordering;

/// One cell of a composition table.
///
/// The database stores an undefined composite as the integer `n`, which is a
/// perfectly good wire format but a poor type: its meaning depends on the table's
/// own size, and relabelling code has to remember not to permute it.  Inside the
/// crate an entry is this enum instead, so `relabel` is exhaustive and an
/// out-of-range entry is unrepresentable.
///
/// The variant order matters.  Deriving `Ord` puts `Mor(a) < Mor(b)` exactly when
/// `a < b`, and every `Mor` before `Undef` — which is the ordering the stored
/// canonical form already uses, since `n` exceeds every morphism label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Entry {
    Mor(u8),
    Undef,
}

impl Entry {
    /// Read an entry written the way the database writes it, where `morphisms`
    /// means "these two do not compose".
    pub fn from_index(value: usize, morphisms: usize) -> Result<Self> {
        match value.cmp(&morphisms) {
            Ordering::Less => Ok(Self::Mor(value as u8)),
            Ordering::Equal => Ok(Self::Undef),
            Ordering::Greater => Err(format!("table entry {value} is outside 0..={morphisms}")),
        }
    }

    /// Write an entry the way the database writes it.
    pub fn to_index(self, morphisms: usize) -> usize {
        match self {
            Self::Mor(arrow) => arrow as usize,
            Self::Undef => morphisms,
        }
    }

    /// The composite, when there is one.
    pub fn morphism(self) -> Option<usize> {
        match self {
            Self::Mor(arrow) => Some(arrow as usize),
            Self::Undef => None,
        }
    }

    pub fn is_defined(self) -> bool {
        matches!(self, Self::Mor(_))
    }

    /// Push the entry along a relabelling of the morphisms.  Being a `match`, this
    /// cannot silently permute an undefined composite the way an integer sentinel
    /// invites.
    pub fn relabel(self, labels: &[usize]) -> Self {
        match self {
            Self::Mor(arrow) => Self::Mor(labels[arrow as usize] as u8),
            Self::Undef => Self::Undef,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Table {
    morphisms: usize,
    objects: usize,
    cells: Vec<Entry>,
}

impl Ord for Table {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cells.cmp(&other.cells)
    }
}

impl PartialOrd for Table {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct Endpoints {
    pub domain: Vec<usize>,
    pub codomain: Vec<usize>,
}

impl Table {
    pub fn from_cells(morphisms: usize, objects: usize, cells: Vec<Entry>) -> Result<Self> {
        if morphisms > u8::MAX as usize {
            return Err(format!(
                "{morphisms} morphisms do not fit in the table format"
            ));
        }
        let expected = morphisms
            .checked_mul(morphisms)
            .ok_or_else(|| "table dimensions overflow usize".to_string())?;
        if cells.len() != expected {
            return Err(format!(
                "expected {} table entries, found {}",
                expected,
                cells.len()
            ));
        }
        Ok(Self {
            morphisms,
            objects,
            cells,
        })
    }

    pub fn parse(line: &str, morphisms: usize, objects: usize) -> Result<Self> {
        let rows = Parser::new(line).parse_table()?;
        if rows.len() != morphisms {
            return Err(format!("expected {morphisms} rows, found {}", rows.len()));
        }
        if let Some((index, row)) = rows
            .iter()
            .enumerate()
            .find(|(_, row)| row.len() != morphisms)
        {
            return Err(format!(
                "row {index} has {} entries; expected {morphisms}",
                row.len()
            ));
        }
        let cells = rows
            .into_iter()
            .flatten()
            .map(|value| Entry::from_index(value, morphisms))
            .collect::<Result<Vec<_>>>()?;
        Self::from_cells(morphisms, objects, cells)
    }

    pub fn morphisms(&self) -> usize {
        self.morphisms
    }

    pub fn objects(&self) -> usize {
        self.objects
    }

    pub fn cells(&self) -> &[Entry] {
        &self.cells
    }

    pub fn excess(&self) -> usize {
        self.morphisms - self.objects
    }

    pub fn at(&self, left: usize, right: usize) -> Entry {
        self.cells[left * self.morphisms + right]
    }

    /// The composite of `left` and `right`, when they compose.
    pub fn composite(&self, left: usize, right: usize) -> Option<usize> {
        self.at(left, right).morphism()
    }

    pub fn format_line(&self) -> String {
        let mut output = String::new();
        output.push('[');
        for row in 0..self.morphisms {
            if row > 0 {
                output.push_str(", ");
            }
            output.push('[');
            for column in 0..self.morphisms {
                if column > 0 {
                    output.push_str(", ");
                }
                output.push_str(&self.at(row, column).to_index(self.morphisms).to_string());
            }
            output.push(']');
        }
        output.push(']');
        output
    }

    /// The opposite category: the same objects and morphisms with every arrow
    /// reversed.  Reversing arrows swaps the two arguments of composition, so the
    /// table is simply transposed.
    pub fn opposite(&self) -> Self {
        let n = self.morphisms;
        let cells = (0..n)
            .flat_map(|left| (0..n).map(move |right| (left, right)))
            .map(|(left, right)| self.at(right, left))
            .collect();
        Self {
            morphisms: n,
            objects: self.objects,
            cells,
        }
    }

    /// Validate the table as a category and recover every morphism's endpoints.
    pub fn validate(&self) -> Result<Endpoints> {
        let n = self.morphisms;
        let k = self.objects;
        if n == 0 {
            return if k == 0 {
                Ok(Endpoints {
                    domain: Vec::new(),
                    codomain: Vec::new(),
                })
            } else {
                Err("a category with no morphisms must have no objects".into())
            };
        }
        if k == 0 || k > n {
            return Err(format!("invalid object count {k} for {n} morphisms"));
        }
        let mut domain = Vec::with_capacity(n);
        let mut codomain = Vec::with_capacity(n);
        for arrow in 0..n {
            let right_units: Vec<_> = (0..k)
                .filter(|&object| self.composite(arrow, object) == Some(arrow))
                .collect();
            let left_units: Vec<_> = (0..k)
                .filter(|&object| self.composite(object, arrow) == Some(arrow))
                .collect();
            if right_units.len() != 1 || left_units.len() != 1 {
                return Err(format!(
                    "morphism {arrow} has right units {right_units:?} and left units {left_units:?}"
                ));
            }
            domain.push(right_units[0]);
            codomain.push(left_units[0]);
        }
        for object in 0..k {
            if domain[object] != object || codomain[object] != object {
                return Err(format!("identity label {object} has the wrong endpoints"));
            }
        }

        for left in 0..n {
            for right in 0..n {
                let result = self.composite(left, right);
                let should_be_defined = domain[left] == codomain[right];
                if result.is_some() != should_be_defined {
                    return Err(format!(
                        "definedness mismatch at ({left},{right}): {:?}",
                        self.at(left, right)
                    ));
                }
                if let Some(result) = result {
                    if domain[result] != domain[right] || codomain[result] != codomain[left] {
                        return Err(format!(
                            "typing mismatch at ({left},{right}): result {result}"
                        ));
                    }
                }
            }
        }

        for left in 0..n {
            for middle in 0..n {
                let Some(head) = self.composite(left, middle) else {
                    continue;
                };
                for right in 0..n {
                    let Some(tail) = self.composite(middle, right) else {
                        continue;
                    };
                    let lhs = self.composite(head, right);
                    let rhs = self.composite(left, tail);
                    if lhs.is_none() || lhs != rhs {
                        return Err(format!(
                            "associativity mismatch at ({left},{middle},{right}): {lhs:?} != {rhs:?}"
                        ));
                    }
                }
            }
        }

        Ok(Endpoints { domain, codomain })
    }

    pub fn is_connected(&self, endpoints: &Endpoints) -> bool {
        if self.objects <= 1 {
            return true;
        }
        let mut adjacency = vec![Vec::new(); self.objects];
        for arrow in 0..self.morphisms {
            let source = endpoints.domain[arrow];
            let target = endpoints.codomain[arrow];
            adjacency[source].push(target);
            adjacency[target].push(source);
        }
        let mut seen = vec![false; self.objects];
        seen[0] = true;
        let mut stack = vec![0];
        while let Some(object) = stack.pop() {
            for &neighbor in &adjacency[object] {
                if !seen[neighbor] {
                    seen[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }
        seen.into_iter().all(|value| value)
    }
}

struct Parser<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
        }
    }

    fn parse_table(mut self) -> Result<Vec<Vec<usize>>> {
        self.skip_space();
        self.expect(b'[')?;
        self.skip_space();
        let mut rows = Vec::new();
        if self.peek() != Some(b']') {
            loop {
                rows.push(self.parse_row()?);
                self.skip_space();
                if self.peek() == Some(b',') {
                    self.position += 1;
                    self.skip_space();
                    if self.peek() == Some(b']') {
                        return Err("trailing comma after final row".into());
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(b']')?;
        self.skip_space();
        if self.position != self.input.len() {
            return Err(format!(
                "unexpected trailing input at byte {}",
                self.position
            ));
        }
        Ok(rows)
    }

    fn parse_row(&mut self) -> Result<Vec<usize>> {
        self.expect(b'[')?;
        self.skip_space();
        let mut values = Vec::new();
        if self.peek() != Some(b']') {
            loop {
                values.push(self.parse_number()?);
                self.skip_space();
                if self.peek() == Some(b',') {
                    self.position += 1;
                    self.skip_space();
                    if self.peek() == Some(b']') {
                        return Err("trailing comma in row".into());
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(b']')?;
        Ok(values)
    }

    fn parse_number(&mut self) -> Result<usize> {
        self.skip_space();
        let start = self.position;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.position += 1;
        }
        if start == self.position {
            return Err(format!("expected a nonnegative integer at byte {start}"));
        }
        std::str::from_utf8(&self.input[start..self.position])
            .expect("ASCII digits")
            .parse()
            .map_err(|_| format!("integer at byte {start} is not a table entry"))
    }

    fn expect(&mut self, expected: u8) -> Result<()> {
        self.skip_space();
        match self.peek() {
            Some(found) if found == expected => {
                self.position += 1;
                Ok(())
            }
            Some(found) => Err(format!(
                "expected '{}', found '{}' at byte {}",
                expected as char, found as char, self.position
            )),
            None => Err(format!(
                "expected '{}', found end of input",
                expected as char
            )),
        }
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }
}
