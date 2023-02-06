use std::fmt;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Puzzle([[u8; 9]; 9]);

/// Counts the amount of cells of each numeric value that "obstruct" a given
/// cell
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
        // If f.alternate(), draw borders and column gaps and leave unfilled
        // cells as whitespace.
        // Otherwise, draw unfilled cells as 0
        todo!()
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Solution([[u8; 9]; 9]);

// TODO: Give Solution something for accessing individual cells
// TODO: impl From<Solution> for [[u8; 9]; 9] ?

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // If f.alternate(), draw borders and column gaps
        todo!()
    }
}
