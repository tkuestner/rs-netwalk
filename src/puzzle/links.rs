use crate::grid::Direction;
use crate::puzzle::{Kind, Orientation};

/// The prototype of a game tile.
///
/// It consists of four links from its center to its four edges. The links can be active or inactive.
/// It does not know its form (I, L, T, etc.) and rotation.
#[derive(Copy, Clone, Default)]
pub struct Links {
    links: [bool; 4],
}

impl std::ops::Index<Direction> for Links {
    type Output = bool;

    fn index(&self, index: Direction) -> &Self::Output {
        &self.links[index as usize]
    }
}

impl std::ops::IndexMut<Direction> for Links {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self.links[index as usize]
    }
}

impl From<Links> for (Kind, Orientation) {
    fn from(links: Links) -> Self {
        match links.links {
            [true, false, false, false] => (Kind::DeadEnd, Orientation::Basic),
            [false, true, false, false] => (Kind::DeadEnd, Orientation::Ccw90),
            [false, false, true, false] => (Kind::DeadEnd, Orientation::Ccw180),
            [false, false, false, true] => (Kind::DeadEnd, Orientation::Ccw270),
            [true, false, true, false] => (Kind::Straight, Orientation::Basic),
            [false, true, false, true] => (Kind::Straight, Orientation::Ccw90),
            [true, true, false, false] => (Kind::Corner, Orientation::Basic),
            [false, true, true, false] => (Kind::Corner, Orientation::Ccw90),
            [false, false, true, true] => (Kind::Corner, Orientation::Ccw180),
            [true, false, false, true] => (Kind::Corner, Orientation::Ccw270),
            [true, true, true, false] => (Kind::TIntersection, Orientation::Basic),
            [false, true, true, true] => (Kind::TIntersection, Orientation::Ccw90),
            [true, false, true, true] => (Kind::TIntersection, Orientation::Ccw180),
            [true, true, false, true] => (Kind::TIntersection, Orientation::Ccw270),
            [true, true, true, true] => (Kind::CrossIntersection, Orientation::Basic),
            _ => unreachable!("encountered an empty tile with no links"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_tile() {
        // An L-tile rotated 90° counter-clockwise
        let mut links = Links::default();
        links[Direction::Up] = true;
        links[Direction::Left] = true;
        let (kind, orientation) = links.into();
        assert_eq!(kind, Kind::Corner);
        assert_eq!(orientation, Orientation::Ccw90);
    }

    #[test]
    #[should_panic(expected = "encountered an empty tile with no links")]
    fn empty_tile() {
        let links = Links::default();
        let _: (Kind, Orientation) = links.into();
    }
}