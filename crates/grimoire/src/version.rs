use core::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionError {
    input: String,
}

impl Version {
    pub fn parse(input: &str) -> Result<Self, VersionError> {
        let mut parts = input.split('.');
        let Some(major) = parts.next().and_then(parse_component) else {
            return Err(VersionError {
                input: input.to_owned(),
            });
        };
        let Some(minor) = parts.next().and_then(parse_component) else {
            return Err(VersionError {
                input: input.to_owned(),
            });
        };
        let Some(patch) = parts.next().and_then(parse_component) else {
            return Err(VersionError {
                input: input.to_owned(),
            });
        };
        if parts.next().is_some() {
            return Err(VersionError {
                input: input.to_owned(),
            });
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    #[must_use]
    pub const fn major(self) -> u64 {
        self.major
    }

    #[must_use]
    pub const fn minor(self) -> u64 {
        self.minor
    }

    #[must_use]
    pub const fn patch(self) -> u64 {
        self.patch
    }
}

fn parse_component(input: &str) -> Option<u64> {
    (!input.is_empty() && input.chars().all(|character| character.is_ascii_digit()))
        .then(|| input.parse().ok())
        .flatten()
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Display for VersionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid version `{}`", self.input)
    }
}

impl std::error::Error for VersionError {}
