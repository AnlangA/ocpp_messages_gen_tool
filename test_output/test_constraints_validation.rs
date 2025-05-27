// Test file to verify that the generated validation constraints work correctly
use rust_decimal::Decimal;
use std::str::FromStr;
use validator::Validate;

// Include the generated TestConstraints struct
mod test_constraints;
use test_constraints::TestConstraints;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_constraints() {
        let valid_instance = TestConstraints::new(
            "Hello World".to_string(), // 11 chars, between 5-50 ✓
            "This is a long string".to_string(), // 21 chars, >= 10 ✓
            "Short".to_string(), // 5 chars, <= 100 ✓
            vec!["item1".to_string(), "item2".to_string(), "item3".to_string()], // 3 items, between 2-10 ✓
            vec![1, 2], // 2 items, >= 1 ✓
            50, // between 1-100 ✓
            Decimal::from_str("50.5").unwrap(), // between 0.5-99.9 ✓
        );

        // This should pass validation
        assert!(valid_instance.validate().is_ok());
    }

    #[test]
    fn test_string_length_constraints() {
        // Test string too short (min = 5)
        let mut instance = TestConstraints::new(
            "Hi".to_string(), // 2 chars, < 5 ✗
            "This is a long string".to_string(),
            "Short".to_string(),
            vec!["item1".to_string(), "item2".to_string()],
            vec![1],
            50,
            Decimal::from_str("50.5").unwrap(),
        );

        assert!(instance.validate().is_err());

        // Fix the string length
        instance.set_string_with_min_max("Hello".to_string()); // 5 chars, >= 5 ✓
        assert!(instance.validate().is_ok());
    }

    #[test]
    fn test_array_length_constraints() {
        // Test array too small (min = 2)
        let mut instance = TestConstraints::new(
            "Hello World".to_string(),
            "This is a long string".to_string(),
            "Short".to_string(),
            vec!["item1".to_string()], // 1 item, < 2 ✗
            vec![1],
            50,
            Decimal::from_str("50.5").unwrap(),
        );

        assert!(instance.validate().is_err());

        // Fix the array length
        instance.set_array_with_min_max(vec!["item1".to_string(), "item2".to_string()]); // 2 items, >= 2 ✓
        assert!(instance.validate().is_ok());
    }

    #[test]
    fn test_numeric_range_constraints() {
        // Test integer out of range (min = 1, max = 100)
        let mut instance = TestConstraints::new(
            "Hello World".to_string(),
            "This is a long string".to_string(),
            "Short".to_string(),
            vec!["item1".to_string(), "item2".to_string()],
            vec![1],
            0, // < 1 ✗
            Decimal::from_str("50.5").unwrap(),
        );

        assert!(instance.validate().is_err());

        // Fix the integer value
        instance.set_integer_with_range(1); // >= 1 ✓
        assert!(instance.validate().is_ok());
    }

    #[test]
    fn test_decimal_range_constraints() {
        // Test decimal out of range (min = 0.5, max = 99.9)
        let mut instance = TestConstraints::new(
            "Hello World".to_string(),
            "This is a long string".to_string(),
            "Short".to_string(),
            vec!["item1".to_string(), "item2".to_string()],
            vec![1],
            50,
            Decimal::from_str("0.1").unwrap(), // < 0.5 ✗
        );

        assert!(instance.validate().is_err());

        // Fix the decimal value
        instance.set_number_with_range(Decimal::from_str("0.5").unwrap()); // >= 0.5 ✓
        assert!(instance.validate().is_ok());
    }

    #[test]
    fn test_optional_field_constraints() {
        let mut instance = TestConstraints::new(
            "Hello World".to_string(),
            "This is a long string".to_string(),
            "Short".to_string(),
            vec!["item1".to_string(), "item2".to_string()],
            vec![1],
            50,
            Decimal::from_str("50.5").unwrap(),
        );

        // Optional field is None, should be valid
        assert!(instance.validate().is_ok());

        // Set optional field with invalid length (min = 3, max = 20)
        instance.set_optional_field(Some("Hi".to_string())); // 2 chars, < 3 ✗
        assert!(instance.validate().is_err());

        // Fix the optional field
        instance.set_optional_field(Some("Hello".to_string())); // 5 chars, between 3-20 ✓
        assert!(instance.validate().is_ok());
    }
}
