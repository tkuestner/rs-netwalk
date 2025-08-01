mod builder;
mod links;

use std::cmp::PartialEq;

use strum::IntoEnumIterator;

pub use builder::Builder;
use crate::grid::{Direction, Grid, Vec2};
use crate::puzzle::links::{Links};

/// The puzzle, consisting of a grid of rotatable tiles, a source, multiple drains, walls, etc.
#[derive(Clone)]
pub struct Puzzle {
    options: Options, // how the puzzle was generated
    tiles: Grid<Tile>,
    walls: Vec<Wall>,
    source: Vec2,  // the tile containing the source is also marked as such
    expected_moves: u32, // expected number of moves required to solve the puzzle
}

impl Puzzle {
    /// Return the options which were applied during puzzle generation.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Immutable access to the grid of tiles.
    pub fn grid(&self) -> &Grid<Tile> {
        &self.tiles
    }

    /// Mutable access to the grid of tiles.
    pub fn grid_mut(&mut self) -> &mut Grid<Tile> {
        &mut self.tiles
    }

    /// Return a list of walls.
    pub fn walls(&self) -> &[Wall] {
        &self.walls
    }

    /// Return the coordinates of the source tile.
    pub fn source(&self) -> &Vec2 {
        &self.source
    }

    /// Return the number of moves expected to solve the puzzle.
    ///
    /// A move is a manipulation of a single tile (one or more rotations).
    /// There might be more than one solution for the puzzle and other solutions may have
    /// fewer moves than the expected number.
    pub fn expected_moves(&self) -> u32 {
        self.expected_moves
    }

    /// Return true if the puzzle is solved. For the puzzle to be considered solved, all tiles
    /// must be powered, not just drains/dead-ends.
    pub fn solved(&self) -> bool {
        self.tiles.iter().all(|tile| tile.powered)
    }

    /// Return the number of rows or columns of tiles on the game board.
    pub fn size(&self) -> u8 {
        assert_eq!(self.tiles.rows(), self.tiles.cols());
        self.grid().rows() as u8
    }

    /// Immutably access the tile at `coord`.
    pub fn get_tile(&self, coord: Vec2) -> Option<&Tile> {
        self.tiles.get(coord)
    }

    /// Recalculate which tiles are connected to the source and thus receive energy.
    pub fn calc_energy(&mut self) {
        assert!(self.tiles.contains_coord(self.source));
        assert_eq!(self.tiles.get(self.source).unwrap().feature, Feature::Source);

        self.tiles.iter_mut().for_each(|tile| tile.powered = false);

        let mut work_stack = vec![self.source];
        while let Some(current) = work_stack.pop() {
            self.tiles[current].powered = true;

            for direction in Direction::iter() {
                let neighbor = current + direction.to_vec2();
                if !self.tiles.wrapping_get(neighbor).powered && self.connected(current, direction) {
                    work_stack.push(self.tiles.normalized_coord(neighbor));
                }
            }
        }
    }

    /// Helper function for `calc_energy`. Return true if two tiles (one at `coord` and the
    /// neighboring tile at `coord` + `dir`) have a connection (i.e. two links and no wall).
    #[doc(hidden)]
    fn connected(&self, coord: Vec2, dir: Direction) -> bool {
        // If the grid is not wrapping, check the invisible walls around the game board.
        if !self.options.wrapping && !self.tiles.contains_coord(coord + dir.to_vec2()) {
            return false;
        }

        if self.wall_between(coord, dir) {
            return false;
        }

        let tile_a = self.tiles.wrapping_get(coord);
        let tile_b = self.tiles.wrapping_get(coord + dir.to_vec2());

        tile_a.has_link(dir) && tile_b.has_link(-dir)
    }

    /// Helper function for `connected`. Return true if there is a wall between two tiles (one
    /// at `coord` and the neighboring tile `coord` + `dir`).
    #[doc(hidden)]
    fn wall_between(&self, coord: Vec2, dir: Direction) -> bool {
        let coord_a = coord;
        let coord_b = self.tiles.normalized_coord(coord + dir.to_vec2());

        let (pos, alignment) = match dir {
            Direction::Up => (coord_a, Alignment::Horizontal),
            Direction::Down => (coord_b, Alignment::Horizontal),
            Direction::Left => (coord_a, Alignment::Vertical),
            Direction::Right => (coord_b, Alignment::Vertical),
        };

        self.walls
            .iter()
            .any(|wall| wall.position == pos && wall.alignment == alignment)
    }
}

