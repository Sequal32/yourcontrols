/// Splits an `i32` into a `Vec<i32>` corresponding to the digits of the number read left to right.
pub struct NumberDigits {
    digits: Vec<i32>,
}

impl NumberDigits {
    pub fn new(value: i32) -> Self {
        let mut digits = Vec::new();
        let mut value = value;

        while value > 0 {
            digits.push(value % 10);
            value /= 10;
        }

        digits.reverse();

        Self { digits }
    }
    // Returns a 0 to simulate padding if the value is missing.
    pub fn get(&self, index: usize) -> i32 {
        if index + 1 > self.digits.len() {
            return 0;
        }
        return self.digits[index];
    }
}
