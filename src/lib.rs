use std::fmt;

static DIVIDER: &str = "+-----+-----+-----+";

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Puzzle([[u8; 9]; 9]); // Unfilled cells are represented by 0

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
        // TODO: Guard against underflow?
        self.0[usize::from(number) - 1] -= 1;
    }

    fn for_number(&self, number: u8) -> u8 {
        self.0[usize::from(number) - 1]
    }

    fn is_full(&self) -> bool {
        self.0.iter().all(|&x| x == 3)
    }
}

struct InProgress {
    puzzle: [[u8; 9]; 9],
    obstructions: [[Option<Obstruction>; 9]; 9],
    // None = cell in input puzzle was already filled
}

impl InProgress {
    fn new(p: &Puzzle) -> Self {
        Self {
            obstructions: [[Some(Obstruction::new()); 9]; 9],
            puzzle: p.0,
        }
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

impl Puzzle {
    pub fn solve(&self) -> Option<Solution> {
        let mut scratch = InProgress::new(self);
        for i in 0..9 {
            for j in 0..9 {
                if scratch.puzzle[i][j] != 0 {
                    scratch.obstructions[i][j] = None;
                    scratch.add_obstruction(i, j);
                }
            }
        }
        let mut i = 0;
        while i < 9 {
            let mut j = 0;
            while j < 9 {
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
                        loop {
                            j = match j.checked_sub(1) {
                                Some(j2) => j2,
                                None => {
                                    // This is where we return None if there's
                                    // no solution:
                                    i = i.checked_sub(1)?;
                                    8
                                }
                            };
                            if let Some(o2) = scratch.obstructions[i][j] {
                                if !o2.is_full() {
                                    j = match j.checked_sub(1) {
                                        Some(j2) => j2,
                                        None => {
                                            i -= 1;
                                            8
                                        }
                                    };
                                    break;
                                }
                                scratch.remove_obstruction(i, j);
                                scratch.puzzle[i][j] = 0;
                            }
                        }
                    }
                }
                j += 1;
            }
            i += 1;
        }
        Some(Solution(scratch.puzzle))
    }
}

// TODO: Give Puzzle a TryFrom<(some sort of 2D array)> impl
// TODO: Give Puzzle a FromStr impl that ignores horizontal whitespace and treats 0's and nondigits as unfilled cells
// TODO: Give Puzzle something for accessing individual cells?

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

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Solution([[u8; 9]; 9]);

// TODO: Give Solution something for accessing individual cells
// TODO: impl From<Solution> for [[u8; 9]; 9] ?

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
}
