/// 2d discrete vector for navigating on a grid of squares
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Vec2 { x, y }
    }

    pub fn splat(value: i32) -> Self {
        Vec2 { x: value, y: value }
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from(v: (i32, i32)) -> Self {
        Vec2 { x: v.0, y: v.1 }
    }
}

impl From<&(i32, i32)> for Vec2 {
    fn from(v: &(i32, i32)) -> Self {
        Vec2 { x: v.0, y: v.1 }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<i32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let v = Vec2::new(1, 2);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);

        let v = Vec2::splat(3);
        assert_eq!(v.x, 3);
        assert_eq!(v.y, 3);

        let v: Vec2 = (4, 5).into();
        assert_eq!(v.x, 4);
        assert_eq!(v.y, 5);

        let data = &(4, 5);
        let v = Vec2::from(data);
        assert_eq!(v.x, 4);
        assert_eq!(v.y, 5);
    }

    #[test]
    fn operators() {
        let a = Vec2::new(1, 2);
        let b = Vec2::new(3, 4);
        let c = a + b;
        assert_eq!(c.x, 4);
        assert_eq!(c.y, 6);

        let c = a - b;
        assert_eq!(c.x, -2);
        assert_eq!(c.y, -2);

        let c = a * 2;
        assert_eq!(c.x, 2);
        assert_eq!(c.y, 4);
    }
}
