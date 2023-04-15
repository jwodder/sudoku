use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

static DIVIDER: &str = "+-----+-----+-----+";

/// An unsolved Sudoku puzzle.
///
/// `Puzzle` instances can be constructed by converting from a grid of `u8`
/// values using [`TryFrom`]/[`TryInto`] or from a string using
/// [`FromStr`]/[`str::parse()`].  See the details on the trait implementations
/// below for more details.
///
/// As `Puzzle` implements `Deref<[[u8; 9]; 9]>`, it can be indexed to obtain
/// the individual rows of the puzzle; "unfilled" cells are represented by 0.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Puzzle([[u8; 9]; 9]);

impl Puzzle {
    /// Solve the puzzle.
    ///
    /// If the puzzle has multiple solutions, one of them is returned, but
    /// which one is unspecified.
    ///
    /// If the puzzle has no solutions, `None` is returned.
    pub fn solve(&self) -> Option<Solution> {
        let mut scratch = InProgress::new(self);
        let mut coords = Coords::new();
        'iloop: while let Some((i, j)) = coords.get() {
            if let Some(o) = scratch.obstructions[i][j] {
                let mut next_test = scratch.puzzle[i][j];
                if next_test != 0 {
                    scratch.remove_obstruction(i, j);
                    scratch.puzzle[i][j] = 0;
                }
                next_test += 1;
                while next_test <= 9 {
                    if o.for_number(next_test) == 0 {
                        scratch.puzzle[i][j] = next_test;
                        scratch.add_obstruction(i, j);
                        break;
                    }
                    next_test += 1;
                }
                if next_test > 9 {
                    // Backtrack
                    while let Some((i2, j2)) = coords.retreat() {
                        if let Some(o2) = scratch.obstructions[i2][j2] {
                            if !o2.is_full() {
                                continue 'iloop;
                            }
                            scratch.remove_obstruction(i2, j2);
                            scratch.puzzle[i2][j2] = 0;
                        }
                    }
                    return None;
                }
            }
            coords.advance();
        }
        Some(Solution(scratch.puzzle))
    }
}

struct InProgress {
    puzzle: [[u8; 9]; 9],
    obstructions: [[Option<Obstruction>; 9]; 9],
    // None = cell in input puzzle was already filled
}

impl InProgress {
    fn new(p: &Puzzle) -> Self {
        let mut scratch = Self {
            obstructions: [[Some(Obstruction::new()); 9]; 9],
            puzzle: p.0,
        };
        for i in 0..9 {
            for j in 0..9 {
                if scratch.puzzle[i][j] != 0 {
                    scratch.obstructions[i][j] = None;
                    scratch.add_obstruction(i, j);
                }
            }
        }
        scratch
    }

    fn adjust_obstruction<F>(&mut self, func: &F, y: usize, x: usize)
    where
        F: Fn(&mut Obstruction),
    {
        if let Some(o) = self.obstructions[y][x].as_mut() {
            func(o);
        }
    }

    fn foreach_obstructed<F>(&mut self, func: F, y: usize, x: usize)
    where
        F: Fn(&mut Obstruction),
    {
        for i in 0..9 {
            if i != x {
                self.adjust_obstruction(&func, y, i);
            }
            if i != y {
                self.adjust_obstruction(&func, i, x);
            }
        }
        let t1 = y % 3;
        let t2 = x % 3;
        let x0 = x - t2;
        let y0 = y - t1;
        self.adjust_obstruction(&func, y0 + (t1 + 1) % 3, x0 + (t2 + 1) % 3);
        self.adjust_obstruction(&func, y0 + (t1 + 1) % 3, x0 + (t2 + 2) % 3);
        self.adjust_obstruction(&func, y0 + (t1 + 2) % 3, x0 + (t2 + 1) % 3);
        self.adjust_obstruction(&func, y0 + (t1 + 2) % 3, x0 + (t2 + 2) % 3);
    }

    fn add_obstruction(&mut self, y: usize, x: usize) {
        let num = self.puzzle[y][x];
        self.foreach_obstructed(|o| o.add(num), y, x);
    }

    fn remove_obstruction(&mut self, y: usize, x: usize) {
        let num = self.puzzle[y][x];
        self.foreach_obstructed(|o| o.remove(num), y, x);
    }
}

/// Counts the amount of cells (max value 3) of each numeric value that
/// "obstruct" a given cell
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct Obstruction([u8; 9]);

