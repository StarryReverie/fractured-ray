use getset::{CopyGetters, WithSetters};

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters, WithSetters)]
pub struct RtState {
    #[getset(get_copy = "pub")]
    visible: bool,
    depth: u8,
    invisible_depth: u8,
    #[getset(get_copy = "pub", set_with = "pub")]
    skip_emissive: bool,
}

impl RtState {
    pub fn new() -> Self {
        Self {
            visible: true,
            depth: 0,
            invisible_depth: 0,
            skip_emissive: false,
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

    pub fn depth(&self) -> usize {
        self.depth as usize
    }

    pub fn invisible_depth(&self) -> usize {
        self.invisible_depth as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters, WithSetters)]
#[getset(get_copy = "pub")]
pub struct PmState {
    #[getset(set_with = "pub")]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePolicy {
    Global,
    Caustic,
}
