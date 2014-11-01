use std::slice;
use num::Integer;
use std::num::FromPrimitive;
use std::f32::consts::PI;
use calx::V2;

/// Hex grid directions.
#[deriving(Eq, PartialEq, Clone, Show, FromPrimitive, Encodable, Decodable)]
pub enum Dir6 {
    North = 0,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

impl Dir6 {
    /// Convert a vector into the closest hex direction.
    pub fn from_v2(v: V2<int>) -> Dir6 {
        let hexadecant = {
            let width = PI / 8.0;
            let mut radian = (v.0 as f32).atan2(-v.1 as f32);
            if radian < 0.0 { radian += 2.0 * PI }
            (radian / width).floor() as int
        };

        // The hexadecants (00 to 15) and the hex directions (*0* to *5*) around
        // the origin.
        //
        //    *0*       *1*
        //       \ 14 15 | 00 01
        //       13\     |      02
        //           \   |
        //     12      \ |        03
        // *5* ----------O-X------- *2*
        //     11        Y \      04
        //               |   \
        //       10      |     \05
        //         09 08 | 07 06 \
        //              *4*       *3*
        //
        // Vectors that are in a space between two hex direction vectors are
        // snapped to the hex direction vector nearer to them. The hexadecants
        // are a convenience structure that lets us assign every vector using
        // the single hexadecant value.
        Dir6::from_int(match hexadecant {
            13 | 14 => 0,
            15 | 0 | 1 => 1,
            2 | 3 | 4 => 2,
            5 | 6 => 3,
            7 | 8 | 9 => 4,
            10 | 11 | 12 => 5,
            _ => panic!("Bad hexadecant")
        })
    }

    /// Convert an integer to a hex dir using modular arithmetic.
    pub fn from_int(i: int) -> Dir6 {
        FromPrimitive::from_int(i.mod_floor(&6)).unwrap()
    }

    /// Convert a hex dir into the corresponding unit vector.
    pub fn to_v2(&self) -> V2<int> {
        [V2(-1, -1),
         V2( 0, -1),
         V2( 1,  0),
         V2( 1,  1),
         V2( 0,  1),
         V2(-1,  0)][*self as uint]
    }

    /// Iterate through the six hex dirs in the standard order.
    pub fn iter() -> slice::Items<'static, Dir6> {
        static DIRS: [Dir6, ..6] = [
            North,
            NorthEast,
            SouthEast,
            South,
            SouthWest,
            NorthWest];

        DIRS.iter()
    }
}

#[cfg(test)]
mod test {
    use calx::V2;
    // XXX: Why doesn't super::* work here?
    use super::{Dir6, North, NorthEast, SouthEast, South, SouthWest, NorthWest};

    #[test]
    fn test_dir6() {
        assert_eq!(North, Dir6::from_int(0));
        assert_eq!(NorthWest, Dir6::from_int(-1));
        assert_eq!(NorthWest, Dir6::from_int(5));
        assert_eq!(NorthEast, Dir6::from_int(1));

        assert_eq!(NorthEast, Dir6::from_v2(V2(20, -21)));
        assert_eq!(SouthEast, Dir6::from_v2(V2(20, -10)));
        assert_eq!(North, Dir6::from_v2(V2(-10, -10)));
        assert_eq!(South, Dir6::from_v2(V2(1, 1)));

        for i in range(0, 6) {
            let d = Dir6::from_int(i);
            let v = d.to_v2();
            let v1 = Dir6::from_int(i - 1).to_v2();
            let v2 = Dir6::from_int(i + 1).to_v2();

            // Test static iter
            assert_eq!(Some(d), Dir6::iter().nth(i as uint).map(|&x| x));

            // Test vector mapping.
            assert_eq!(d, Dir6::from_v2(v));

            // Test opposite dir vector mapping.
            assert_eq!(Dir6::from_int(i + 3), Dir6::from_v2(-v));

            // Test approximation of longer vectors.
            assert_eq!(d, Dir6::from_v2(v * 3));
            assert_eq!(d, Dir6::from_v2(v * 3 + v1));
            assert_eq!(d, Dir6::from_v2(v * 3 + v2));
        }
    }
}