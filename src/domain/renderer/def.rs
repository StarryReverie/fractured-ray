use std::ops::{Add, Mul};

use crate::domain::color::core::Spectrum;
use crate::domain::image::core::Image;
use crate::domain::material::def::{FluxEstimation, RefDynMaterial};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;

use super::{PmContext, PmState, RtContext, RtState};

#[cfg_attr(test, mockall::automock)]
pub trait Renderer: Send + Sync + 'static {
    fn render(&self) -> Image;

    fn trace<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        state: RtState,
        ray: &Ray,
        range: DisRange,
    ) -> Contribution;

    #[allow(clippy::needless_lifetimes)]
    fn trace_to<'a, 'i, 'm>(
        &'a self,
        context: &mut RtContext<'a>,
        state: RtState,
        ray: &Ray,
        target: Option<(&'i RayIntersection, RefDynMaterial<'m>)>,
    ) -> Contribution;

    fn emit<'a>(
        &'a self,
        context: &mut PmContext<'a>,
        state: PmState,
        photon: &PhotonRay,
        range: DisRange,
    );
}

#[derive(Debug, Clone, PartialEq)]
pub enum Contribution {
    Light(Spectrum),
    Global(FluxEstimation),
    Caustic(FluxEstimation),
    All(Box<ContributionInner>),
}

impl Contribution {
    #[inline]
    pub fn new() -> Self {
        Self::Light(Spectrum::zero())
    }

    #[inline]
    pub fn from_light(light: Spectrum) -> Self {
        Self::Light(light)
    }

    #[inline]
    pub fn from_global(global: FluxEstimation) -> Self {
        Self::Global(global)
    }

    #[inline]
    pub fn from_caustic(caustic: FluxEstimation) -> Self {
        Self::Caustic(caustic)
    }

    pub fn average(estimations: Vec<Contribution>) -> Contribution {
        if estimations.is_empty() {
            return Self::new();
        }

        let light_sum = (estimations.iter()).map(|e| e.light()).sum::<Spectrum>();
        let light_avg = light_sum / Val::from(estimations.len());

        let iter_global = estimations.iter().flat_map(|e| e.global());
        let global_avg = FluxEstimation::average(iter_global);

        let iter_caustic = estimations.iter().flat_map(|e| e.caustic());
        let caustic_avg = FluxEstimation::average(iter_caustic);

        Self::from_light(light_avg)
            + Self::from_global(global_avg)
            + Self::from_caustic(caustic_avg)
    }

    pub fn light(&self) -> Spectrum {
        match self {
            Self::Light(light) => *light,
            Self::All(s) => s.light,
            _ => Spectrum::zero(),
        }
    }

    pub fn global(&self) -> Option<&FluxEstimation> {
        match self {
            Self::Global(global) => Some(global),
            Self::All(s) => Some(&s.global),
            _ => None,
        }
    }

    pub fn caustic(&self) -> Option<&FluxEstimation> {
        match self {
            Self::Caustic(caustic) => Some(caustic),
            Self::All(s) => Some(&s.caustic),
            _ => None,
        }
    }

    fn into_all(self) -> Self {
        match self {
            Self::Light(light) => Self::All(Box::new(ContributionInner {
                light,
                ..ContributionInner::zero()
            })),
            Self::Global(global) => Self::All(Box::new(ContributionInner {
                global,
                ..ContributionInner::zero()
            })),
            Self::Caustic(caustic) => Self::All(Box::new(ContributionInner {
                caustic,
                ..ContributionInner::zero()
            })),
            s => s,
        }
    }

    pub fn clamp(mut self) -> Self {
        if let Self::All(s) = &mut self {
            let max_radius = s.global.radius();
            s.caustic.clamp_radius(max_radius);
        }
        self
    }
}

impl Add for Contribution {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (res, rhs) = match (&self, &rhs) {
            (Self::All(_), _) => (self, rhs),
            (_, Self::All(_)) => (rhs, self),
            (_, _) => (self.into_all(), rhs),
        };

        let Self::All(mut res) = res else {
            unreachable!("lhs should match Self::All(_) now");
        };

        match rhs {
            Self::Light(light) => res.light += light,
            Self::Global(global) => res.global += global,
            Self::Caustic(caustic) => res.caustic += caustic,
            Self::All(rhs) => {
                res.light += rhs.light;
                res.global += rhs.global;
                res.caustic += rhs.caustic;
            }
        }

        Self::All(res)
    }
}

impl Mul<Val> for Contribution {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        match self {
            Self::Light(light) => Self::Light(light * rhs),
            Self::Global(global) => Self::Global(global * rhs),
            Self::Caustic(caustic) => Self::Caustic(caustic * rhs),
            Self::All(mut s) => {
                s.light *= rhs;
                s.global *= rhs;
                s.caustic *= rhs;
                Self::All(s)
            }
        }
    }
}

impl Mul<Contribution> for Val {
    type Output = <Contribution as Mul<Self>>::Output;

    #[inline]
    fn mul(self, rhs: Contribution) -> Self::Output {
        rhs * self
    }
}

impl Mul<Spectrum> for Contribution {
    type Output = Self;

    fn mul(self, rhs: Spectrum) -> Self::Output {
        match self {
            Self::Light(light) => Self::Light(light * rhs),
            Self::Global(global) => Self::Global(global * rhs),
            Self::Caustic(caustic) => Self::Caustic(caustic * rhs),
            Self::All(mut s) => {
                s.light *= rhs;
                s.global *= rhs;
                s.caustic *= rhs;
                Self::All(s)
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

impl ContributionInner {
    #[inline]
    fn zero() -> Self {
        Self {
            light: Spectrum::zero(),
            global: FluxEstimation::empty(),
            caustic: FluxEstimation::empty(),
        }
    }
}
