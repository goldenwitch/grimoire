use core::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Namespace(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceError {
    input: String,
}

impl Namespace {
    pub fn parse(input: &str) -> Result<Self, NamespaceError> {
        let payload = input.strip_prefix("https://");
        let valid = payload.is_some_and(|rest| {
            let authority = rest.split('/').next().unwrap_or_default();
            !authority.is_empty()
                && input.is_ascii()
                && input.chars().all(|character| {
                    !character.is_ascii_control() && !character.is_ascii_whitespace()
                })
        });
        if valid {
            Ok(Self(input.to_owned()))
        } else {
            Err(NamespaceError {
                input: input.to_owned(),
            })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Namespace {
    type Error = NamespaceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for NamespaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid extension namespace `{}`", self.input)
    }
}

impl std::error::Error for NamespaceError {}
