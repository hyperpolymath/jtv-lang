// Number system implementation supporting 7 types
use crate::ast::{BasicType, Number};
use crate::echo::{carrier_echo, Echo};
use crate::error::{JtvError, Result};
use num_complex::Complex64;
use num_rational::Ratio;
use num_traits::Zero;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Rational(Ratio<i64>),
    Complex(Complex64),
    Hex(i64),
    Binary(i64),
    Symbolic(String),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    /// v2 — an opaque, linearly-consumed reversal token (handle into the
    /// interpreter's token store). Not a number: arithmetic and comparison
    /// against it fall through to the existing `TypeError` arms.
    ReversalToken(u64),
    Unit,
}

// ============================================================================
// NUMBER-SYSTEM STRATIFICATION — runtime-value bridge to Echo (ADR-0010)
// ============================================================================
// The runtime-value counterpart of the type-level `echo::carrier_echo`
// (`BasicType -> Echo`) and the Lean `JtvEcho.lean` SECTION 6 stratification:
// a runtime `Value`'s reversal tier is forced by the additive algebra of its
// number system. The algebra -> tier map is NOT duplicated here — it is taken
// from `echo::carrier_echo`, so there is a single source of truth.

impl Value {
    /// The number system this runtime value belongs to, or `None` for the
    /// non-numeric carriers (bool / string / list / tuple / token / unit) —
    /// which have no additive algebra and so cannot be `+=` targets.
    pub fn number_system(&self) -> Option<BasicType> {
        Some(match self {
            Value::Int(_) => BasicType::Int,
            Value::Float(_) => BasicType::Float,
            Value::Rational(_) => BasicType::Rational,
            Value::Complex(_) => BasicType::Complex,
            Value::Hex(_) => BasicType::Hex,
            Value::Binary(_) => BasicType::Binary,
            Value::Symbolic(_) => BasicType::Symbolic,
            Value::Bool(_)
            | Value::String(_)
            | Value::List(_)
            | Value::Tuple(_)
            | Value::ReversalToken(_)
            | Value::Unit => return None,
        })
    }

    /// The Echo reversal tier forced by this value's additive algebra — the
    /// runtime-value counterpart of `echo::carrier_echo` and `JtvEcho.lean`
    /// SECTION 6 (`abelianGroup -> Safe`, `approxGroup -> Neutral`,
    /// `nonGroup -> Breaking`). A non-numeric value induces no additive-reversal
    /// obligation, hence `Safe`.
    pub fn reversal_echo(&self) -> Echo {
        match self.number_system() {
            Some(ty) => carrier_echo(&ty),
            None => Echo::Safe,
        }
    }
}

