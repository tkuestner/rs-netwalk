use std::collections::{HashMap, HashSet};

use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use strum::IntoEnumIterator;

use crate::direction::Direction;
use crate::grid::{Grid, Vec2};
use crate::puzzle::links::Links;

use super::{Difficulty, Feature, Kind, Options, Alignment, Puzzle, Tile, Wall};


/// A builder capable of creating a random puzzle.
///
/// Use `with_options` to supply options, e.g., the size of the game board.
#[derive(Default)]
pub struct Builder {
    options: Options,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            options: Default::default(),
        }
    }

    /// Supply options to the builder.
    ///
    /// # Panics
    /// This function panics if the option `board_size` is smaller than 3 or larger than 20.
    pub fn with_options(mut self, options: Options) -> Self {
        if options.board_size < 3 {
            panic!("board size must be at least 3");
        }
        if options.board_size > 20 {
            panic!("board size must not be greater than 20");
        }

        self.options = options;
        self
    }

    /// Create a new puzzle.
    pub fn build(&self) -> Puzzle {
        // Place the source in the center
        let center = self.options.board_size / 2;
        let source = Vec2::splat(center as i32);
        let links = self.create_grid_of_links(source);

        // Transform the grid of links into a grid of tiles
        let mut tiles = Grid::<Tile>::from_data(
            self.options.board_size as usize,
            self.options.board_size as usize,
            links.iter()
                .map(|links| Tile::from_links(*links))
                .collect::<Vec<_>>(),
        );
        tiles[source].feature = Feature::Source;

        let walls = self.create_walls(&tiles, 0.06, 0.2);

        let expected_moves = self.rotate_tiles(&mut tiles, 0.8, 0.1);

        let mut puzzle = Puzzle {
            options: self.options,
            tiles,
            walls,
            source,
            expected_moves,
        };

        puzzle.calc_energy();
        puzzle
    }

    ///Create the underlying spanning tree of the grid graph.
    ///
    /// The algorithm starts with a source in the center and chooses an already visited tile at
    /// random to extend the tree to a random unvisited tile.
    fn create_grid_of_links(&self, source: Vec2) -> Grid<Links> {
        let size = self.options.board_size as usize;
        let mut proto_tiles = Grid::<Tile>::with_size(size, size, Links::default());

        let mut visited = Grid::<bool>::with_size(size, size, false);
        visited[source] = true;

        // The set of boundary nodes.
        let mut boundary = HashSet::from([source]);

        #[derive(Copy, Clone, Debug)]
        struct Connection {
            parent: Vec2,
            child: Vec2,
            direction: Direction,
        }

        loop {
            // Check all proto-tiles on the boundary. What type of tile would be created when this
            // proto-tile is connected to a neighboring unvisited proto-tile.

            let mut new_boundary = HashSet::new();
            let mut connections = vec![];

            for parent in boundary.iter().copied() {
                for direction in Direction::iter() {
                    let mut child = parent + direction.to_vec2();
                    if self.options.wrapping {
                        child = proto_tiles.normalized_coord(child);
                    }
                    if proto_tiles.contains_coord(child) && !visited[child] {
                        connections.push(Connection {
                            parent,
                            child,
                            direction,
                        });
                        new_boundary.insert(parent);
                    }
                }
            }

            if connections.is_empty() {
                break;
            }

            let weighted_connections: Vec<_> = connections.iter().map(|connection| {
                proto_tiles[connection.parent][connection.direction] = true;
                let kind = Tile::from_links(proto_tiles[connection.parent]).kind();
                proto_tiles[connection.parent][connection.direction] = false;

                (connection, difficulties()[&self.options.difficulty][&kind])
            }).collect();

            let connection = weighted_choice(&weighted_connections);

            new_boundary.insert(connection.child);
            visited[connection.child] = true;

            proto_tiles[connection.parent][connection.direction] = true;
            proto_tiles[connection.child][-connection.direction] = true;
            boundary = new_boundary;
        }

        proto_tiles
    }

    ///Randomly place some walls
    ///
    /// Must be called on the solved grid of tiles (i.e. before the tiles are rotated) because the
    /// function places walls only at positions where there are no connections between tiles in
    /// the solution.
    ///
    /// The actual number of walls is drawn from a normal distribution with parameters `mean`
    /// (percentage of total number of possible walls) and `std_dev` (standard deviation).
    fn create_walls(&self, tiles: &Grid<Tile>, mean_percent: f32, std_dev: f32) -> Vec<Wall> {
        let mut walls = vec![];
        for index in tiles.indices_iter() {
            // Top of tile
            if (self.options.wrapping || index.y != 0) && !tiles[index].has_link(Direction::Up) {
                walls.push(Wall { position: index, alignment: Alignment::Horizontal })
            }
            // Left of tile
            if (self.options.wrapping || index.x != 0) && !tiles[index].has_link(Direction::Left) {
                walls.push(Wall { position: index, alignment: Alignment::Vertical })
            }
        }
        let mean = mean_percent * walls.len() as f32;
        let normal = Normal::new(mean, std_dev * mean).unwrap();
        let count = normal
            .sample(&mut rand::rng())
            .clamp(0.0, walls.len() as f32) as usize;
        walls
            .choose_multiple(&mut rand::rng(), count)
            .copied()
            .collect()
    }

    /// Randomly rotate some tiles.
    ///
    /// Must be called on the solved grid of tiles in order to jumble the puzzle.
    fn rotate_tiles(&self, tiles: &mut Grid<Tile>, mean_percent: f32, std_dev: f32) -> u32 {
        let indices_rotatable_tiles = tiles.indexed_iter().filter_map(|(index, tile)| {
            match tile.kind {
                Kind::CrossIntersection => None,
                _ => Some(index),
            }
        }).collect::<Vec<_>>();

        let mean = mean_percent * indices_rotatable_tiles.len() as f32;
        let normal = Normal::new(mean, std_dev * mean).unwrap();
        let count = normal
            .sample(&mut rand::rng())
            .clamp(0.0, indices_rotatable_tiles.len() as f32) as usize;
        let mut rng = rand::rng();
        let rotate_indices = indices_rotatable_tiles
            .choose_multiple(&mut rng, count)
            .copied()
            .collect::<Vec<_>>();
        let expected_moves = rotate_indices.len();

        // Apply
        for index in rotate_indices {
            let tile = tiles.get_mut(index).unwrap();
            if tile.kind == Kind::Straight {
                tile.rotate();
            } else {
                let rotation_count = rng.random_range(1..4);
                for _ in 0..rotation_count {
                    tile.rotate();
                }
            }
        }

        expected_moves as u32
    }
}