/// The game / puzzle options, e.g. difficulty and board size.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Options {
    /// The number of rows and columns of the game board.
    pub board_size: u8,
    /// The difficulty of the puzzle.
    pub difficulty: Difficulty,
    /// If true, the game board forms a torus, i.e. energy can flow from a tile on the left edge to
    /// a tile on the right edge, as well as from the top edge to the bottom edge.
    pub wrapping: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            board_size: 3,
            difficulty: Difficulty::Easy,
            wrapping: false,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, strum::Display)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

/// A tile on the game board.
///
/// Tiles contain pipes of certain shapes and can also contain an energy source or drain. Tiles
/// can be powered (if connected to an energy source) or unpowered. Tiles can be rotated which
/// changes the connection of the pipes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tile {
    kind: Kind,
    feature: Feature,
    orientation: Orientation,
    powered: bool,
}

impl Tile {
    pub fn from_links(links: Links) -> Self {
        let (kind, rotation): (Kind, Orientation) = links.into();
        let feature = match kind {
            Kind::DeadEnd => Feature::Drain,
            _ => Feature::None,
        };

        Tile {
            kind,
            feature,
            orientation: rotation,
            powered: false
        }
    }

    pub fn kind(&self) -> Kind { self.kind }

    pub fn feature(&self) -> Feature { self.feature }

    pub fn orientation(&self) -> Orientation { self.orientation }

    pub fn powered(&self) -> bool { self.powered }

    pub fn rotate(&mut self) { self.orientation = self.orientation.next_ccw(); }

    pub fn has_link(&self, direction: Direction) -> bool {
        let base_config = match self.kind {
            // east, north, west, south (right, up, left, down)
            Kind::DeadEnd => [true, false, false, false],
            Kind::Straight => [true, false, true, false],
            Kind::Corner => [true, true, false, false],
            Kind::TIntersection => [true, true, true, false],
            Kind::CrossIntersection => [true, true, true, true],
        };

        let index: i8 = match direction {
            Direction::Up => 1,
            Direction::Down => 3,
            Direction::Left => 2,
            Direction::Right => 0,
        };

        let offset: i8 = match self.orientation {
            Orientation::Basic => 0,
            Orientation::Ccw90 => 1,
            Orientation::Ccw180 => 2,
            Orientation::Ccw270 => 3,
        };

        base_config[(index - offset).rem_euclid(4) as usize]
    }
}

/// The shape of the pipes on a tile, e.g. I, L or T.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumIs)]
pub enum Kind {
    DeadEnd,
    Straight,
    Corner,
    TIntersection,
    CrossIntersection,
}

/// A source or a drain sitting (on top of) a tile.
///
/// Most tiles have only pipes and no feature on them. There is only a single source tile,
/// usually placed in the center of the game board. Tiles with a dead-end pipe are automatically
/// considered drains.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Feature {
    None,
    Drain,
    Source,
}

#[derive(Copy, Clone, Debug, PartialEq, strum::EnumIter)]
pub enum Orientation {
    Basic,   // fundamental, not rotated, facing right
    Ccw90,   // rotated 90° counter-clockwise, facing up
    Ccw180,  // rotated 180° counter-clockwise, facing left
    Ccw270,  // rotated 270° counter-clockwise, facing down
}

impl Orientation {
    /// Transform the orientation into an angle in radian.
    pub fn to_angle(&self) -> f32 {
        match self {
            Orientation::Basic => 0.,
            Orientation::Ccw90 => std::f32::consts::PI / 2.0,
            Orientation::Ccw180 => std::f32::consts::PI,
            Orientation::Ccw270 => std::f32::consts::PI + std::f32::consts::PI / 2.0,
        }
    }

    /// Get the next orientation in counter-clockwise order.
    pub fn next_ccw(&self) -> Self {
        match self {
            Orientation::Basic => Orientation::Ccw90,
            Orientation::Ccw90 => Orientation::Ccw180,
            Orientation::Ccw180 => Orientation::Ccw270,
            Orientation::Ccw270 => Orientation::Basic,
        }
    }
}