impl Obstruction {
    fn new() -> Self {
        Obstruction([0; 9])
    }

    fn add(&mut self, number: u8) {
        self.0[usize::from(number) - 1] += 1;
    }

    fn remove(&mut self, number: u8) {
        self.0[usize::from(number) - 1] -= 1;
    }

    fn for_number(&self, number: u8) -> u8 {
        self.0[usize::from(number) - 1]
    }

    fn is_full(&self) -> bool {
        self.0.iter().all(|&x| x == 3)
    }
}

struct Coords(Option<(usize, usize)>);

impl Coords {
    fn new() -> Coords {
        Coords(Some((0, 0)))
    }

    fn get(&self) -> Option<(usize, usize)> {
        self.0
    }

    fn advance(&mut self) {
        if let Some((mut i, mut j)) = self.0 {
            j += 1;
            if j >= 9 {
                i += 1;
                j = 0;
            }
            if i >= 9 {
                self.0 = None;
            } else {
                self.0 = Some((i, j));
            }
        }
    }

    fn retreat(&mut self) -> Option<(usize, usize)> {
        let (mut i, mut j) = self.0?;
        j = match j.checked_sub(1) {
            Some(j2) => j2,
            None => {
                i = i.checked_sub(1)?;
                8
            }
        };
        self.0 = Some((i, j));
        Some((i, j))
    }
}

/// Error type returned when trying to construct a [`Puzzle`] from invalid
/// input
#[derive(Copy, Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TryIntoPuzzleError {
    /// Returned when the input contains a cell with a value larger than 9.
    /// The argument is the value of the cell in question.
    #[error("cell value {0} is too large")]
    NumTooBig(u8),

    /// Returned when the input grid contains a row that is not exactly 9 cells
    /// long
    #[error("row not 9 cells long")]
    BadRowSize,

    /// Returned when the input grid is not exactly 9 rows long
    #[error("grid not 9 rows long")]
    BadGridSize,
}

/// Convert a 9×9 grid into a [`Puzzle`].  Cell values must be in the range
/// `0..=9`, where 0 represents an "unfilled" cell.
///
/// # Errors
///
/// Fails if any cell has a value larger than 9.
impl TryFrom<[[u8; 9]; 9]> for Puzzle {
    type Error = TryIntoPuzzleError;

    fn try_from(value: [[u8; 9]; 9]) -> Result<Puzzle, TryIntoPuzzleError> {
        for row in &value {
            for &cell in row {
                if cell > 9 {
                    return Err(TryIntoPuzzleError::NumTooBig(cell));
                }
            }
        }
        Ok(Puzzle(value))
    }
}

/// Convert a slice of `u8` arrays into a [`Puzzle`].  Cell values must be in
/// the range `0..=9`, where 0 represents an "unfilled" cell.
///
/// # Errors
///
/// Fails if any cell has a value larger than 9 or if the grid is not exactly
/// 9×9.
impl<T: AsRef<[u8]>> TryFrom<&[T]> for Puzzle {
    type Error = TryIntoPuzzleError;

    fn try_from(value: &[T]) -> Result<Puzzle, TryIntoPuzzleError> {
        let mut grid = Vec::with_capacity(9);
        for row in value {
            let row =
                <[u8; 9]>::try_from(row.as_ref()).map_err(|_| TryIntoPuzzleError::BadRowSize)?;
            grid.push(row);
        }
        <[[u8; 9]; 9]>::try_from(grid.as_slice())
            .map_err(|_| TryIntoPuzzleError::BadGridSize)?
            .try_into()
    }
}

/// Convert a [`Vec`] of `u8` arrays into a [`Puzzle`].  Cell values must be in
/// the range `0..=9`, where 0 represents an "unfilled" cell.
///
/// # Errors
///
/// Fails if any cell has a value larger than 9 or if the grid is not exactly
/// 9×9.
impl<T: AsRef<[u8]>> TryFrom<Vec<T>> for Puzzle {
    type Error = TryIntoPuzzleError;

    fn try_from(v: Vec<T>) -> Result<Puzzle, TryIntoPuzzleError> {
        Puzzle::try_from(&v[..])
    }
}

