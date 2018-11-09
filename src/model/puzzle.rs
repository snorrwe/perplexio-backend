use super::vector::{segments_intersecting, Vector};
use rand::prelude::*;
use std::fmt;

/// Tables are matrices of characters
/// They are column major
/// so e.g.
/// ```
/// // Accessing element [1,5] where 1 is the column, 5 is the row number
/// let table = Table::new(10,10);
/// let x = table::at(1, 5);
/// ```
#[derive(Serialize)]
pub struct Puzzle {
    table: Vec<char>,
    columns: usize,
    rows: usize,
    solutions: Vec<Vec<usize>>,
}

#[derive(Serialize, Debug)]
pub enum PuzzleError {
    CantFit,
    InvalidArgument,
}

impl Puzzle {
    pub fn empty(col: usize, row: usize) -> Puzzle {
        let mut table = Vec::with_capacity(col * row);
        unsafe {
            table.set_len(col * row);
        }
        for chr in table.iter_mut() {
            *chr = '\0';
        }
        Puzzle {
            table: table,
            columns: col,
            rows: row,
            solutions: vec![],
        }
    }

    pub fn get_table(&self) -> &Vec<char> {
        &self.table
    }

    pub fn get_shape(&self) -> (usize, usize) {
        (self.columns, self.rows)
    }

    pub fn get_solutions(&self) -> &Vec<Vec<usize>> {
        &self.solutions
    }

