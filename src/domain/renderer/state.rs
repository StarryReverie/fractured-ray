#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtState {
    visible: bool,
    depth: u8,
    invisible_depth: u8,
    filter_emissive: bool,
}

impl RtState {
    pub fn new() -> Self {
        Self {
            visible: true,
            depth: 0,
            invisible_depth: 0,
            filter_emissive: false,
        }
    }

    pub fn mark_invisible(self) -> Self {
        Self {
            visible: false,
            ..self
        }
    }

    pub fn increment_depth(self) -> Self {
        if self.visible {
            Self {
                depth: self.depth + 1,
                ..self
            }
        } else {
            Self {
                depth: self.depth + 1,
                invisible_depth: self.invisible_depth + 1,
                ..self
            }
        }
    }

    pub fn with_filter_emissive(self, filter_emissive: bool) -> Self {
        Self {
            filter_emissive,
            ..self
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn depth(&self) -> usize {
        self.depth as usize
    }

    pub fn invisible_depth(&self) -> usize {
        self.invisible_depth as usize
    }

    pub fn filter_emissive(&self) -> bool {
        self.filter_emissive
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmState {
    has_specular: bool,
    policy: StoragePolicy,
}

impl PmState {
    pub fn new(has_specular: bool, policy: StoragePolicy) -> Self {
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

    pub fn policy(&self) -> StoragePolicy {
        self.policy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePolicy {
    Global,
    Caustic,
}
