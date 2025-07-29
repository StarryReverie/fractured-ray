#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmState {
    has_specular: bool,
    policy: PmPolicy,
}

impl PmState {
    pub fn new(has_specular: bool, policy: PmPolicy) -> Self {
        Self {
            has_specular,
            policy,
        }
    }

    pub fn with_has_specular(self, has_specular: bool) -> Self {
        Self {
            has_specular,
            ..self
        }
    }

    pub fn has_specular(&self) -> bool {
        self.has_specular
    }

    pub fn policy(&self) -> PmPolicy {
        self.policy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmPolicy {
    Global,
    Caustic,
}
