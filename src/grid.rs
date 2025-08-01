use strum::IntoEnumIterator;

pub use crate::direction::Direction;
use crate::direction::DirectionIter;
pub use crate::vec2::Vec2;

/// A grid of tiles.
/// The index of the top-left tile is (0, 0) and the tiles stored in row-major order.
#[derive(Clone)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T> Grid<T> {
    pub fn with_size<S: Clone>(rows: usize, cols: usize, init: S) -> Grid<S> {
        Grid {
            rows,
            cols,
            data: vec![init; rows * cols],
        }
    }

    pub fn from_data(rows: usize, cols: usize, data: Vec<T>) -> Grid<T> {
        assert_eq!(rows * cols, data.len());
        Grid { rows, cols, data }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn contains_coord(&self, coord: Vec2) -> bool {
        if coord.x < 0 || coord.y < 0 {
            return false;
        }
        coord.x >= 0
            && (coord.x as usize) < self.cols
            && coord.y >= 0
            && (coord.y as usize) < self.rows
    }
    
    pub fn normalized_coord(&self, coord: Vec2) -> Vec2 {
        Vec2::new(
            coord.x.rem_euclid(self.cols as i32),
            coord.y.rem_euclid(self.rows as i32),
        )
    }

    pub fn get(&self, coord: Vec2) -> Option<&T> {
        if self.contains_coord(coord) {
            Some(&self.data[self.linear_index(coord)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, coord: Vec2) -> Option<&mut T> {
        if self.contains_coord(coord) {
            let index = self.linear_index(coord);
            Some(&mut self.data[index])
        } else {
            None
        }
    }

    pub fn wrapping_get(&self, coord: Vec2) -> &T {
        if let Some(value) = self.get(coord) {
            value
        } else {
            let x = (coord.x as isize).rem_euclid(self.cols as isize);
            let y = (coord.y as isize).rem_euclid(self.rows as isize);
            &self.data[self.linear_index(Vec2::new(x as i32, y as i32))]
        }
    }

    pub fn indices_iter(&self) -> IndicesIter {
        IndicesIter {
            index: Vec2::default(),
            rows: self.rows(),
            cols: self.cols(),
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.data.iter(),
            indices_iter: self.indices_iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        let indices_iter = self.indices_iter();
        let iter = self.data.iter_mut();
        IterMut { iter, indices_iter }
    }

    pub fn indexed_iter(&self) -> IndexedIter<T> {
        IndexedIter {
            iter: self.data.iter(),
            indices_iter: self.indices_iter(),
        }
    }

    pub fn indexed_iter_mut(&mut self) -> IndexedIterMut<T> {
        let indices_iter = self.indices_iter();
        let iter = self.data.iter_mut();
        IndexedIterMut { iter, indices_iter }
    }

    // Read-only indexed neighbors iterator
    pub fn neighbors(&self, coord: Vec2) -> NeighborsIter<T> {
        NeighborsIter {
            grid: self,
            center: coord,
            direction: Direction::iter(),
        }
    }

    /// Panics if `coord` is not on the  grid, i.e. self.contains_coord(coord) returns false.
    #[doc(hidden)]
    fn linear_index(&self, coord: Vec2) -> usize {
        if !self.contains_coord(coord) {
            panic!("Grid::index() called for a coordinate not on the grid");
        }
        coord.y as usize * self.cols + coord.x as usize
    }
}

impl<T> std::ops::Index<Vec2> for Grid<T> {
    type Output = T;

    /// Panics if index is out of bounds
    fn index(&self, index: Vec2) -> &Self::Output {
        self.get(index)
            .expect("index must be inside the grid's bounds")
    }
}

impl<T> std::ops::IndexMut<Vec2> for Grid<T> {
    /// Panics if index is out of bounds
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        self.get_mut(index)
            .expect("index must be inside the grid's bounds")
    }
}

pub struct IndicesIter {
    index: Vec2,
    rows: usize,
    cols: usize,
}

impl Iterator for IndicesIter {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        let index = if 0 <= self.index.x
            && (self.index.x as usize) < self.cols
            && 0 <= self.index.y
            && (self.index.y as usize) < self.rows
        {
            Some(self.index)
        } else {
            None
        };

        self.index.x += 1;
        if self.index.x as usize >= self.cols {
            self.index.x = 0;
            self.index.y += 1;
        }

        index
    }
}

pub struct Iter<'a, T> {
    iter: std::slice::Iter<'a, T>,
    indices_iter: IndicesIter,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices_iter.next();
        self.iter.next()
    }
}

pub struct IterMut<'a, T> {
    iter: std::slice::IterMut<'a, T>,
    indices_iter: IndicesIter,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices_iter.next();
        self.iter.next()
    }
}

pub struct IndexedIter<'a, T> {
    iter: std::slice::Iter<'a, T>,
    indices_iter: IndicesIter,
}

impl<'a, T> Iterator for IndexedIter<'a, T> {
    type Item = (Vec2, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices_iter.next();
        let item = self.iter.next();
        Some((index?, item?))
    }
}

