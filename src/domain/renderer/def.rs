use std::ops::Mul;

use crate::domain::color::Color;
use crate::domain::image::Image;
use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::photon::PhotonRay;

use super::{PmContext, PmState, RtContext, RtState};

#[cfg_attr(test, mockall::automock)]
pub trait Renderer: Send + Sync + 'static {
    fn render(&self) -> Image;

    fn trace<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        state: RtState,
        ray: Ray,
        range: DisRange,
    ) -> Contribution;

    fn emit<'a>(
        &'a self,
        context: &mut PmContext<'a>,
        state: PmState,
        photon: PhotonRay,
        range: DisRange,
    );
}

#[derive(Debug, Clone, PartialEq)]
pub enum Contribution {
    Light(Color),
    All(Box<ContributionInner>),
}

impl Contribution {
    pub fn new() -> Self {
        Color::BLACK.into()
    }

    pub fn add_light(&mut self, light: Color) {
        match self {
            Contribution::Light(color) => *color = *color + light,
            Contribution::All(s) => s.light = s.light + light,
        }
    }

    pub fn add_caustic(&mut self, flux: Vector, num: Val) {
        match self {
            Contribution::Light(light) => {
                *self = Self::All(Box::new(ContributionInner {
                    light: *light,
                    flux_caustic: flux,
                    flux_global: Vector::zero(),
                    num_caustic: num,
                    num_global: Val(0.0),
                }))
            }
            Contribution::All(s) => {
                s.flux_caustic = s.flux_caustic + flux;
                s.num_caustic += num;
            }
        }
    }

    pub fn add_global(&mut self, flux: Vector, num: Val) {
        match self {
            Contribution::Light(light) => {
                *self = Self::All(Box::new(ContributionInner {
                    light: *light,
                    flux_caustic: Vector::zero(),
                    flux_global: flux,
                    num_caustic: Val(0.0),
                    num_global: num,
                }))
            }
            Contribution::All(s) => {
                s.flux_global = s.flux_global + flux;
                s.num_global += num;
            }
        }
    }

    pub fn light(&self) -> Color {
        match self {
            Contribution::Light(light) => *light,
            Contribution::All(s) => s.light,
        }
    }

    pub fn flux_caustic(&self) -> Vector {
        match self {
            Contribution::Light(_) => Vector::zero(),
            Contribution::All(s) => s.flux_caustic,
        }
    }

    pub fn flux_global(&self) -> Vector {
        match self {
            Contribution::Light(_) => Vector::zero(),
            Contribution::All(s) => s.flux_global,
        }
    }

    pub fn num_caustic(&self) -> Val {
        match self {
            Contribution::Light(_) => Val(0.0),
            Contribution::All(s) => s.num_caustic,
        }
    }

    pub fn num_global(&self) -> Val {
        match self {
            Contribution::Light(_) => Val(0.0),
            Contribution::All(s) => s.num_global,
        }
    }
}

impl From<Color> for Contribution {
    fn from(value: Color) -> Self {
        Self::Light(value)
    }
}

impl Mul<Val> for Contribution {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        match self {
            Contribution::Light(light) => (light * rhs).into(),
            Contribution::All(mut s) => {
                *s = *s * rhs;
                Contribution::All(s)
            }
        }
    }
}

impl Mul<Contribution> for Val {
    type Output = Contribution;

    #[inline]
    fn mul(self, rhs: Contribution) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vector> for Contribution {
    type Output = Self;

    fn mul(self, rhs: Vector) -> Self::Output {
        match self {
            Contribution::Light(light) => (light * rhs).into(),
            Contribution::All(mut s) => {
                *s = *s * rhs;
                Contribution::All(s)
            }
        }
    }
}

impl Mul<Contribution> for Vector {
    type Output = Contribution;

    #[inline]
    fn mul(self, rhs: Contribution) -> Self::Output {
        rhs * self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContributionInner {
    light: Color,
    flux_caustic: Vector,
    flux_global: Vector,
    num_caustic: Val,
    num_global: Val,
}

impl Mul<Val> for ContributionInner {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        Self {
            light: self.light * rhs,
            flux_caustic: self.flux_caustic * rhs,
            flux_global: self.flux_global * rhs,
            ..self
        }
    }
}

impl Mul<ContributionInner> for Val {
    type Output = ContributionInner;

    #[inline]
    fn mul(self, rhs: ContributionInner) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vector> for ContributionInner {
    type Output = Self;

    fn mul(self, rhs: Vector) -> Self::Output {
        Self {
            light: self.light * rhs,
            flux_caustic: self.flux_caustic * rhs,
            flux_global: self.flux_global * rhs,
            ..self
        }
    }
}

impl Mul<ContributionInner> for Vector {
    type Output = ContributionInner;

    #[inline]
    fn mul(self, rhs: ContributionInner) -> Self::Output {
        rhs * self
    }
}
