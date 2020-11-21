use std::{fmt::Display, net::IpAddr};
use serde::{Serialize, Deserialize};

pub enum HostnameError {
    UnresolvedHostname(std::io::Error),
    WrongIpVType
}

impl Display for HostnameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostnameError::UnresolvedHostname(e) => write!(f, "Could not lookup hostname! Reason: {}", e),
            HostnameError::WrongIpVType => write!(f, "No hostname ips matched the requested IP version.")
        }
    }
}

pub fn get_hostname_ip(hostname: &str, isipv6: bool) -> Result<IpAddr, HostnameError> {
    match dns_lookup::lookup_host(&hostname) {
        Ok(results) => match results.into_iter().find(|&x| x.is_ipv6() && isipv6 || x.is_ipv4() && !isipv6) {
            Some(ip) => Ok(ip),
            None => Err(HostnameError::WrongIpVType)
        }
        Err(e) => Err(HostnameError::UnresolvedHostname(e))
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