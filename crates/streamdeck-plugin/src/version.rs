use crate::error::{Error, Result};

/// Parsed Stream Deck / plugin version (`major[.minor[.patch[.build]]]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
}

impl Version {
    pub fn parse(value: &str) -> Result<Self> {
        if !is_valid(value) {
            return Err(Error::InvalidVersion {
                value: value.to_string(),
            });
        }
        let segs: Vec<&str> = value.split('.').collect();
        let parse_at = |i: usize| -> Result<u32> {
            match segs.get(i).copied().filter(|s| !s.is_empty()) {
                Some(s) => s.parse::<u32>().map_err(|_| Error::InvalidVersion {
                    value: value.to_string(),
                }),
                None => Ok(0),
            }
        };
        Ok(Self {
            major: parse_at(0)?,
            minor: parse_at(1)?,
            patch: parse_at(2)?,
            build: parse_at(3)?,
        })
    }

    pub fn compare_to(&self, other: &Version) -> i32 {
        let a = [self.major, self.minor, self.patch, self.build];
        let b = [other.major, other.minor, other.patch, other.build];
        for i in 0..4 {
            if a[i] < b[i] {
                return -1;
            }
            if a[i] > b[i] {
                return 1;
            }
        }
        0
    }

    pub fn as_major_minor(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

fn is_valid(value: &str) -> bool {
    fn part(s: &str) -> bool {
        if s == "0" {
            return true;
        }
        !s.is_empty() && !s.starts_with('0') && s.chars().all(|c| c.is_ascii_digit())
    }
    let segs: Vec<&str> = value.split('.').collect();
    (1..=4).contains(&segs.len()) && segs.iter().all(|s| part(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_partial_versions() {
        let v = Version::parse("7.1").unwrap();
        assert_eq!(
            v,
            Version {
                major: 7,
                minor: 1,
                patch: 0,
                build: 0
            }
        );
        assert!(Version::parse("99.8.6.54321").is_ok());
        assert!(Version::parse("nope").is_err());
        assert!(Version::parse("4294967296").is_err());
        assert_eq!(Version::parse("4294967295").unwrap().major, u32::MAX);
    }

    #[test]
    fn compares_in_order() {
        let a = Version::parse("6.5").unwrap();
        let b = Version::parse("7.0").unwrap();
        assert_eq!(a.compare_to(&b), -1);
        assert_eq!(b.compare_to(&a), 1);
        assert_eq!(a.compare_to(&a), 0);
    }
}