/// Parse a [`Puzzle`] from a string consisting of nine lines of nine cells
/// each, where each cell is either a digit in `0..=9` (0 representing
/// an "unfilled" cell) or any non-space, non-digit character (also
/// representing an "unfilled" cell).  Horizontal whitespace and blank lines
/// are ignored.
///
/// For example, the following input:
///
/// ```text
/// 000780500
/// 200650700
/// 000000630
/// 010000070
/// 000506000
/// 060000020
/// 087000000
/// 003017009
/// 004092000
/// ```
///
/// is parsed the same as this input:
///
/// ```text
/// . . .  7 8 .  5 . .
/// 2 . .  6 5 .  7 . .
/// . . .  . . .  6 3 .
///
/// . 1 .  . . .  . 7 .
/// . . .  5 . 6  . . .
/// . 6 .  . . .  . 2 .
///
/// . 8 7  . . .  . . .
/// . . 3  . 1 7  . . 9
/// . . 4  . 9 2  . . .
/// ```
///
/// # Errors
///
/// Fails if the input grid is not exactly 9×9.
impl FromStr for Puzzle {
    type Err = TryIntoPuzzleError;

    fn from_str(s: &str) -> Result<Puzzle, TryIntoPuzzleError> {
        let mut grid = Vec::with_capacity(9);
        for line in s.lines() {
            let mut row = Vec::with_capacity(9);
            for c in line.chars() {
                if let Some(x) = c.to_digit(10) {
                    row.push(u8::try_from(x).unwrap());
                } else if !c.is_whitespace() {
                    row.push(0);
                }
            }
            if !row.is_empty() {
                grid.push(row);
            }
        }
        grid.try_into()
    }
}

impl Deref for Puzzle {
    type Target = [[u8; 9]; 9];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Display a [`Puzzle`] as nine lines of nine cells.
///
/// In the default representation, "unfilled" cells are represented by `0`, and
/// there is no horizontal whitespace, e.g.:
///
/// ```text
/// 003020600
/// 900305001
/// 001806400
/// 008102900
/// 700000008
/// 006708200
/// 002609500
/// 800203009
/// 005010300
/// ```
///
/// In the alternate representation (selected with the `#` modifier), a border
/// is drawn around the grid and between regions, adjacent cells are separated
/// with a space, and "unfilled" cells are represented by a space, e.g.:
///
/// ```text
/// +-----+-----+-----+
/// |    3|  2  |6    |
/// |9    |3   5|    1|
/// |    1|8   6|4    |
/// +-----+-----+-----+
/// |    8|1   2|9    |
/// |7    |     |    8|
/// |    6|7   8|2    |
/// +-----+-----+-----+
/// |    2|6   9|5    |
/// |8    |2   3|    9|
/// |    5|  1  |3    |
/// +-----+-----+-----+
/// ```
///
/// Both forms lack a final terminating newline.
impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            for y in 0..9 {
                if y % 3 == 0 {
                    writeln!(f, "{DIVIDER}")?;
                }
                for x in 0..9 {
                    write!(f, "{}", if x % 3 == 0 { '|' } else { ' ' })?;
                    let c = self.0[y][x];
                    if c == 0 {
                        write!(f, " ")?;
                    } else {
                        write!(f, "{c}")?;
                    }
                }
                writeln!(f, "|")?;
            }
            write!(f, "{DIVIDER}")?;
        } else {
            for y in 0..9 {
                if y > 0 {
                    writeln!(f)?;
                }
                for x in 0..9 {
                    write!(f, "{}", self.0[y][x])?;
                }
            }
        }
        Ok(())
    }
}

/// A solution to a Sudoku puzzle.
///
/// `Solution` instances are returned by [`Puzzle::solve`].
///
/// As `Solution` implements `Deref<[[u8; 9]; 9]>`, it can be indexed to obtain
/// the individual rows of the solution.  Alternatively, a `Solution` can be
/// converted directly to a `[[u8; 9]; 9]` via the [`From`]/[`Into`] traits.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Solution([[u8; 9]; 9]);

