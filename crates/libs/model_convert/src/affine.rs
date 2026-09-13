//! The 3x4 row-major affine transform every bone transform in the suite is stored in:
//! SKL bind poses, `.model` inverse binds, IR bones. Composition and inversion are done in
//! `f64` so inverting a real bone matrix twice returns within float epsilon of the input.

/// A 3x4 row-major affine transform, three rows of `[r0, r1, r2, t]`: the layout the SKL and
/// `.model` files store a bone's bind (or inverse bind) transform in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine(pub [f32; 12]);

impl Affine {
    /// The transform that moves nothing.
    pub const IDENTITY: Affine = Affine([
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0,
    ]);

    /// A transform from its rotation part and its translation.
    pub fn from_rotation_translation(rotation: [[f32; 3]; 3], translation: [f32; 3]) -> Affine {
        Affine([
            rotation[0][0],
            rotation[0][1],
            rotation[0][2],
            translation[0],
            rotation[1][0],
            rotation[1][1],
            rotation[1][2],
            translation[1],
            rotation[2][0],
            rotation[2][1],
            rotation[2][2],
            translation[2],
        ])
    }

    /// The rotation part, three rows of three.
    pub fn rotation(&self) -> [[f32; 3]; 3] {
        let m = &self.0;
        [[m[0], m[1], m[2]], [m[4], m[5], m[6]], [m[8], m[9], m[10]]]
    }

    /// The translation part.
    pub fn translation(&self) -> [f32; 3] {
        [self.0[3], self.0[7], self.0[11]]
    }

    /// `self` applied after `other`: `(self * other)(p) == self(other(p))`.
    pub fn multiply(&self, other: &Affine) -> Affine {
        let a = self.rotation64();
        let at = self.translation64();
        let b = other.rotation64();
        let bt = other.translation64();
        let mut rotation = [[0f64; 3]; 3];
        let mut translation = [0f64; 3];
        for row in 0..3 {
            for column in 0..3 {
                for k in 0..3 {
                    rotation[row][column] += a[row][k] * b[k][column];
                }
            }
            for k in 0..3 {
                translation[row] += a[row][k] * bt[k];
            }
            translation[row] += at[row];
        }
        Self::from64(rotation, translation)
    }

    /// `None` when the rotation part is singular (`|det| < 1e-12`).
    pub fn inverse(&self) -> Option<Affine> {
        let r = self.rotation64();
        let determinant = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
            - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
            + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
        if determinant.abs() < 1e-12 {
            return None;
        }
        // The inverse is the transposed cofactor matrix over the determinant: entry (row,
        // column) of the inverse is cofactor(column, row).
        let cofactor = |row: usize, column: usize| -> f64 {
            let rows: Vec<usize> = (0..3).filter(|other| *other != row).collect();
            let columns: Vec<usize> = (0..3).filter(|other| *other != column).collect();
            let minor = r[rows[0]][columns[0]] * r[rows[1]][columns[1]]
                - r[rows[0]][columns[1]] * r[rows[1]][columns[0]];
            let sign = if (row + column).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            sign * minor
        };
        let mut inverse_rotation = [[0f64; 3]; 3];
        for (row, out_row) in inverse_rotation.iter_mut().enumerate() {
            for (column, out) in out_row.iter_mut().enumerate() {
                *out = cofactor(column, row) / determinant;
            }
        }
        let t = self.translation64();
        let mut translation = [0f64; 3];
        for row in 0..3 {
            for k in 0..3 {
                translation[row] -= inverse_rotation[row][k] * t[k];
            }
        }
        Some(Self::from64(inverse_rotation, translation))
    }

    /// The full transform applied to a position.
    pub fn transform_point(&self, point: [f32; 3]) -> [f32; 3] {
        let rotation = self.rotation();
        let translation = self.translation();
        let mut out = [0f32; 3];
        for row in 0..3 {
            out[row] = translation[row]
                + rotation[row][0] * point[0]
                + rotation[row][1] * point[1]
                + rotation[row][2] * point[2];
        }
        out
    }