// TODO Maybe create special data structure to quickly lookup if there is a wall between two tiles.
// A horizontal wall is associated with the tile below it. In other words, a tile can have a wall along the top edge.
// A vertical wall is associated with the tile to the right of it. In other words, a tile can have a wall along the left edge.

/// A wall between two tiles.
///
/// A wall can be aligned horizontally or vertically.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Wall {
    position: Vec2,         // top or left tile
    alignment: Alignment, // maybe use two Vec2 instead
}

impl Wall {
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn orientation(&self) -> Alignment {
        self.alignment
    }
}

/// The horizontal or vertical alignment of a wall.
#[derive(Copy, Clone, Debug, Eq, PartialEq, strum::EnumIter, Hash)]
pub enum Alignment {
    Horizontal,
    Vertical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn example_puzzle() -> Puzzle {
        let options = Options {
            board_size: 3,
            difficulty: Difficulty::Easy,
            wrapping: false,
        };

        let mut grid = Grid::<Tile>::with_size(
            3,
            3,
            Tile {
                kind: Kind::DeadEnd,
                feature: Feature::None,
                orientation: Orientation::Basic,
                powered: false,
            },
        );
        *grid.get_mut((0, 2).into()).unwrap() = Tile {
            kind: Kind::DeadEnd,
            feature: Feature::Drain,
            orientation: Orientation::Ccw270,
            powered: false,
        };

        *grid.get_mut((1, 2).into()).unwrap() = Tile {
            kind: Kind::TIntersection,
            feature: Feature::None,
            orientation: Orientation::Basic,
            powered: false,
        };

        *grid.get_mut((2, 2).into()).unwrap() = Tile {
            kind: Kind::Corner,
            feature: Feature::None,
            orientation: Orientation::Ccw180,
            powered: false,
        };
        *grid.get_mut((0, 1).into()).unwrap() = Tile {
            kind: Kind::DeadEnd,
            feature: Feature::Drain,
            orientation: Orientation::Ccw270,
            powered: false,
        };
        *grid.get_mut((1, 1).into()).unwrap() = Tile {
            kind: Kind::TIntersection,
            feature: Feature::Source,
            orientation: Orientation::Ccw270,
            powered: false,
        };
        *grid.get_mut((2, 1).into()).unwrap() = Tile {
            kind: Kind::Straight,
            feature: Feature::None,
            orientation: Orientation::Basic,
            powered: false,
        };
        *grid.get_mut((0, 0).into()).unwrap() = Tile {
            kind: Kind::DeadEnd,
            feature: Feature::Drain,
            orientation: Orientation::Ccw270,
            powered: false,
        };
        *grid.get_mut((1, 0).into()).unwrap() = Tile {
            kind: Kind::Corner,
            feature: Feature::None,
            orientation: Orientation::Ccw90,
            powered: false,
        };
        *grid.get_mut((2, 0).into()).unwrap() = Tile {
            kind: Kind::DeadEnd,
            feature: Feature::Drain,
            orientation: Orientation::Basic,
            powered: false,
        };

        let walls = vec![
            Wall {
                position: (0, 2).into(),
                alignment: Alignment::Horizontal,
            },
            Wall {
                position: (2, 1).into(),
                alignment: Alignment::Vertical,
            },
            Wall {
                position: (1, 1).into(),
                alignment: Alignment::Vertical,
            },
        ];

        let source = Vec2::new(1, 1);
        let expected_moves = 8;

        let mut puzzle = Puzzle {
            options,
            tiles: grid,
            walls,
            source,
            expected_moves,
        };
        puzzle.calc_energy();
        puzzle
    }

    #[test]
    fn verify_example_puzzle() {
        let puzzle = example_puzzle();
        assert_eq!(puzzle.solved(), false);
        assert_eq!(puzzle.size(), 3);
        assert_eq!(puzzle.get_tile(Vec2::new(0, 0)), Some(&Tile {
            kind: Kind::DeadEnd,
            feature: Feature::Drain,
            orientation: Orientation::Ccw270,
            powered: false
        }));
    }
}