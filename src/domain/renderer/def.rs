use std::ops::Mul;

use crate::domain::color::Spectrum;
use crate::domain::image::Image;
use crate::domain::material::def::FluxEstimation;
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
    Light(Spectrum),
    All(Box<ContributionInner>),
}

impl Contribution {
    pub fn new() -> Self {
        Spectrum::zero().into()
    }

    pub fn average(estimations: Vec<Contribution>) -> Contribution {
        if estimations.is_empty() {
            return Contribution::new();
        }

        let light_sum = (estimations.iter()).map(|e| e.light()).sum::<Spectrum>();
        let light_avg = light_sum / Val::from(estimations.len());

        let iter_global = estimations.iter().flat_map(|e| e.global());
        let global_avg = FluxEstimation::average(iter_global);

        let iter_caustic = estimations.iter().flat_map(|e| e.caustic());
        let caustic_avg = FluxEstimation::average(iter_caustic);

        if global_avg.is_empty() && caustic_avg.is_empty() {
            light_avg.into()
        } else {
            let mut res = Contribution::from(light_avg);
            res.set_global(global_avg);
            res.set_caustic(caustic_avg);
            res
        }
    }

    pub fn add_light(&mut self, light: Spectrum) {
        match self {
            Contribution::Light(color) => *color += light,
            Contribution::All(s) => s.light += light,
        }
    }

    pub fn set_global(&mut self, global: FluxEstimation) {
        if global.is_empty() {
            return;
        }
        match self {
            Contribution::Light(light) => {
                *self = Self::All(Box::new(ContributionInner {
                    light: *light,
                    global,
                    caustic: FluxEstimation::empty(),
                }))
            }
            Contribution::All(s) => {
                s.global = global;
            }
        }
    }

    pub fn set_caustic(&mut self, caustic: FluxEstimation) {
        if caustic.is_empty() {
            return;
        }
        match self {
            Contribution::Light(light) => {
                *self = Self::All(Box::new(ContributionInner {
                    light: *light,
                    caustic,
                    global: FluxEstimation::empty(),
                }))
            }
            Contribution::All(s) => {
                s.caustic = caustic;
            }
        }
    }

    pub fn light(&self) -> Spectrum {
        match self {
            Contribution::Light(light) => *light,
            Contribution::All(s) => s.light,
        }
    }

    pub fn global(&self) -> Option<&FluxEstimation> {
        match self {
            Contribution::Light(_) => None,
            Contribution::All(s) => Some(&s.global),
        }
    }

    pub fn caustic(&self) -> Option<&FluxEstimation> {
        match self {
            Contribution::Light(_) => None,
            Contribution::All(s) => Some(&s.caustic),
        }
    }
}

impl From<Spectrum> for Contribution {
    fn from(value: Spectrum) -> Self {
        Self::Light(value)
    }
}

impl Mul<Val> for Contribution {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        match self {
            Contribution::Light(light) => (light * rhs).into(),
            Contribution::All(mut s) => {
                s.light *= rhs;
                s.global = s.global * rhs;
                s.caustic = s.caustic * rhs;
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

impl Mul<Spectrum> for Contribution {
    type Output = Self;

    fn mul(self, rhs: Spectrum) -> Self::Output {
        match self {
            Contribution::Light(light) => (light * rhs).into(),
            Contribution::All(mut s) => {
                s.light *= rhs;
                s.global = s.global * rhs;
                s.caustic = s.caustic * rhs;
                Contribution::All(s)
            }
        }
    }
}

impl Mul<Contribution> for Spectrum {
    type Output = <Contribution as Mul<Spectrum>>::Output;

    #[inline]
    fn mul(self, rhs: Contribution) -> Self::Output {
        rhs * self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContributionInner {
    light: Spectrum,
    global: FluxEstimation,
    caustic: FluxEstimation,
}