pub struct IndexedIterMut<'a, T> {
    iter: std::slice::IterMut<'a, T>,
    indices_iter: IndicesIter,
}

impl<'a, T> Iterator for IndexedIterMut<'a, T> {
    type Item = (Vec2, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices_iter.next();
        let item = self.iter.next();
        Some((index?, item?))
    }
}

pub struct NeighborsIter<'a, T> {
    grid: &'a Grid<T>,
    center: Vec2,
    direction: DirectionIter,
}

impl<'a, T> Iterator for NeighborsIter<'a, T> {
    type Item = (Vec2, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        for direction in self.direction.by_ref() {
            let n_pos = self.center + direction.to_vec2();
            if let Some(neighbor) = self.grid.get(n_pos) {
                return Some((n_pos, neighbor));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_functionality() {
        let grid = Grid::from_data(2, 2, vec![1, 2, 3, 4]);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 2);
        assert!(grid.contains_coord(Vec2 { x: 0, y: 0 }));
        assert!(!grid.contains_coord(Vec2 { x: 1, y: 2 }));
        assert_eq!(grid.get(Vec2::default()), Some(&1));

        let mut grid = Grid::<u32>::with_size(2, 2, 0);
        assert_eq!(grid.get(Vec2::default()), Some(&0));
        *grid.get_mut(Vec2 { x: 0, y: 0 }).unwrap() = 4;
        assert_eq!(grid.get(Vec2::default()), Some(&4));
    }

    #[test]
    #[should_panic]
    fn grid_from_data_invalid_length() {
        Grid::from_data(2, 2, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    #[should_panic]
    fn linear_index_out_of_bounds_overflow_x() {
        let grid = Grid::from_data(2, 2, vec![1, 2, 3, 4]);
        grid.linear_index(Vec2 { x: 2, y: 0 });
    }

    #[test]
    #[should_panic]
    fn linear_index_out_of_bounds_overflow_y() {
        let grid = Grid::from_data(2, 2, vec![1, 2, 3, 4]);
        grid.linear_index(Vec2 { x: 0, y: 2 });
    }

    #[test]
    #[should_panic]
    fn linear_index_out_of_bounds_underflow_x() {
        let grid = Grid::from_data(2, 2, vec![1, 2, 3, 4]);
        grid.linear_index(Vec2 { x: -1, y: 0 });
    }

    #[test]
    fn index_operator() {
        let mut grid = Grid::from_data(2, 2, vec![1, 2, 3, 4]);
        assert_eq!(grid[(0, 0).into()], 1);
        assert_eq!(grid[(1, 0).into()], 2);
        assert_eq!(grid[(0, 1).into()], 3);
        assert_eq!(grid[(1, 1).into()], 4);

        grid[(0, 0).into()] = 5;
        assert_eq!(grid[(0, 0).into()], 5);
    }

    #[test]
    #[should_panic]
    fn index_operator_out_of_bounds() {
        let mut grid = Grid::from_data(2, 2, vec![1, 2, 3, 4]);
        grid[(0, 2).into()] = 5;
    }

    #[test]
    fn iterator() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            data: vec![0, 1, 2, 3],
        };
        let mut it = grid.iter();
        assert_eq!(it.next(), Some(&0));
        assert_eq!(it.next(), Some(&1));
        assert_eq!(it.next(), Some(&2));
        assert_eq!(it.next(), Some(&3));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mutable_iterator() {
        let mut grid = Grid {
            rows: 2,
            cols: 2,
            data: vec![0, 1, 2, 3],
        };
        for tile in grid.iter_mut() {
            *tile *= 2;
        }
        assert_eq!(grid.data, vec![0, 2, 4, 6]);
    }

    #[test]
    fn indexed_iterator() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            data: vec![0, 1, 2, 3],
        };
        let mut it = grid.indexed_iter();
        assert_eq!(it.next(), Some((Vec2::new(0, 0), &0)));
        assert_eq!(it.next(), Some((Vec2::new(1, 0), &1)));
        assert_eq!(it.next(), Some((Vec2::new(0, 1), &2)));
        assert_eq!(it.next(), Some((Vec2::new(1, 1), &3)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mutable_indexed_iterator() {
        let mut grid = Grid {
            rows: 2,
            cols: 2,
            data: vec![0, 1, 2, 3],
        };
        for (_index, tile) in grid.indexed_iter_mut() {
            *tile = 8;
        }
        assert_eq!(grid.data, vec![8, 8, 8, 8]);
    }

    #[test]
    fn neighbors_iterator() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            data: vec![0, 1, 2, 3],
        };
        let mut it = grid.neighbors(Vec2 { x: 0, y: 0 });
        // Note this depends on the iteration order of Direction.
        assert_eq!(it.next(), Some((Vec2::new(1, 0), &1))); // right
        assert_eq!(it.next(), Some((Vec2::new(0, 1), &2))); // down
        assert_eq!(it.next(), None);
    }
}
