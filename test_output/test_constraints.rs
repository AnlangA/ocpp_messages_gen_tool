use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// TestConstraints message structure.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TestConstraints {
    /// String with both min and max length constraints
    #[validate(length(min = 5, max = 50))]
    pub string_with_min_max: String,

    /// String with only min length constraint
    #[validate(length(min = 10))]
    pub string_with_min_only: String,

    /// String with only max length constraint
    #[validate(length(max = 100))]
    pub string_with_max_only: String,

    /// Array with both min and max items constraints
    #[validate(length(min = 2, max = 10))]
    pub array_with_min_max: Vec<String>,

    /// Array with only min items constraint
    #[validate(length(min = 1))]
    pub array_with_min_only: Vec<i32>,

    /// Integer with min and max constraints
    #[validate(range(min = 1, max = 100))]
    pub integer_with_range: i32,

    /// Number with min and max constraints
    pub number_with_range: Decimal,

    /// Optional field with constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 3, max = 20))]
    pub optional_field: Option<String>,
}

impl TestConstraints {
    /// Creates a new instance of the struct.
    ///
    /// * `string_with_min_max` - String with both min and max length constraints
    /// * `string_with_min_only` - String with only min length constraint
    /// * `string_with_max_only` - String with only max length constraint
    /// * `array_with_min_max` - Array with both min and max items constraints
    /// * `array_with_min_only` - Array with only min items constraint
    /// * `integer_with_range` - Integer with min and max constraints
    /// * `number_with_range` - Number with min and max constraints
    ///
    /// # Returns
    ///
    /// A new instance of the struct with required fields set and optional fields as None.
    pub fn new(string_with_min_max: String, string_with_min_only: String, string_with_max_only: String, array_with_min_max: Vec<String>, array_with_min_only: Vec<i32>, integer_with_range: i32, number_with_range: Decimal) -> Self {
        Self {
            string_with_min_max,
            string_with_min_only,
            string_with_max_only,
            array_with_min_max,
            array_with_min_only,
            integer_with_range,
            number_with_range,
            optional_field: None,
        }
    }

    /// Sets the string_with_min_max field.
    ///
    /// * `string_with_min_max` - String with both min and max length constraints
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_string_with_min_max(&mut self, string_with_min_max: String) -> &mut Self {
        self.string_with_min_max = string_with_min_max;
        self
    }

    /// Sets the string_with_min_only field.
    ///
    /// * `string_with_min_only` - String with only min length constraint
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_string_with_min_only(&mut self, string_with_min_only: String) -> &mut Self {
        self.string_with_min_only = string_with_min_only;
        self
    }

    /// Sets the string_with_max_only field.
    ///
    /// * `string_with_max_only` - String with only max length constraint
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_string_with_max_only(&mut self, string_with_max_only: String) -> &mut Self {
        self.string_with_max_only = string_with_max_only;
        self
    }

    /// Sets the array_with_min_max field.
    ///
    /// * `array_with_min_max` - Array with both min and max items constraints
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_array_with_min_max(&mut self, array_with_min_max: Vec<String>) -> &mut Self {
        self.array_with_min_max = array_with_min_max;
        self
    }

    /// Sets the array_with_min_only field.
    ///
    /// * `array_with_min_only` - Array with only min items constraint
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_array_with_min_only(&mut self, array_with_min_only: Vec<i32>) -> &mut Self {
        self.array_with_min_only = array_with_min_only;
        self
    }

    /// Sets the integer_with_range field.
    ///
    /// * `integer_with_range` - Integer with min and max constraints
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_integer_with_range(&mut self, integer_with_range: i32) -> &mut Self {
        self.integer_with_range = integer_with_range;
        self
    }

    /// Sets the number_with_range field.
    ///
    /// * `number_with_range` - Number with min and max constraints
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_number_with_range(&mut self, number_with_range: Decimal) -> &mut Self {
        self.number_with_range = number_with_range;
        self
    }

    /// Sets the optional_field field.
    ///
    /// * `optional_field` - Optional field with constraints
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn set_optional_field(&mut self, optional_field: Option<String>) -> &mut Self {
        self.optional_field = optional_field;
        self
    }

    /// Gets a reference to the string_with_min_max field.
    ///
    /// # Returns
    ///
    /// String with both min and max length constraints
    pub fn get_string_with_min_max(&self) -> &String {
        &self.string_with_min_max
    }

    /// Gets a reference to the string_with_min_only field.
    ///
    /// # Returns
    ///
    /// String with only min length constraint
    pub fn get_string_with_min_only(&self) -> &String {
        &self.string_with_min_only
    }

    /// Gets a reference to the string_with_max_only field.
    ///
    /// # Returns
    ///
    /// String with only max length constraint
    pub fn get_string_with_max_only(&self) -> &String {
        &self.string_with_max_only
    }

    /// Gets a reference to the array_with_min_max field.
    ///
    /// # Returns
    ///
    /// Array with both min and max items constraints
    pub fn get_array_with_min_max(&self) -> &Vec<String> {
        &self.array_with_min_max
    }

    /// Gets a reference to the array_with_min_only field.
    ///
    /// # Returns
    ///
    /// Array with only min items constraint
    pub fn get_array_with_min_only(&self) -> &Vec<i32> {
        &self.array_with_min_only
    }

    /// Gets a reference to the integer_with_range field.
    ///
    /// # Returns
    ///
    /// Integer with min and max constraints
    pub fn get_integer_with_range(&self) -> &i32 {
        &self.integer_with_range
    }

    /// Gets a reference to the number_with_range field.
    ///
    /// # Returns
    ///
    /// Number with min and max constraints
    pub fn get_number_with_range(&self) -> &Decimal {
        &self.number_with_range
    }

    /// Gets a reference to the optional_field field.
    ///
    /// # Returns
    ///
    /// Optional field with constraints
    pub fn get_optional_field(&self) -> Option<&String> {
        self.optional_field.as_ref()
    }

    /// Sets the optional_field field and returns self for builder pattern.
    ///
    /// * `optional_field` - Optional field with constraints
    ///
    /// # Returns
    ///
    /// Self with the field set.
    pub fn with_optional_field(mut self, optional_field: String) -> Self {
        self.optional_field = Some(optional_field);
        self
    }

}
