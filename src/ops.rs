/// Adds arithmetic operations similar to `checked_*` but returning Result with nice errors
pub trait ArithmeticTryOps<RHS = Self> where Self: Sized + core::fmt::Display + core::fmt::Debug + TypeName, RHS: Sized + core::fmt::Display + core::fmt::Debug {
    fn try_add(self, other: RHS) -> Result<Self, OverflowError<Self, RHS>>;
    fn try_sub(self, other: RHS) -> Result<Self, OverflowError<Self, RHS>>;
    fn try_mul(self, other: RHS) -> Result<Self, OverflowError<Self, RHS>>;
    fn try_div(self, other: RHS) -> Result<Self, DivisionByZeroError<Self>>;
    fn try_div_euclid(self, other: RHS) -> Result<Self, DivisionByZeroError<Self>>;
    fn try_rem(self, other: RHS) -> Result<Self, DivisionByZeroError<Self>>;
    fn try_rem_euclid(self, other: RHS) -> Result<Self, DivisionByZeroError<Self>>;
    fn try_pow(self, other: u32) -> Result<Self, OverflowError<Self, u32>>;
    //fn try_next_power_of_two(self) -> Result<Self, NextPowerOfTwoError<Self>;
    fn try_shl(self, other: u32) -> Result<Self, BigShiftError<Self>>;
    fn try_shr(self, other: u32) -> Result<Self, BigShiftError<Self>>;
}

#[derive(Debug, thiserror::Error)]
#[error("operation {left} {op} {right} overflowed")]
pub struct OverflowError<L: core::fmt::Display + core::fmt::Debug, R: core::fmt::Display + core::fmt::Debug> {
    left: L,
    op: &'static str,
    right: R,
}

#[derive(Debug, thiserror::Error)]
#[error("attempted to divide {0} by zero")]
pub struct DivisionByZeroError<T: core::fmt::Display + core::fmt::Debug>(T);

/// Retrurned from << and >> when RHS is too much
#[derive(Debug, thiserror::Error)]
#[error("operation {left} {op} {right} attempted to shift too much (the type of LHS is {})", L::type_name())]
pub struct BigShiftError<L: core::fmt::Display + TypeName + core::fmt::Debug> {
    left: L,
    op: &'static str,
    right: u32,
}

pub trait TypeName {
    fn type_name() -> &'static str;
}

macro_rules! impl_type_names {
    ($($type:ty),*) => {
        $(
            impl TypeName for $type {
                fn type_name() -> &'static str {
                    stringify!($type)
                }
            }
        )*
    }
}

macro_rules! impl_overflowing_op {
    ($try_op:ident, $check_op:ident, $rhs:ty, $op_str:expr) => {
        fn $try_op(self, other: $rhs) -> Result<Self, OverflowError<Self, $rhs>> {
            self.$check_op(other).ok_or(OverflowError {
                left: self,
                op: $op_str,
                right: other,
            })
        }
    }
}

macro_rules! impl_arith_op {
    ($($type:ty),*) => {
        $(
            impl_type_names!($type);

            impl ArithmeticTryOps for $type {
                impl_overflowing_op!(try_add, checked_add, $type, "+");
                impl_overflowing_op!(try_sub, checked_sub, $type, "-");
                impl_overflowing_op!(try_mul, checked_mul, $type, "*");
                // we don't use ^ to avoid mistaking it for bit xor
                impl_overflowing_op!(try_pow, checked_pow, u32, "**");

                fn try_div(self, other: Self) -> Result<Self, DivisionByZeroError<Self>> {
                    self.checked_div(other).ok_or(DivisionByZeroError(self))
                }

                fn try_div_euclid(self, other: Self) -> Result<Self, DivisionByZeroError<Self>> {
                    self.checked_div_euclid(other).ok_or(DivisionByZeroError(self))
                }

                fn try_rem(self, other: Self) -> Result<Self, DivisionByZeroError<Self>> {
                    self.checked_rem(other).ok_or(DivisionByZeroError(self))
                }

                fn try_rem_euclid(self, other: Self) -> Result<Self, DivisionByZeroError<Self>> {
                    self.checked_rem_euclid(other).ok_or(DivisionByZeroError(self))
                }

                fn try_shl(self, other: u32) -> Result<Self, BigShiftError<Self>> {
                    self.checked_shl(other).ok_or(BigShiftError {
                        left: self,
                        op: "<<",
                        right: other,
                    })
                }

                fn try_shr(self, other: u32) -> Result<Self, BigShiftError<Self>> {
                    self.checked_shr(other).ok_or(BigShiftError {
                        left: self,
                        op: ">>",
                        right: other,
                    })
                }
            }
        )*
    }
}

impl_arith_op!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::ArithmeticTryOps;

    #[test]
    fn add() {
        assert!(255u8.try_add(1).is_err());
    }
}
