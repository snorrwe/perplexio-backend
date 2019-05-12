use std::ops;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, GraphQLObject)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    pub fn new(x: i32, y: i32) -> Vector {
        Vector { x: x, y: y }
    }

    pub fn normal(&self) -> Vector {
        let x = if self.x != 0 { self.x.abs() } else { 1 };
        let y = if self.y != 0 { self.y.abs() } else { 1 };
        Vector {
            x: self.x / x,
            y: self.y / y,
        }
    }
}

impl ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Mul<i32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: i32) -> Vector {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub fn segments_intersecting(
    start_a: Vector,
    end_a: Vector,
    start_b: Vector,
    end_b: Vector,
) -> bool {
    let a = end_a - start_a;
    let b = start_b - end_b;

    assert!(a.x != 0 || a.y != 0);
    assert!(b.x != 0 || b.y != 0);

    let d = start_b - start_a;

    let det = a.x * b.y - a.y * b.x;

    if det == 0 {
        return a.x * d.y - a.y * d.x == 0;
    }

    let r = (d.x * b.y - d.y * b.x) as f32 / det as f32;
    let s = (a.x * d.y - a.y * d.x) as f32 / det as f32;

    !(r < 0. || r > 1. || s < 0. || s > 1.)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let res = Vector { x: 0, y: 2 } + Vector { x: 4, y: -3 };

        assert_eq!(res.x, 4);
        assert_eq!(res.y, -1);
    }

    #[test]
    fn test_sub() {
        let res = Vector { x: 0, y: 2 } - Vector { x: 4, y: -3 };

        assert_eq!(res.x, -4);
        assert_eq!(res.y, 5);
    }

    #[test]
    fn test_simple_intersection() {
        let start_a = Vector { x: 2, y: 3 };
        let end_a = Vector { x: 4, y: 1 };

        let start_b = Vector { x: 0, y: 1 };
        let end_b = Vector { x: 3, y: 3 };

        assert!(segments_intersecting(start_a, end_a, start_b, end_b));
    }

    #[test]
    fn test_intersects_with_itself() {
        let start_a = Vector { x: 2, y: 3 };
        let end_a = Vector { x: 3, y: 2 };

        let start_b = Vector { x: 3, y: 2 };
        let end_b = Vector { x: 2, y: 3 };

        assert!(segments_intersecting(start_a, end_a, start_b, end_b));
    }

    #[test]
    fn test_simple_non_intersection() {
        let start_a = Vector { x: 2, y: 3 };
        let end_a = Vector { x: 4, y: 1 };

        let start_b = Vector { x: 2, y: 0 };
        let end_b = Vector { x: 6, y: 1 };

        assert!(!segments_intersecting(start_a, end_a, start_b, end_b));
    }

    #[test]
    fn test_endpoints_touching_considered_intersecting() {
        let start_a = Vector { x: -2, y: 3 };
        let end_a = Vector { x: 2, y: 0 };

        let start_b = Vector { x: 2, y: 0 };
        let end_b = Vector { x: 6, y: 1 };

        assert!(segments_intersecting(start_a, end_a, start_b, end_b));
    }
}