impl Deref for Solution {
    type Target = [[u8; 9]; 9];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Solution> for [[u8; 9]; 9] {
    fn from(value: Solution) -> [[u8; 9]; 9] {
        value.0
    }
}

/// Display a [`Solution`] as nine lines of nine cells.
///
/// In the default representation, there is no horizontal whitespace, e.g.:
///
/// ```text
/// 483921657
/// 967345821
/// 251876493
/// 548132976
/// 729564138
/// 136798245
/// 372689514
/// 814253769
/// 695417382
/// ```
///
/// In the alternate representation (selected with the `#` modifier), a border
/// is drawn around the grid and between regions and adjacent cells are
/// separated with a space, e.g.:
///
/// ```text
/// +-----+-----+-----+
/// |4 8 3|9 2 1|6 5 7|
/// |9 6 7|3 4 5|8 2 1|
/// |2 5 1|8 7 6|4 9 3|
/// +-----+-----+-----+
/// |5 4 8|1 3 2|9 7 6|
/// |7 2 9|5 6 4|1 3 8|
/// |1 3 6|7 9 8|2 4 5|
/// +-----+-----+-----+
/// |3 7 2|6 8 9|5 1 4|
/// |8 1 4|2 5 3|7 6 9|
/// |6 9 5|4 1 7|3 8 2|
/// +-----+-----+-----+
/// ```
///
/// Both forms lack a final terminating newline.
impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            for y in 0..9 {
                if y % 3 == 0 {
                    writeln!(f, "{DIVIDER}")?;
                }
                for x in 0..9 {
                    write!(f, "{}{}", if x % 3 == 0 { '|' } else { ' ' }, self.0[y][x])?;
                }
                writeln!(f, "|")?;
            }
            write!(f, "{DIVIDER}")?;
        } else {
            for y in 0..9 {
                if y > 0 {
                    writeln!(f)?;
                }
                for x in 0..9 {
                    write!(f, "{}", self.0[y][x])?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display_puzzle() {
        let puzzle = Puzzle([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 5, 0, 0, 1],
            [0, 0, 1, 8, 0, 6, 4, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ]);
        assert_eq!(
            puzzle.to_string(),
            concat!(
                "003020600\n",
                "900305001\n",
                "001806400\n",
                "008102900\n",
                "700000008\n",
                "006708200\n",
                "002609500\n",
                "800203009\n",
                "005010300",
            )
        );
        assert_eq!(
            format!("{puzzle:#}"),
            concat!(
                "+-----+-----+-----+\n",
                "|    3|  2  |6    |\n",
                "|9    |3   5|    1|\n",
                "|    1|8   6|4    |\n",
                "+-----+-----+-----+\n",
                "|    8|1   2|9    |\n",
                "|7    |     |    8|\n",
                "|    6|7   8|2    |\n",
                "+-----+-----+-----+\n",
                "|    2|6   9|5    |\n",
                "|8    |2   3|    9|\n",
                "|    5|  1  |3    |\n",
                "+-----+-----+-----+",
            ),
        );
    }

    #[test]
    fn test_display_solution() {
        let solution = Solution([
            [4, 8, 3, 9, 2, 1, 6, 5, 7],
            [9, 6, 7, 3, 4, 5, 8, 2, 1],
            [2, 5, 1, 8, 7, 6, 4, 9, 3],
            [5, 4, 8, 1, 3, 2, 9, 7, 6],
            [7, 2, 9, 5, 6, 4, 1, 3, 8],
            [1, 3, 6, 7, 9, 8, 2, 4, 5],
            [3, 7, 2, 6, 8, 9, 5, 1, 4],
            [8, 1, 4, 2, 5, 3, 7, 6, 9],
            [6, 9, 5, 4, 1, 7, 3, 8, 2],
        ]);
        assert_eq!(
            solution.to_string(),
            concat!(
                "483921657\n",
                "967345821\n",
                "251876493\n",
                "548132976\n",
                "729564138\n",
                "136798245\n",
                "372689514\n",
                "814253769\n",
                "695417382",
            )
        );
        assert_eq!(
            format!("{solution:#}"),
            concat!(
                "+-----+-----+-----+\n",
                "|4 8 3|9 2 1|6 5 7|\n",
                "|9 6 7|3 4 5|8 2 1|\n",
                "|2 5 1|8 7 6|4 9 3|\n",
                "+-----+-----+-----+\n",
                "|5 4 8|1 3 2|9 7 6|\n",
                "|7 2 9|5 6 4|1 3 8|\n",
                "|1 3 6|7 9 8|2 4 5|\n",
                "+-----+-----+-----+\n",
                "|3 7 2|6 8 9|5 1 4|\n",
                "|8 1 4|2 5 3|7 6 9|\n",
                "|6 9 5|4 1 7|3 8 2|\n",
                "+-----+-----+-----+",
            )
        );
    }

    #[test]
    fn test_solve01() {
        let puzzle = Puzzle([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 5, 0, 0, 1],
            [0, 0, 1, 8, 0, 6, 4, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ]);
        let solution = Solution([
            [4, 8, 3, 9, 2, 1, 6, 5, 7],
            [9, 6, 7, 3, 4, 5, 8, 2, 1],
            [2, 5, 1, 8, 7, 6, 4, 9, 3],
            [5, 4, 8, 1, 3, 2, 9, 7, 6],
            [7, 2, 9, 5, 6, 4, 1, 3, 8],
            [1, 3, 6, 7, 9, 8, 2, 4, 5],
            [3, 7, 2, 6, 8, 9, 5, 1, 4],
            [8, 1, 4, 2, 5, 3, 7, 6, 9],
            [6, 9, 5, 4, 1, 7, 3, 8, 2],
        ]);
        assert_eq!(puzzle.solve().unwrap(), solution);
    }

    #[test]
    fn test_solve02() {
        let puzzle = Puzzle([
            [2, 0, 0, 0, 8, 0, 3, 0, 0],
            [0, 6, 0, 0, 7, 0, 0, 8, 4],
            [0, 3, 0, 5, 0, 0, 2, 0, 9],
            [0, 0, 0, 1, 0, 5, 4, 0, 8],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [4, 0, 2, 7, 0, 6, 0, 0, 0],
            [3, 0, 1, 0, 0, 7, 0, 4, 0],
            [7, 2, 0, 0, 4, 0, 0, 6, 0],
            [0, 0, 4, 0, 1, 0, 0, 0, 3],
        ]);
        let solution = Solution([
            [2, 4, 5, 9, 8, 1, 3, 7, 6],
            [1, 6, 9, 2, 7, 3, 5, 8, 4],
            [8, 3, 7, 5, 6, 4, 2, 1, 9],
            [9, 7, 6, 1, 2, 5, 4, 3, 8],
            [5, 1, 3, 4, 9, 8, 6, 2, 7],
            [4, 8, 2, 7, 3, 6, 9, 5, 1],
            [3, 9, 1, 6, 5, 7, 8, 4, 2],
            [7, 2, 8, 3, 4, 9, 1, 6, 5],
            [6, 5, 4, 8, 1, 2, 7, 9, 3],
        ]);
        assert_eq!(puzzle.solve().unwrap(), solution);
    }

    #[test]
    fn test_solve03() {
        let puzzle = Puzzle([
            [0, 0, 0, 0, 0, 0, 9, 0, 7],
            [0, 0, 0, 4, 2, 0, 1, 8, 0],
            [0, 0, 0, 7, 0, 5, 0, 2, 6],
            [1, 0, 0, 9, 0, 4, 0, 0, 0],
            [0, 5, 0, 0, 0, 0, 0, 4, 0],
            [0, 0, 0, 5, 0, 7, 0, 0, 9],
            [9, 2, 0, 1, 0, 8, 0, 0, 0],
            [0, 3, 4, 0, 5, 9, 0, 0, 0],
            [5, 0, 7, 0, 0, 0, 0, 0, 0],
        ]);
        let solution = Solution([
            [4, 6, 2, 8, 3, 1, 9, 5, 7],
            [7, 9, 5, 4, 2, 6, 1, 8, 3],
            [3, 8, 1, 7, 9, 5, 4, 2, 6],
            [1, 7, 3, 9, 8, 4, 2, 6, 5],
            [6, 5, 9, 3, 1, 2, 7, 4, 8],
            [2, 4, 8, 5, 6, 7, 3, 1, 9],
            [9, 2, 6, 1, 7, 8, 5, 3, 4],
            [8, 3, 4, 2, 5, 9, 6, 7, 1],
            [5, 1, 7, 6, 4, 3, 8, 9, 2],
        ]);
        assert_eq!(puzzle.solve().unwrap(), solution);
    }

    #[test]
    fn test_solve_ambiguous() {
        // From <https://math.stackexchange.com/a/345255/10655>
        let puzzle = Puzzle([
            [1, 4, 5, 3, 2, 7, 6, 9, 8],
            [8, 3, 9, 6, 5, 4, 1, 2, 7],
            [6, 7, 2, 9, 1, 8, 5, 4, 3],
            [4, 9, 6, 0, 8, 5, 3, 7, 0],
            [2, 1, 8, 4, 7, 3, 9, 5, 6],
            [7, 5, 3, 0, 9, 6, 4, 8, 0],
            [3, 6, 7, 5, 4, 2, 8, 1, 9],
            [9, 8, 4, 7, 6, 1, 2, 3, 5],
            [5, 2, 1, 8, 3, 9, 7, 6, 4],
        ]);
        let Solution(grid) = puzzle.solve().unwrap();
        for row in grid {
            for c in row {
                assert_ne!(c, 0);
            }
        }
    }

    #[test]
    fn test_solve_unsolvable() {
        // From <https://www.reddit.com/r/sudoku/comments/7q76ay/>
        let puzzle = Puzzle([
            [2, 0, 0, 9, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 6, 0],
            [0, 0, 0, 0, 0, 1, 0, 0, 0],
            [5, 0, 2, 6, 0, 0, 4, 0, 7],
            [0, 0, 0, 0, 0, 4, 1, 0, 0],
            [0, 0, 0, 0, 9, 8, 0, 2, 3],
            [0, 0, 0, 0, 0, 3, 0, 8, 0],
            [0, 0, 5, 0, 1, 0, 0, 0, 0],
            [0, 0, 7, 0, 0, 0, 0, 0, 0],
        ]);
        assert_eq!(puzzle.solve(), None);
    }

    #[test]
    fn test_try_from_array() {
        let p1 = Puzzle::try_from([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 5, 0, 0, 1],
            [0, 0, 1, 8, 0, 6, 4, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ])
        .unwrap();
        let p2 = Puzzle([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 5, 0, 0, 1],
            [0, 0, 1, 8, 0, 6, 4, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ]);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_try_from_vec() {
        let p1 = Puzzle::try_from(vec![
            vec![0, 0, 3, 0, 2, 0, 6, 0, 0],
            vec![9, 0, 0, 3, 0, 5, 0, 0, 1],
            vec![0, 0, 1, 8, 0, 6, 4, 0, 0],
            vec![0, 0, 8, 1, 0, 2, 9, 0, 0],
            vec![7, 0, 0, 0, 0, 0, 0, 0, 8],
            vec![0, 0, 6, 7, 0, 8, 2, 0, 0],
            vec![0, 0, 2, 6, 0, 9, 5, 0, 0],
            vec![8, 0, 0, 2, 0, 3, 0, 0, 9],
            vec![0, 0, 5, 0, 1, 0, 3, 0, 0],
        ])
        .unwrap();
        let p2 = Puzzle([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 5, 0, 0, 1],
            [0, 0, 1, 8, 0, 6, 4, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ]);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_try_from_array_with_invalid_cell() {
        let r = Puzzle::try_from([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 50, 0, 0, 11],
            [0, 0, 13, 8, 0, 6, 42, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ]);
        assert_eq!(r, Err(TryIntoPuzzleError::NumTooBig(50)));
    }

    #[test]
    fn test_try_from_slices_with_long_row() {
        let r = Puzzle::try_from(
            [
                [0, 0, 3, 0, 2, 0, 6, 0, 0].as_slice(),
                [9, 0, 0, 3, 0, 5, 0, 0, 1].as_slice(),
                [0, 0, 1, 8, 0, 6, 4, 0, 0, 42, 113].as_slice(),
                [0, 0, 8, 1, 0, 2, 9, 0, 0].as_slice(),
                [7, 0, 0, 0, 0, 0, 0, 0, 8].as_slice(),
                [0, 0, 6, 7, 0, 8, 2, 0, 0].as_slice(),
                [0, 0, 2, 6, 0, 9, 5, 0, 0].as_slice(),
                [8, 0, 0, 2, 0, 3, 0, 0, 9].as_slice(),
                [0, 0, 5, 0, 1, 0, 3, 0, 0].as_slice(),
            ]
            .as_slice(),
        );
        assert_eq!(r, Err(TryIntoPuzzleError::BadRowSize));
    }

    #[test]
    fn test_try_from_long_slice_with_short_row() {
        let r = Puzzle::try_from(
            [
                [0, 0, 3, 0, 2, 0, 6, 0, 0].as_slice(),
                [9, 0, 0, 3, 0, 5, 0, 0, 1].as_slice(),
                [0, 0, 1, 8, 0, 6, 4].as_slice(),
                [0, 0, 8, 1, 0, 2, 9, 0, 0].as_slice(),
                [7, 0, 0, 0, 0, 0, 0, 0, 8].as_slice(),
                [0, 0, 6, 7, 0, 8, 2, 0, 0].as_slice(),
                [0, 0, 2, 6, 0, 9, 5, 0, 0].as_slice(),
                [8, 0, 0, 2, 0, 3, 0, 0, 9].as_slice(),
                [0, 0, 5, 0, 1, 0, 3, 0, 0].as_slice(),
                [9, 8, 7, 6, 5, 4, 3, 2, 1].as_slice(),
                [9, 8, 7, 6, 5, 4, 3, 2, 1].as_slice(),
            ]
            .as_slice(),
        );
        assert_eq!(r, Err(TryIntoPuzzleError::BadRowSize));
    }

    #[test]
    fn test_try_from_long_slice() {
        let r = Puzzle::try_from(
            [
                [0, 0, 3, 0, 2, 0, 6, 0, 0].as_slice(),
                [9, 0, 0, 3, 0, 5, 0, 0, 1].as_slice(),
                [0, 0, 1, 8, 0, 6, 4, 0, 0].as_slice(),
                [0, 0, 8, 1, 0, 2, 9, 0, 0].as_slice(),
                [7, 0, 0, 0, 0, 0, 0, 0, 8].as_slice(),
                [0, 0, 6, 7, 0, 8, 2, 0, 0].as_slice(),
                [0, 0, 2, 6, 0, 9, 5, 0, 0].as_slice(),
                [8, 0, 0, 2, 0, 3, 0, 0, 9].as_slice(),
                [0, 0, 5, 0, 1, 0, 3, 0, 0].as_slice(),
                [9, 8, 7, 6, 5, 4, 3, 2, 1].as_slice(),
                [9, 8, 7, 6, 5, 4, 3, 2, 1].as_slice(),
            ]
            .as_slice(),
        );
        assert_eq!(r, Err(TryIntoPuzzleError::BadGridSize));
    }

    #[test]
    fn test_try_from_short_slice() {
        let r = Puzzle::try_from(
            [
                [0, 0, 3, 0, 2, 0, 6, 0, 0].as_slice(),
                [9, 0, 0, 3, 0, 5, 0, 0, 1].as_slice(),
                [0, 0, 1, 8, 0, 6, 4, 0, 0].as_slice(),
                [0, 0, 8, 1, 0, 2, 9, 0, 0].as_slice(),
                [7, 0, 0, 0, 0, 0, 0, 0, 8].as_slice(),
                [0, 0, 6, 7, 0, 8, 2, 0, 0].as_slice(),
                [0, 0, 2, 6, 0, 9, 5, 0, 0].as_slice(),
                [8, 0, 0, 2, 0, 3, 0, 0, 9].as_slice(),
            ]
            .as_slice(),
        );
        assert_eq!(r, Err(TryIntoPuzzleError::BadGridSize));
    }

    #[test]
    fn test_parse_puzzle() {
        let s = concat!(
            "000780500\n",
            "200650700\n",
            "000000630\n",
            "010000070\n",
            "000506000\n",
            "060000020\n",
            "087000000\n",
            "003017009\n",
            "004092000\n",
        );
        let puzzle = Puzzle([
            [0, 0, 0, 7, 8, 0, 5, 0, 0],
            [2, 0, 0, 6, 5, 0, 7, 0, 0],
            [0, 0, 0, 0, 0, 0, 6, 3, 0],
            [0, 1, 0, 0, 0, 0, 0, 7, 0],
            [0, 0, 0, 5, 0, 6, 0, 0, 0],
            [0, 6, 0, 0, 0, 0, 0, 2, 0],
            [0, 8, 7, 0, 0, 0, 0, 0, 0],
            [0, 0, 3, 0, 1, 7, 0, 0, 9],
            [0, 0, 4, 0, 9, 2, 0, 0, 0],
        ]);
        assert_eq!(s.parse::<Puzzle>().unwrap(), puzzle);
    }

    #[test]
    fn test_parse_spaced_puzzle() {
        let s = concat!(
            "0 0 0  7 8 0  5 0 0\n",
            "2 0 0  6 5 0  7 0 0\n",
            "0 0 0  0 0 0  6 3 0\n",
            "\n",
            "0 1 0  0 0 0  0 7 0\n",
            "0 0 0  5 0 6  0 0 0\n",
            "0 6 0  0 0 0  0 2 0\n",
            "\n",
            "0 8 7  0 0 0  0 0 0\n",
            "0 0 3  0 1 7  0 0 9\n",
            "0 0 4  0 9 2  0 0 0\n",
        );
        let puzzle = Puzzle([
            [0, 0, 0, 7, 8, 0, 5, 0, 0],
            [2, 0, 0, 6, 5, 0, 7, 0, 0],
            [0, 0, 0, 0, 0, 0, 6, 3, 0],
            [0, 1, 0, 0, 0, 0, 0, 7, 0],
            [0, 0, 0, 5, 0, 6, 0, 0, 0],
            [0, 6, 0, 0, 0, 0, 0, 2, 0],
            [0, 8, 7, 0, 0, 0, 0, 0, 0],
            [0, 0, 3, 0, 1, 7, 0, 0, 9],
            [0, 0, 4, 0, 9, 2, 0, 0, 0],
        ]);
        assert_eq!(s.parse::<Puzzle>().unwrap(), puzzle);
    }

    #[test]
    fn test_parse_punctuated_puzzle() {
        let s = concat!(
            "...78.5..\n",
            "2..65.7..\n",
            "......63.\n",
            ".1.....7.\n",
            "...5.6...\n",
            ".6.....2.\n",
            ".87......\n",
            "..3.17..9\n",
            "..4.92...\n",
        );
        let puzzle = Puzzle([
            [0, 0, 0, 7, 8, 0, 5, 0, 0],
            [2, 0, 0, 6, 5, 0, 7, 0, 0],
            [0, 0, 0, 0, 0, 0, 6, 3, 0],
            [0, 1, 0, 0, 0, 0, 0, 7, 0],
            [0, 0, 0, 5, 0, 6, 0, 0, 0],
            [0, 6, 0, 0, 0, 0, 0, 2, 0],
            [0, 8, 7, 0, 0, 0, 0, 0, 0],
            [0, 0, 3, 0, 1, 7, 0, 0, 9],
            [0, 0, 4, 0, 9, 2, 0, 0, 0],
        ]);
        assert_eq!(s.parse::<Puzzle>().unwrap(), puzzle);
    }

    #[test]
    fn test_index_puzzle() {
        let puzzle = Puzzle([
            [0, 0, 3, 0, 2, 0, 6, 0, 0],
            [9, 0, 0, 3, 0, 5, 0, 0, 1],
            [0, 0, 1, 8, 0, 6, 4, 0, 0],
            [0, 0, 8, 1, 0, 2, 9, 0, 0],
            [7, 0, 0, 0, 0, 0, 0, 0, 8],
            [0, 0, 6, 7, 0, 8, 2, 0, 0],
            [0, 0, 2, 6, 0, 9, 5, 0, 0],
            [8, 0, 0, 2, 0, 3, 0, 0, 9],
            [0, 0, 5, 0, 1, 0, 3, 0, 0],
        ]);
        assert_eq!(puzzle[0], [0, 0, 3, 0, 2, 0, 6, 0, 0]);
        assert_eq!(puzzle[0][2], 3);
        assert_eq!(puzzle[8], [0, 0, 5, 0, 1, 0, 3, 0, 0]);
    }

    #[test]
    fn test_index_solution() {
        let solution = Solution([
            [4, 8, 3, 9, 2, 1, 6, 5, 7],
            [9, 6, 7, 3, 4, 5, 8, 2, 1],
            [2, 5, 1, 8, 7, 6, 4, 9, 3],
            [5, 4, 8, 1, 3, 2, 9, 7, 6],
            [7, 2, 9, 5, 6, 4, 1, 3, 8],
            [1, 3, 6, 7, 9, 8, 2, 4, 5],
            [3, 7, 2, 6, 8, 9, 5, 1, 4],
            [8, 1, 4, 2, 5, 3, 7, 6, 9],
            [6, 9, 5, 4, 1, 7, 3, 8, 2],
        ]);
        assert_eq!(solution[0], [4, 8, 3, 9, 2, 1, 6, 5, 7]);
        assert_eq!(solution[0][1], 8);
        assert_eq!(solution[8], [6, 9, 5, 4, 1, 7, 3, 8, 2]);
    }

    #[test]
    fn test_from_puzzle() {
        let grid = [
            [4, 8, 3, 9, 2, 1, 6, 5, 7],
            [9, 6, 7, 3, 4, 5, 8, 2, 1],
            [2, 5, 1, 8, 7, 6, 4, 9, 3],
            [5, 4, 8, 1, 3, 2, 9, 7, 6],
            [7, 2, 9, 5, 6, 4, 1, 3, 8],
            [1, 3, 6, 7, 9, 8, 2, 4, 5],
            [3, 7, 2, 6, 8, 9, 5, 1, 4],
            [8, 1, 4, 2, 5, 3, 7, 6, 9],
            [6, 9, 5, 4, 1, 7, 3, 8, 2],
        ];
        let solution = Solution(grid);
        assert_eq!(<[[u8; 9]; 9]>::from(solution), grid);
    }
}
