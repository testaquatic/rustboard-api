use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::User => write!(f, "user"),
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            _ => Err(format!("알 수 없는 역할: {}", s)),
        }
    }
}