fn difficulties() -> HashMap<Difficulty, HashMap<Kind, u32>> {
    let easy = HashMap::from([
        (Kind::CrossIntersection, 1),
        (Kind::TIntersection, 1),
        (Kind::Corner, 4),
        (Kind::Straight, 3),
        (Kind::DeadEnd, 1),
    ]);
    let medium = HashMap::from([
        (Kind::CrossIntersection, 0),
        (Kind::TIntersection, 1),
        (Kind::Corner, 5),
        (Kind::Straight, 2),
        (Kind::DeadEnd, 1),
    ]);
    let hard = HashMap::from([
        (Kind::CrossIntersection, 0),
        (Kind::TIntersection, 2),
        (Kind::Corner, 5),
        (Kind::Straight, 0),
        (Kind::DeadEnd, 1),
    ]);
    HashMap::from([
        (Difficulty::Easy, easy),
        (Difficulty::Medium, medium),
        (Difficulty::Hard, hard),
    ])
}

fn weighted_choice<T>(slice: &[(T, u32)]) -> &T {
    let mut rng = rand::rng();

    // Special case: if all weights are zero, rand::choose_weighted cannot be used.
    if slice.iter().all(|&(_, weight)| weight == 0) {
        &slice
            .choose(&mut rng)
            .expect("slice must not be empty")
            .0
    } else {
        &slice
            .choose_weighted(&mut rng, |s| s.1)
            .expect("correct weights")
            .0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn build_options_board_size_below_min() {
        let options = Options {
            board_size: 2,
            difficulty: Difficulty::Easy,
            wrapping: false,
        };
        let _builder = Builder::default().with_options(options);
    }

    #[test]
    #[should_panic]
    fn build_options_board_size_above_max() {
        let options = Options {
            board_size: 21,
            difficulty: Difficulty::Hard,
            wrapping: true,
        };
        let _builder = Builder::default().with_options(options);
    }

    #[test]
    fn build_random_puzzle() {
        let options = Options {
            board_size: 3,
            difficulty: Difficulty::Easy,
            wrapping: false,
        };
        let builder = Builder::default().with_options(options);
        let puzzle = builder.build();
        assert_eq!(*puzzle.options(), options);
    }
}
