use strum::EnumIter;
use thiserror::Error;

use crate::vec2::Vec2;

/// The four cardinal directions.
///
/// They are ordered counter-clockwise starting from the x-axis. They can also be converted to
/// integers, e.g. to be used as indices.
#[derive(Copy, Clone, Debug, EnumIter, Eq, PartialEq)]
pub enum Direction {
    Right = 0,
    Up = 1,
    Left = 2,
    Down = 3,
}

impl Direction {
    pub(crate) fn to_vec2(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0, -1),
            Direction::Down => Vec2::new(0, 1),
            Direction::Left => Vec2::new(-1, 0),
            Direction::Right => Vec2::new(1, 0),
        }
    }
}

impl TryFrom<Vec2> for Direction {
    type Error = DirectionError;

    fn try_from(value: Vec2) -> Result<Self, Self::Error> {
        match value {
            Vec2 { x: 0, y: -1 } => Ok(Direction::Up),
            Vec2 { x: 0, y: 1 } => Ok(Direction::Down),
            Vec2 { x: -1, y: 0 } => Ok(Direction::Left),
            Vec2 { x: 1, y: 0 } => Ok(Direction::Right),
            _ => Err(DirectionError::InvalidDirection(value)),
        }
    }
}

impl std::ops::Neg for Direction {
    type Output = Direction;
    fn neg(self) -> Self::Output {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Copy, Clone, Debug, Error, Eq, PartialEq)]
pub enum DirectionError {
    #[error("invalid direction from '{0:?}'")]
    InvalidDirection(Vec2),
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn iteration() {
        let mut iter = Direction::iter();
        assert_eq!(iter.next(), Some(Direction::Right));
        assert_eq!(iter.next(), Some(Direction::Up));
        assert_eq!(iter.next(), Some(Direction::Left));
        assert_eq!(iter.next(), Some(Direction::Down));
    }

    #[test]
    fn conversion() {
        let v = Vec2::new(0, -1);
        assert_eq!(Direction::try_from(v), Ok(Direction::Up));
        assert_eq!(Direction::Up.to_vec2(), v);

        let v = Vec2::new(0, 1);
        assert_eq!(Direction::try_from(v), Ok(Direction::Down));
        assert_eq!(Direction::Down.to_vec2(), v);

        let v = Vec2::new(-1, 0);
        assert_eq!(Direction::try_from(v), Ok(Direction::Left));
        assert_eq!(Direction::Left.to_vec2(), v);

        let v = Vec2::new(1, 0);
        assert_eq!(Direction::try_from(v), Ok(Direction::Right));
        assert_eq!(Direction::Right.to_vec2(), v);

        let v = Vec2::new(-1, 1);
        assert_eq!(
            Direction::try_from(v),
            Err(DirectionError::InvalidDirection(v))
        );
    }

    #[test]
    fn opposite() {
        assert_eq!(-Direction::Up, Direction::Down);
        assert_eq!(-Direction::Down, Direction::Up);
        assert_eq!(-Direction::Left, Direction::Right);
        assert_eq!(-Direction::Right, Direction::Left);
    }
}
