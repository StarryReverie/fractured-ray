use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

use super::NoiseGenerator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerlinNoiseGenerator {}

impl PerlinNoiseGenerator {
    const PERMUTATION_BIT_LEN: usize = 8;
    const PERMUTATION_INDEX_BITMASK: usize = (1 << Self::PERMUTATION_BIT_LEN) - 1;

    #[rustfmt::skip]
    const PERMUTATION: [usize; 1 << Self::PERMUTATION_BIT_LEN] = [
        151, 160, 137,  91,  90,  15, 131,  13, 201,  95,  96,  53, 194, 233,   7, 225,
        140,  36, 103,  30,  69, 142,   8,  99,  37, 240,  21,  10,  23, 190,   6, 148,
        247, 120, 234,  75,   0,  26, 197,  62,  94, 252, 219, 203, 117,  35,  11,  32,
         57, 177,  33,  88, 237, 149,  56,  87, 174,  20, 125, 136, 171, 168,  68, 175,
         74, 165,  71, 134, 139,  48,  27, 166,  77, 146, 158, 231,  83, 111, 229, 122,
         60, 211, 133, 230, 220, 105,  92,  41,  55,  46, 245,  40, 244, 102, 143,  54,
         65,  25,  63, 161,   1, 216,  80,  73, 209,  76, 132, 187, 208,  89,  18, 169,
        200, 196, 135, 130, 116, 188, 159,  86, 164, 100, 109, 198, 173, 186,   3,  64,
         52, 217, 226, 250, 124, 123,   5, 202,  38, 147, 118, 126, 255,  82,  85, 212,
        207, 206,  59, 227,  47,  16,  58,  17, 182, 189,  28,  42, 223, 183, 170, 213,
        119, 248, 152,   2,  44, 154, 163,  70, 221, 153, 101, 155, 167,  43, 172,   9,
        129,  22,  39, 253,  19,  98, 108, 110,  79, 113, 224, 232, 178, 185, 112, 104,
        218, 246,  97, 228, 251,  34, 242, 193, 238, 210, 144,  12, 191, 179, 162, 241,
         81,  51, 145, 235, 249,  14, 239, 107,  49, 192, 214,  31, 181, 199, 106, 157,
        184,  84, 204, 176, 115, 121,  50,  45, 127,   4, 150, 254, 138, 236, 205,  93,
        222, 114,  67,  29,  24,  72, 243, 141, 128, 195,  78,  66, 215,  61, 156, 180,
    ];

    #[inline]
    pub fn new() -> Self {
        Self {}
    }

    #[inline]
    fn perm(index: usize) -> usize {
        Self::PERMUTATION[index & Self::PERMUTATION_INDEX_BITMASK]
    }

    #[inline]
    fn fade(x: Val) -> Val {
        x * x * x * (x * (x * Val(6.0) - Val(15.0)) + Val(10.0))
    }

    #[inline]
    fn gradient_dot(hash: usize, x: Val, y: Val, z: Val) -> Val {
        match hash & 0xf {
            0x0 => x + y,
            0x1 => -x + y,
            0x2 => x - y,
            0x3 => -x - y,
            0x4 => x + z,
            0x5 => -x + z,
            0x6 => x - z,
            0x7 => -x - z,
            0x8 => y + z,
            0x9 => -y + z,
            0xa => y - z,
            0xb => -y - z,
            0xc => y + x,
            0xd => -y + z,
            0xe => y - x,
            0xf => -y - z,
            _ => unreachable!("`hash & 0xf` should be in [0x0, 0xf)"),
        }
    }
}

impl NoiseGenerator for PerlinNoiseGenerator {
    #[allow(clippy::identity_op)]
    fn evaluate(&self, point: Point) -> Val {
        let (xi, yi, zi) = (point.x().floor(), point.y().floor(), point.z().floor());
        let (xf, yf, zf) = (point.x() - xi, point.y() - yi, point.z() - zi);

        let xi = (i64::from(xi) & (Self::PERMUTATION_INDEX_BITMASK as i64)) as usize;
        let yi = (i64::from(yi) & (Self::PERMUTATION_INDEX_BITMASK as i64)) as usize;
        let zi = (i64::from(zi) & (Self::PERMUTATION_INDEX_BITMASK as i64)) as usize;

        let (tx, ty, tz) = (Self::fade(xf), Self::fade(yf), Self::fade(zf));

        let h000 = Self::perm(Self::perm(Self::perm(xi + 0) + yi + 0) + zi + 0);
        let h001 = Self::perm(Self::perm(Self::perm(xi + 0) + yi + 0) + zi + 1);
        let h010 = Self::perm(Self::perm(Self::perm(xi + 0) + yi + 1) + zi + 0);
        let h011 = Self::perm(Self::perm(Self::perm(xi + 0) + yi + 1) + zi + 1);
        let h100 = Self::perm(Self::perm(Self::perm(xi + 1) + yi + 0) + zi + 0);
        let h101 = Self::perm(Self::perm(Self::perm(xi + 1) + yi + 0) + zi + 1);
        let h110 = Self::perm(Self::perm(Self::perm(xi + 1) + yi + 1) + zi + 0);
        let h111 = Self::perm(Self::perm(Self::perm(xi + 1) + yi + 1) + zi + 1);

        let d000 = Self::gradient_dot(h000, xf - Val(0.0), yf - Val(0.0), zf - Val(0.0));
        let d001 = Self::gradient_dot(h001, xf - Val(0.0), yf - Val(0.0), zf - Val(1.0));
        let d010 = Self::gradient_dot(h010, xf - Val(0.0), yf - Val(1.0), zf - Val(0.0));
        let d011 = Self::gradient_dot(h011, xf - Val(0.0), yf - Val(1.0), zf - Val(1.0));
        let d100 = Self::gradient_dot(h100, xf - Val(1.0), yf - Val(0.0), zf - Val(0.0));
        let d101 = Self::gradient_dot(h101, xf - Val(1.0), yf - Val(0.0), zf - Val(1.0));
        let d110 = Self::gradient_dot(h110, xf - Val(1.0), yf - Val(1.0), zf - Val(0.0));
        let d111 = Self::gradient_dot(h111, xf - Val(1.0), yf - Val(1.0), zf - Val(1.0));

        let xy00 = Val::lerp(d000, d001, tz);
        let xy01 = Val::lerp(d010, d011, tz);
        let xy10 = Val::lerp(d100, d101, tz);
        let xy11 = Val::lerp(d110, d111, tz);

        let x0 = Val::lerp(xy00, xy01, ty);
        let x1 = Val::lerp(xy10, xy11, ty);

        Val::lerp(x0, x1, tx)
    }
}
