use core::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Address(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressError {
    input: String,
}

impl Address {
    pub fn parse(input: &str) -> Result<Self, AddressError> {
        let valid = input.strip_prefix('@').is_some_and(|rest| {
            !rest.is_empty()
                && rest.split('/').all(|segment| {
                    !segment.is_empty()
                        && segment.chars().all(|character| {
                            character.is_ascii_alphanumeric()
                                || character == '_'
                                || character == '-'
                        })
                })
        });
        if valid {
            Ok(Self(input.to_owned()))
        } else {
            Err(AddressError {
                input: input.to_owned(),
            })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Address {
    type Error = AddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for AddressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid address `{}`", self.input)
    }
}

impl std::error::Error for AddressError {}
