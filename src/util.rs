use std::{fmt::Display, io, net::IpAddr, ops::Add, path::PathBuf, ops::Sub};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum HostnameLookupError {
    UnresolvedHostname(io::Error),
    WrongIpVersion
}

impl Display for HostnameLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostnameLookupError::UnresolvedHostname(e) => write!(f, "Could not lookup hostname! Reason: {}", e),
            HostnameLookupError::WrongIpVersion => write!(f, "No hostname ips matched the requested IP version.")
        }
    }
}

impl From<io::Error> for HostnameLookupError {
    fn from(e: io::Error) -> Self {
        HostnameLookupError::UnresolvedHostname(e)
    }
}

pub fn get_hostname_ip(hostname: &str, isipv6: bool) -> Result<IpAddr, HostnameLookupError> {
    match dns_lookup::lookup_host(&hostname)?.into_iter().find(|&x| x.is_ipv6() && isipv6 || x.is_ipv4() && !isipv6) {
        Some(ip) => Ok(ip),
        None => Err(HostnameLookupError::WrongIpVersion)
    }
}

#[derive(Eq, PartialEq)]
pub enum Category {
    Shared,
    Master,
    Server,
    Init
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, PartialOrd)]
pub enum VarReaderTypes {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64)
}

impl VarReaderTypes {
    pub fn get_as_f64(&self) -> Option<&f64> {
        if let VarReaderTypes::F64(data) = self {return Some(data)}
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InDataTypes {
    Bool,
    I32,
    I64,
    F64
}

pub struct NumberDigits {
    digits: Vec<i32>
}

impl NumberDigits {
    pub fn new(value: i32) -> Self {
        let mut digits = Vec::new();
        let mut value = value;

        while value > 0 {
            digits.push(value % 10);
            value /= 10;
        }

        Self {
            digits
        }
    }
    // Returns a 0 to simulate padding if the value is missing
    // Reads left to right
    pub fn get(&self, index: usize) -> i32 {
        if index + 1 > self.digits.len() {return 0}
        return self.digits[index]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.x-rhs.x,
            y: self.y-rhs.y,
            z: self.z-rhs.z,
        }
    }
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.x+rhs.x,
            y: self.y+rhs.y,
            z: self.z+rhs.z,
        }
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Vector3 {x: 0.0, y: 0.0, z: 0.0}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_number_digits() {
        let digits = NumberDigits::new(503);
        // Get 1s place
        assert_eq!(digits.get(0), 3);
        assert_eq!(digits.get(1), 0);
        assert_eq!(digits.get(2), 5);
        // Simulate padding in thousands place
        assert_eq!(digits.get(3), 0);
    }
}