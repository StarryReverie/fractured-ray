use snafu::prelude::*;

use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadAngle(Val);

impl SpreadAngle {
    #[inline]
    pub fn new(angle: Val) -> Result<Self, TryNewSpreadAngleError> {
        ensure!((Val(0.0)..=Val::PI).contains(&angle), InvalidAngleSnafu);
        Ok(Self((angle * Val(0.5)).cos()))
    }

    #[inline]
    pub fn hemisphere() -> Self {
        Self(Val(0.0))
    }

    #[inline]
    pub fn directional() -> Self {
        Self(Val(1.0))
    }

    #[inline]
    pub fn cos_half(&self) -> Val {
        self.0
    }

    #[inline]
    pub fn angle(&self) -> Val {
        self.0.acos() * Val(2.0)
    }

    #[inline]
    pub fn is_directional(&self) -> bool {
        self.0 == Val(1.0)
    }

    #[inline]
    pub fn is_hemisphere(&self) -> bool {
        self.0 == Val(0.0)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewSpreadAngleError {
    #[snafu(display("spread angle should be in [0, pi]"))]
    InvalidAngle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_angle_new_succeeds() {
        let angle = SpreadAngle::new(Val::PI / Val(2.0)).unwrap();
        assert_eq!(angle.cos_half(), Val(2.0).sqrt() / Val(2.0));
        assert_eq!(angle.angle(), Val::PI / Val(2.0));

        let angle = SpreadAngle::directional();
        assert_eq!(angle.cos_half(), Val(1.0));
        assert_eq!(angle.angle(), Val(0.0));

        let angle = SpreadAngle::hemisphere();
        assert_eq!(angle.cos_half(), Val(0.0));
        assert_eq!(angle.angle(), Val::PI);
    }

    #[test]
    fn spread_angle_new_fails_when_angle_is_invalid() {
        assert!(matches!(
            SpreadAngle::new(Val(-1.0)),
            Err(TryNewSpreadAngleError::InvalidAngle),
        ));

        assert!(matches!(
            SpreadAngle::new(Val::PI + Val(1.0)),
            Err(TryNewSpreadAngleError::InvalidAngle),
        ));
    }
}
