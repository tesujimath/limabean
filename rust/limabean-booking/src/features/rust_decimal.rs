use super::{Number, Sign};

impl Number for rust_decimal::Decimal {
    fn abs(&self) -> Self {
        rust_decimal::Decimal::abs(self)
    }

    fn sign(&self) -> Option<Sign> {
        use Sign::*;

        if self.is_zero() {
            None
        } else if self.is_sign_negative() {
            Some(Negative)
        } else {
            Some(Positive)
        }
    }

    fn zero() -> Self {
        rust_decimal::Decimal::ZERO
    }

    fn new(m: i64, scale: u32) -> Self {
        rust_decimal::Decimal::new(m, scale)
    }

    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

    fn scale(&self) -> u32 {
        rust_decimal::Decimal::scale(self)
    }

    fn rescaled(self, scale: u32) -> Self {
        let mut n = self;
        n.rescale(scale);
        n
    }
}