    /// Rotation part only (normals, tangents).
    pub fn transform_direction(&self, direction: [f32; 3]) -> [f32; 3] {
        let rotation = self.rotation();
        let mut out = [0f32; 3];
        for row in 0..3 {
            out[row] = rotation[row][0] * direction[0]
                + rotation[row][1] * direction[1]
                + rotation[row][2] * direction[2];
        }
        out
    }

    /// The largest `|a - b|` over the twelve components.
    pub fn max_component_delta(&self, other: &Affine) -> f32 {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max)
    }

    /// The rotation part widened to `f64` for the arithmetic paths.
    fn rotation64(&self) -> [[f64; 3]; 3] {
        let rotation = self.rotation();
        let mut out = [[0f64; 3]; 3];
        for row in 0..3 {
            for column in 0..3 {
                out[row][column] = f64::from(rotation[row][column]);
            }
        }
        out
    }

    /// The translation part widened to `f64`.
    fn translation64(&self) -> [f64; 3] {
        let translation = self.translation();
        [
            f64::from(translation[0]),
            f64::from(translation[1]),
            f64::from(translation[2]),
        ]
    }

    /// Store an `f64` result back into the `f32` layout.
    fn from64(rotation: [[f64; 3]; 3], translation: [f64; 3]) -> Affine {
        let mut out = [0f32; 12];
        for row in 0..3 {
            for column in 0..3 {
                out[row * 4 + column] = rotation[row][column] as f32;
            }
            out[row * 4 + 3] = translation[row] as f32;
        }
        Affine(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeletons::skeletons;
    use pes_version::PesVersion;

    /// A 90-degree rotation about y (x -> -z, z -> x) plus translation.
    fn rotated() -> Affine {
        let (sin, cos) = (1.0f32, 0.0f32);
        Affine::from_rotation_translation(
            [[cos, 0.0, sin], [0.0, 1.0, 0.0], [-sin, 0.0, cos]],
            [1.0, 2.0, 3.0],
        )
    }

    fn assert_close(a: &Affine, b: &Affine) {
        assert!(
            a.max_component_delta(b) < 1e-6,
            "delta {}",
            a.max_component_delta(b)
        );
    }

    #[test]
    fn identity_round_trips() {
        let identity = Affine::IDENTITY;
        assert_close(&identity.multiply(&identity), &identity);
        assert_close(&identity.inverse().expect("inverse"), &identity);
        assert_eq!(identity.transform_point([4.0, 5.0, 6.0]), [4.0, 5.0, 6.0]);
    }

    #[test]
    fn multiply_by_inverse_is_identity() {
        let m = rotated();
        let inverse = m.inverse().expect("inverse");
        assert_close(&m.multiply(&inverse), &Affine::IDENTITY);
        assert_close(&inverse.multiply(&m), &Affine::IDENTITY);
    }

    #[test]
    fn inverting_twice_returns_the_input() {
        let hand = skeletons(PesVersion::Pes21)
            .body
            .bone("sk_hand_l")
            .expect("sk_hand_l");
        let twice = hand
            .matrix
            .inverse()
            .and_then(|first| first.inverse())
            .expect("twice");
        assert_close(&twice, &hand.matrix);
    }

    #[test]
    fn singular_rotation_has_no_inverse() {
        assert!(Affine([0.0; 12]).inverse().is_none());
    }

    #[test]
    fn transform_moves_points_but_not_directions() {
        let m = rotated();
        // 90 degrees about y: (1,0,0) -> (0,0,-1); the point then moves by (1,2,3).
        let point = m.transform_point([1.0, 0.0, 0.0]);
        assert!((point[0] - 1.0).abs() < 1e-6, "{point:?}");
        assert!((point[1] - 2.0).abs() < 1e-6, "{point:?}");
        assert!((point[2] - 2.0).abs() < 1e-6, "{point:?}");
        let direction = m.transform_direction([1.0, 0.0, 0.0]);
        assert!((direction[0]).abs() < 1e-6, "{direction:?}");
        assert!((direction[1]).abs() < 1e-6, "{direction:?}");
        assert!((direction[2] - -1.0).abs() < 1e-6, "{direction:?}");
    }
}