    pub fn at<'a>(&'a self, col: usize, row: usize) -> &'a char {
        assert!(col < self.columns);
        assert!(row < self.rows);
        let index = self.index(col, row);
        &self.table[index]
    }

    pub fn at_mut<'a>(&'a mut self, col: usize, row: usize) -> &'a mut char {
        assert!(col < self.columns);
        assert!(row < self.rows);
        let index = self.index(col, row);
        &mut self.table[index]
    }

    pub fn set(&mut self, col: usize, row: usize, chr: char) {
        let index = self.index(col, row);
        self.table[index] = chr;
    }

    fn index(&self, col: usize, row: usize) -> usize {
        assert!(col < self.columns);
        assert!(row < self.rows);
        col + self.columns * row
    }

    pub fn from_words(words: Vec<String>, max_iterations: usize) -> Result<Puzzle, PuzzleError> {
        let mut result = Err(PuzzleError::InvalidArgument);
        'a: for i in 0..max_iterations {
            result = Self::_from_words(&words, 10 + i);
            if let Ok(r) = result {
                let (a, b) = if r.columns > r.rows {
                    (r.columns, r.rows)
                } else {
                    (r.rows, r.columns)
                };
                let almost_square = a - b <= 2;
                if almost_square {
                    return Ok(r);
                }
            }
            result = Err(PuzzleError::CantFit);
        }
        result
    }

    fn _from_words(words: &Vec<String>, max_iterations: usize) -> Result<Puzzle, PuzzleError> {
        assert!(words.len() > 0);
        let mut segments: Vec<(Vector, Vector)> = words
            .iter()
            .map(|w| Self::random_segment_by_word(w))
            .collect();
        let mut intersections = vec![];
        intersections.reserve(words.len() * 2);
        let mut rng = thread_rng();
        'a: for _ in 0..max_iterations {
            Self::intersections(&segments, &mut intersections);
            for intersection in intersections.iter() {
                let (i, j) = intersection;
                let a = segments[*i];
                let b = segments[*j];
                let step = if rng.gen() { 1 } else { -1 };
                segments[*i] = Self::move_forward(step, a);
                let step = if rng.gen() { 1 } else { -1 };
                segments[*j] = Self::move_forward(step, b);
            }
            if intersections.is_empty() {
                break 'a;
            }
        }
        if !intersections.is_empty() {
            return Err(PuzzleError::CantFit);
        }
        let (min, max) = Self::find_minmax(&segments);
        let cols = max.x - min.x + 1;
        let rows = max.y - min.y + 1;
        let dir = Vector::new(0, 0) - min;

        Self::translate_segments(dir, &mut segments);

        let mut result = Puzzle::empty(cols as usize, rows as usize);
        words
            .iter()
            .zip(segments.iter())
            .for_each(|(word, segment)| {
                let dir = segment.1 - segment.0;
                let dir = dir.normal();
                let mut current = segment.0;
                let mut solution = vec![];
                for chr in word.chars() {
                    let x = current.x as usize;
                    let y = current.y as usize;
                    result.set(x, y, chr);
                    solution.push(result.index(x, y));
                    current = current + dir;
                }
                result.solutions.push(solution);
            });
        Self::fill_nulls(&mut result);
        Ok(result)
    }

    fn find_minmax(segments: &Vec<(Vector, Vector)>) -> (Vector, Vector) {
        let mut min_x: i32 = 0;
        let mut min_y: i32 = 0;
        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;
        segments.iter().for_each(|segment| {
            if min_x > segment.0.x {
                min_x = segment.0.x;
            }
            if min_x > segment.1.x {
                min_x = segment.1.x;
            }
            if min_y > segment.0.y {
                min_y = segment.0.y;
            }
            if min_y > segment.1.y {
                min_y = segment.1.y;
            }
            if max_x < segment.0.x {
                max_x = segment.0.x;
            }
            if max_x < segment.1.x {
                max_x = segment.1.x;
            }
            if max_y < segment.0.y {
                max_y = segment.0.y;
            }
            if max_y < segment.1.y {
                max_y = segment.1.y;
            }
        });
        (Vector::new(min_x, min_y), Vector::new(max_x, max_y))
    }

    fn translate_segments(dir: Vector, segments: &mut Vec<(Vector, Vector)>) {
        for segment in segments.iter_mut() {
            *segment = (segment.0 + dir, segment.1 + dir);
        }
    }

    fn move_forward(steps: i32, segment: (Vector, Vector)) -> (Vector, Vector) {
        let dir = segment.1 - segment.0;
        (segment.0 + dir * steps, segment.1 + dir * steps)
    }

    /// Return the indeces of intersecting segments
    fn intersections(segments: &[(Vector, Vector)], res: &mut Vec<(usize, usize)>) {
        res.clear();
        for (i, s1) in segments.iter().enumerate() {
            for j in i + 1..segments.len() {
                let s2 = segments[j];
                if segments_intersecting(s1.0, s1.1, s2.0, s2.1) {
                    res.push((i, j));
                }
            }
        }
    }

    fn random_segment_by_word(word: &String) -> (Vector, Vector) {
        let dirs = [
            Vector::new(0, 1),
            Vector::new(0, -1),
            Vector::new(1, 0),
            Vector::new(1, 1),
            Vector::new(1, -1),
            Vector::new(-1, 0),
            Vector::new(-1, 1),
            Vector::new(-1, -1),
        ];
        let mut rng = thread_rng();
        let dir = dirs[rng.gen_range(0, dirs.len())];

        let start = Vector::new(rng.gen_range(0, 10), rng.gen_range(0, 10));
        let dir = dir * word.len() as i32;
        let end = start + dir;
        (start, end)
    }

    fn fill_nulls(&mut self) {
        let mut rng = thread_rng();
        for chr in self.table.iter_mut() {
            if *chr == '\0' {
                *chr = rng.gen_range(b'a', b'z') as char;
            }
        }
    }
}

impl fmt::Display for Puzzle {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Puzzle {}×{}\nSolutions:\n",
            self.columns, self.rows
        )?;
        for s in self.solutions.iter() {
            for c in s {
                write!(formatter, " {}", c);
            }
            write!(formatter, "\n");
        }
        write!(formatter, "\n");
        for r in 0..self.rows {
            for c in 0..self.columns {
                write!(formatter, "{} ", *self.at(c, r))?;
            }
            write!(formatter, "\n",)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fitting() {
        let words = vec![
            "cerial",
            "great",
            "frootloops",
            "almafa",
            "korte",
            "krumpli",
            "bejgli",
            "cerial",
            "great",
            "frootloops",
        ]
        .iter()
        .map(|w| w.to_string())
        .collect();
        let puzzle = Puzzle::from_words(words, 1000).expect("Failed to generate");
        println!("{}", puzzle);
    }
}