impl Value {
    // Addition operation (the only arithmetic operation in Data Language)
    pub fn add(&self, other: &Value) -> Result<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_add(*b)
                .map(Value::Int)
                .ok_or(JtvError::IntegerOverflow),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a + b)),
            (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a + b)),
            (Value::Hex(a), Value::Hex(b)) => a
                .checked_add(*b)
                .map(Value::Hex)
                .ok_or(JtvError::IntegerOverflow),
            (Value::Binary(a), Value::Binary(b)) => a
                .checked_add(*b)
                .map(Value::Binary)
                .ok_or(JtvError::IntegerOverflow),
            (Value::Symbolic(a), Value::Symbolic(b)) => {
                Ok(Value::Symbolic(format!("{} + {}", a, b)))
            }
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            // Type coercion
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Int(a), Value::Rational(b)) => Ok(Value::Rational(Ratio::from_integer(*a) + b)),
            (Value::Rational(a), Value::Int(b)) => Ok(Value::Rational(a + Ratio::from_integer(*b))),
            (Value::Float(a), Value::Complex(b)) => Ok(Value::Complex(Complex64::new(*a, 0.0) + b)),
            (Value::Complex(a), Value::Float(b)) => Ok(Value::Complex(a + Complex64::new(*b, 0.0))),
            _ => Err(JtvError::TypeError(format!(
                "Cannot add {:?} and {:?}",
                self, other
            ))),
        }
    }

    // Negation operation (safe: handles i64::MIN overflow)
    pub fn negate(&self) -> Result<Value> {
        match self {
            Value::Int(n) => n
                .checked_neg()
                .map(Value::Int)
                .ok_or(JtvError::IntegerOverflow),
            Value::Float(n) => Ok(Value::Float(-n)),
            Value::Rational(n) => Ok(Value::Rational(-n)),
            Value::Complex(n) => Ok(Value::Complex(-n)),
            Value::Hex(n) => n
                .checked_neg()
                .map(Value::Hex)
                .ok_or(JtvError::IntegerOverflow),
            Value::Binary(n) => n
                .checked_neg()
                .map(Value::Binary)
                .ok_or(JtvError::IntegerOverflow),
            Value::Symbolic(s) => Ok(Value::Symbolic(format!("-({})", s))),
            _ => Err(JtvError::TypeError(format!("Cannot negate {:?}", self))),
        }
    }

    // Comparison operations (for Control expressions)
    pub fn eq(&self, other: &Value) -> Result<bool> {
        Ok(self == other)
    }

    pub fn ne(&self, other: &Value) -> Result<bool> {
        Ok(self != other)
    }

    pub fn lt(&self, other: &Value) -> Result<bool> {
        match (self, other) {
            // Same types
            (Value::Int(a), Value::Int(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            (Value::Rational(a), Value::Rational(b)) => Ok(a < b),
            (Value::Hex(a), Value::Hex(b)) => Ok(a < b),
            (Value::Binary(a), Value::Binary(b)) => Ok(a < b),
            // Cross-type: Int with other numeric types
            (Value::Int(a), Value::Float(b)) => Ok((*a as f64) < *b),
            (Value::Float(a), Value::Int(b)) => Ok(*a < (*b as f64)),
            (Value::Int(a), Value::Rational(b)) => Ok(Ratio::from_integer(*a) < *b),
            (Value::Rational(a), Value::Int(b)) => Ok(*a < Ratio::from_integer(*b)),
            (Value::Hex(a), Value::Int(b)) | (Value::Int(b), Value::Hex(a)) => Ok(a < b),
            (Value::Binary(a), Value::Int(b)) | (Value::Int(b), Value::Binary(a)) => Ok(a < b),
            _ => Err(JtvError::TypeError(format!(
                "Cannot compare {:?} and {:?}",
                self, other
            ))),
        }
    }

    pub fn le(&self, other: &Value) -> Result<bool> {
        match (self, other) {
            // Same types
            (Value::Int(a), Value::Int(b)) => Ok(a <= b),
            (Value::Float(a), Value::Float(b)) => Ok(a <= b),
            (Value::Rational(a), Value::Rational(b)) => Ok(a <= b),
            (Value::Hex(a), Value::Hex(b)) => Ok(a <= b),
            (Value::Binary(a), Value::Binary(b)) => Ok(a <= b),
            // Cross-type: Int with other numeric types
            (Value::Int(a), Value::Float(b)) => Ok((*a as f64) <= *b),
            (Value::Float(a), Value::Int(b)) => Ok(*a <= (*b as f64)),
            (Value::Int(a), Value::Rational(b)) => Ok(Ratio::from_integer(*a) <= *b),
            (Value::Rational(a), Value::Int(b)) => Ok(*a <= Ratio::from_integer(*b)),
            (Value::Hex(a), Value::Int(b)) | (Value::Int(b), Value::Hex(a)) => Ok(a <= b),
            (Value::Binary(a), Value::Int(b)) | (Value::Int(b), Value::Binary(a)) => Ok(a <= b),
            _ => Err(JtvError::TypeError(format!(
                "Cannot compare {:?} and {:?}",
                self, other
            ))),
        }
    }

    pub fn gt(&self, other: &Value) -> Result<bool> {
        match (self, other) {
            // Same types
            (Value::Int(a), Value::Int(b)) => Ok(a > b),
            (Value::Float(a), Value::Float(b)) => Ok(a > b),
            (Value::Rational(a), Value::Rational(b)) => Ok(a > b),
            (Value::Hex(a), Value::Hex(b)) => Ok(a > b),
            (Value::Binary(a), Value::Binary(b)) => Ok(a > b),
            // Cross-type: Int with other numeric types
            (Value::Int(a), Value::Float(b)) => Ok((*a as f64) > *b),
            (Value::Float(a), Value::Int(b)) => Ok(*a > (*b as f64)),
            (Value::Int(a), Value::Rational(b)) => Ok(Ratio::from_integer(*a) > *b),
            (Value::Rational(a), Value::Int(b)) => Ok(*a > Ratio::from_integer(*b)),
            (Value::Hex(a), Value::Int(b)) | (Value::Int(b), Value::Hex(a)) => Ok(a > b),
            (Value::Binary(a), Value::Int(b)) | (Value::Int(b), Value::Binary(a)) => Ok(a > b),
            _ => Err(JtvError::TypeError(format!(
                "Cannot compare {:?} and {:?}",
                self, other
            ))),
        }
    }

    pub fn ge(&self, other: &Value) -> Result<bool> {
        match (self, other) {
            // Same types
            (Value::Int(a), Value::Int(b)) => Ok(a >= b),
            (Value::Float(a), Value::Float(b)) => Ok(a >= b),
            (Value::Rational(a), Value::Rational(b)) => Ok(a >= b),
            (Value::Hex(a), Value::Hex(b)) => Ok(a >= b),
            (Value::Binary(a), Value::Binary(b)) => Ok(a >= b),
            // Cross-type: Int with other numeric types
            (Value::Int(a), Value::Float(b)) => Ok((*a as f64) >= *b),
            (Value::Float(a), Value::Int(b)) => Ok(*a >= (*b as f64)),
            (Value::Int(a), Value::Rational(b)) => Ok(Ratio::from_integer(*a) >= *b),
            (Value::Rational(a), Value::Int(b)) => Ok(*a >= Ratio::from_integer(*b)),
            (Value::Hex(a), Value::Int(b)) | (Value::Int(b), Value::Hex(a)) => Ok(a >= b),
            (Value::Binary(a), Value::Int(b)) | (Value::Int(b), Value::Binary(a)) => Ok(a >= b),
            _ => Err(JtvError::TypeError(format!(
                "Cannot compare {:?} and {:?}",
                self, other
            ))),
        }
    }

    // Conversion from AST Number to Value
    pub fn from_number(num: &Number) -> Result<Value> {
        match num {
            Number::Int(n) => Ok(Value::Int(*n)),
            Number::Float(n) => Ok(Value::Float(*n)),
            Number::Rational(num, den) => {
                if *den == 0 {
                    Err(JtvError::DivisionByZero)
                } else {
                    Ok(Value::Rational(Ratio::new(*num, *den)))
                }
            }
            Number::Complex(real, imag) => Ok(Value::Complex(Complex64::new(*real, *imag))),
            Number::Hex(s) => {
                let without_prefix = s.trim_start_matches("0x");
                let n = i64::from_str_radix(without_prefix, 16)
                    .map_err(|e| JtvError::ParseError(format!("Invalid hex: {}", e)))?;
                Ok(Value::Hex(n))
            }
            Number::Binary(s) => {
                let without_prefix = s.trim_start_matches("0b");
                let n = i64::from_str_radix(without_prefix, 2)
                    .map_err(|e| JtvError::ParseError(format!("Invalid binary: {}", e)))?;
                Ok(Value::Binary(n))
            }
            Number::Symbolic(s) => Ok(Value::Symbolic(s.clone())),
        }
    }

    // Check if value is truthy (for conditionals)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Rational(r) => !r.is_zero(),
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Unit => false,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Rational(r) => write!(f, "{}/{}", r.numer(), r.denom()),
            Value::Complex(c) => {
                if c.im >= 0.0 {
                    write!(f, "{}+{}i", c.re, c.im)
                } else {
                    write!(f, "{}{}i", c.re, c.im)
                }
            }
            Value::Hex(n) => write!(f, "0x{:x}", n),
            Value::Binary(n) => write!(f, "0b{:b}", n),
            Value::Symbolic(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::ReversalToken(id) => write!(f, "<reversal-token #{}>", id),
            Value::Unit => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_addition() {
        let a = Value::Int(5);
        let b = Value::Int(3);
        let result = a.add(&b).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_float_addition() {
        let a = Value::Float(5.5);
        let b = Value::Float(3.2);
        let result = a.add(&b).unwrap();
        assert!(matches!(result, Value::Float(_)));
    }

    #[test]
    fn test_rational_addition() {
        let a = Value::Rational(Ratio::new(1, 2));
        let b = Value::Rational(Ratio::new(1, 3));
        let result = a.add(&b).unwrap();
        assert_eq!(result, Value::Rational(Ratio::new(5, 6)));
    }

    #[test]
    fn test_complex_addition() {
        let a = Value::Complex(Complex64::new(1.0, 2.0));
        let b = Value::Complex(Complex64::new(3.0, 4.0));
        let result = a.add(&b).unwrap();
        assert_eq!(result, Value::Complex(Complex64::new(4.0, 6.0)));
    }

    #[test]
    fn test_hex_addition() {
        let a = Value::Hex(0x10);
        let b = Value::Hex(0x20);
        let result = a.add(&b).unwrap();
        assert_eq!(result, Value::Hex(0x30));
    }

    #[test]
    fn test_binary_addition() {
        let a = Value::Binary(0b1010);
        let b = Value::Binary(0b0101);
        let result = a.add(&b).unwrap();
        assert_eq!(result, Value::Binary(0b1111));
    }

    #[test]
    fn test_symbolic_addition() {
        let a = Value::Symbolic("x".to_string());
        let b = Value::Symbolic("y".to_string());
        let result = a.add(&b).unwrap();
        assert_eq!(result, Value::Symbolic("x + y".to_string()));
    }

    #[test]
    fn test_type_coercion() {
        let a = Value::Int(5);
        let b = Value::Float(3.5);
        let result = a.add(&b).unwrap();
        assert!(matches!(result, Value::Float(_)));
    }

    #[test]
    fn value_number_system_maps_carriers() {
        assert_eq!(Value::Int(0).number_system(), Some(BasicType::Int));
        assert_eq!(Value::Float(0.0).number_system(), Some(BasicType::Float));
        assert_eq!(Value::Hex(0).number_system(), Some(BasicType::Hex));
        assert_eq!(Value::Binary(0).number_system(), Some(BasicType::Binary));
        assert_eq!(
            Value::Symbolic("x".to_string()).number_system(),
            Some(BasicType::Symbolic)
        );
        // Non-numeric carriers have no additive algebra.
        assert_eq!(Value::Bool(true).number_system(), None);
        assert_eq!(Value::Unit.number_system(), None);
        assert_eq!(Value::ReversalToken(0).number_system(), None);
    }

    #[test]
    fn value_reversal_echo_mirrors_section6() {
        // Exact abelian groups -> Safe (incl. the ℤ-encodings hex/binary).
        assert_eq!(Value::Int(0).reversal_echo(), Echo::Safe);
        assert_eq!(Value::Hex(0).reversal_echo(), Echo::Safe);
        assert_eq!(Value::Binary(0).reversal_echo(), Echo::Safe);
        assert_eq!(Value::Symbolic("x".to_string()).reversal_echo(), Echo::Safe);
        // Float -> Neutral (non-associative, lossy reverse-add).
        assert_eq!(Value::Float(1.5).reversal_echo(), Echo::Neutral);
        // Non-numeric -> Safe (no additive-reversal obligation).
        assert_eq!(Value::Bool(true).reversal_echo(), Echo::Safe);
        assert_eq!(Value::Unit.reversal_echo(), Echo::Safe);
    }
}
