//! Type-safe OData filter builder for API queries.
//!
//! This module provides a fluent API for building OData filter expressions
//! with compile-time type safety and automatic string escaping.
//!
//! # Example
//!
//! ```
//! use spiris::query::Filter;
//!
//! // Simple equality filter
//! let filter = Filter::field("IsActive").eq(true);
//! assert_eq!(filter.to_string(), "IsActive eq true");
//!
//! // Combined filters with AND/OR
//! let filter = Filter::field("IsActive").eq(true)
//!     .and(Filter::field("Name").contains("Corp"));
//! assert_eq!(filter.to_string(), "(IsActive eq true) and (contains(Name, 'Corp'))");
//!
//! // String values are automatically escaped
//! let filter = Filter::field("Name").eq("O'Brien & Co");
//! assert_eq!(filter.to_string(), "Name eq 'O''Brien & Co'");
//! ```

use std::fmt;

/// A filter expression for OData queries.
///
/// Filters can be combined using `and()` and `or()` methods, and negated using `not()`.
#[derive(Debug, Clone)]
pub struct Filter {
    expression: String,
}

impl Filter {
    /// Start building a filter for a specific field.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("CustomerNumber").eq("CUST001");
    /// ```
    pub fn field(name: &str) -> FieldFilter {
        FieldFilter {
            field: name.to_string(),
        }
    }

    /// Create a filter from a raw OData expression string.
    ///
    /// Use this for complex expressions not supported by the builder.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::raw("year(InvoiceDate) eq 2024");
    /// ```
    pub fn raw(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    /// Combine this filter with another using AND.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("IsActive").eq(true)
    ///     .and(Filter::field("Country").eq("SE"));
    /// ```
    pub fn and(self, other: Filter) -> Filter {
        Filter {
            expression: format!("({}) and ({})", self.expression, other.expression),
        }
    }

    /// Combine this filter with another using OR.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Status").eq("Draft")
    ///     .or(Filter::field("Status").eq("Pending"));
    /// ```
    pub fn or(self, other: Filter) -> Filter {
        Filter {
            expression: format!("({}) or ({})", self.expression, other.expression),
        }
    }

    /// Negate this filter.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("IsActive").eq(true).not();
    /// assert_eq!(filter.to_string(), "not (IsActive eq true)");
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Filter {
        Filter {
            expression: format!("not ({})", self.expression),
        }
    }

    /// Get the OData filter expression as a string.
    pub fn as_str(&self) -> &str {
        &self.expression
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}

impl From<Filter> for String {
    fn from(filter: Filter) -> Self {
        filter.expression
    }
}

/// Builder for field-level filter operations.
///
/// Created by `Filter::field()`.
#[derive(Debug, Clone)]
pub struct FieldFilter {
    field: String,
}

impl FieldFilter {
    /// Filter where field equals value.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("IsActive").eq(true);
    /// assert_eq!(filter.to_string(), "IsActive eq true");
    ///
    /// let filter = Filter::field("Name").eq("Acme Corp");
    /// assert_eq!(filter.to_string(), "Name eq 'Acme Corp'");
    /// ```
    pub fn eq<T: FilterValue>(self, value: T) -> Filter {
        Filter {
            expression: format!("{} eq {}", self.field, value.to_odata()),
        }
    }

    /// Filter where field does not equal value.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Status").ne("Cancelled");
    /// assert_eq!(filter.to_string(), "Status ne 'Cancelled'");
    /// ```
    pub fn ne<T: FilterValue>(self, value: T) -> Filter {
        Filter {
            expression: format!("{} ne {}", self.field, value.to_odata()),
        }
    }

    /// Filter where field is greater than value.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("TotalAmount").gt(1000.0);
    /// assert_eq!(filter.to_string(), "TotalAmount gt 1000");
    /// ```
    pub fn gt<T: FilterValue>(self, value: T) -> Filter {
        Filter {
            expression: format!("{} gt {}", self.field, value.to_odata()),
        }
    }

    /// Filter where field is greater than or equal to value.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Quantity").ge(10);
    /// assert_eq!(filter.to_string(), "Quantity ge 10");
    /// ```
    pub fn ge<T: FilterValue>(self, value: T) -> Filter {
        Filter {
            expression: format!("{} ge {}", self.field, value.to_odata()),
        }
    }

    /// Filter where field is less than value.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Price").lt(100.0);
    /// assert_eq!(filter.to_string(), "Price lt 100");
    /// ```
    pub fn lt<T: FilterValue>(self, value: T) -> Filter {
        Filter {
            expression: format!("{} lt {}", self.field, value.to_odata()),
        }
    }

    /// Filter where field is less than or equal to value.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Age").le(65);
    /// assert_eq!(filter.to_string(), "Age le 65");
    /// ```
    pub fn le<T: FilterValue>(self, value: T) -> Filter {
        Filter {
            expression: format!("{} le {}", self.field, value.to_odata()),
        }
    }

    /// Filter where field contains the substring.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Name").contains("Corp");
    /// assert_eq!(filter.to_string(), "contains(Name, 'Corp')");
    /// ```
    pub fn contains(self, value: &str) -> Filter {
        Filter {
            expression: format!("contains({}, '{}')", self.field, escape_string(value)),
        }
    }

    /// Filter where field starts with the prefix.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("CustomerNumber").starts_with("CUST");
    /// assert_eq!(filter.to_string(), "startswith(CustomerNumber, 'CUST')");
    /// ```
    pub fn starts_with(self, value: &str) -> Filter {
        Filter {
            expression: format!("startswith({}, '{}')", self.field, escape_string(value)),
        }
    }

    /// Filter where field ends with the suffix.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Email").ends_with("@example.com");
    /// assert_eq!(filter.to_string(), "endswith(Email, '@example.com')");
    /// ```
    pub fn ends_with(self, value: &str) -> Filter {
        Filter {
            expression: format!("endswith({}, '{}')", self.field, escape_string(value)),
        }
    }

    /// Filter where field is null.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Email").is_null();
    /// assert_eq!(filter.to_string(), "Email eq null");
    /// ```
    pub fn is_null(self) -> Filter {
        Filter {
            expression: format!("{} eq null", self.field),
        }
    }

    /// Filter where field is not null.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::query::Filter;
    ///
    /// let filter = Filter::field("Email").is_not_null();
    /// assert_eq!(filter.to_string(), "Email ne null");
    /// ```
    pub fn is_not_null(self) -> Filter {
        Filter {
            expression: format!("{} ne null", self.field),
        }
    }
}

/// Trait for values that can be used in OData filter expressions.
pub trait FilterValue {
    /// Convert the value to its OData string representation.
    fn to_odata(&self) -> String;
}

impl FilterValue for bool {
    fn to_odata(&self) -> String {
        self.to_string()
    }
}

impl FilterValue for &str {
    fn to_odata(&self) -> String {
        format!("'{}'", escape_string(self))
    }
}

impl FilterValue for String {
    fn to_odata(&self) -> String {
        format!("'{}'", escape_string(self))
    }
}

impl FilterValue for &String {
    fn to_odata(&self) -> String {
        format!("'{}'", escape_string(self))
    }
}

impl FilterValue for i32 {
    fn to_odata(&self) -> String {
        self.to_string()
    }
}

impl FilterValue for i64 {
    fn to_odata(&self) -> String {
        self.to_string()
    }
}

impl FilterValue for u32 {
    fn to_odata(&self) -> String {
        self.to_string()
    }
}

impl FilterValue for u64 {
    fn to_odata(&self) -> String {
        self.to_string()
    }
}

impl FilterValue for f32 {
    fn to_odata(&self) -> String {
        // Format without unnecessary decimal places
        if self.fract() == 0.0 {
            format!("{:.0}", self)
        } else {
            self.to_string()
        }
    }
}

impl FilterValue for f64 {
    fn to_odata(&self) -> String {
        // Format without unnecessary decimal places
        if self.fract() == 0.0 {
            format!("{:.0}", self)
        } else {
            self.to_string()
        }
    }
}

impl FilterValue for chrono::DateTime<chrono::Utc> {
    fn to_odata(&self) -> String {
        // OData date-time format
        self.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }
}

impl FilterValue for chrono::NaiveDate {
    fn to_odata(&self) -> String {
        self.format("%Y-%m-%d").to_string()
    }
}

/// Escape a string for use in OData expressions.
/// Single quotes are escaped by doubling them.
fn escape_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_bool() {
        let filter = Filter::field("IsActive").eq(true);
        assert_eq!(filter.to_string(), "IsActive eq true");
    }

    #[test]
    fn test_eq_string() {
        let filter = Filter::field("Name").eq("Acme");
        assert_eq!(filter.to_string(), "Name eq 'Acme'");
    }

    #[test]
    fn test_eq_string_with_quotes() {
        let filter = Filter::field("Name").eq("O'Brien");
        assert_eq!(filter.to_string(), "Name eq 'O''Brien'");
    }

    #[test]
    fn test_ne() {
        let filter = Filter::field("Status").ne("Cancelled");
        assert_eq!(filter.to_string(), "Status ne 'Cancelled'");
    }

    #[test]
    fn test_gt_number() {
        let filter = Filter::field("Amount").gt(1000);
        assert_eq!(filter.to_string(), "Amount gt 1000");
    }

    #[test]
    fn test_ge() {
        let filter = Filter::field("Quantity").ge(10);
        assert_eq!(filter.to_string(), "Quantity ge 10");
    }

    #[test]
    fn test_lt() {
        let filter = Filter::field("Price").lt(100.5);
        assert_eq!(filter.to_string(), "Price lt 100.5");
    }

    #[test]
    fn test_le() {
        let filter = Filter::field("Age").le(65);
        assert_eq!(filter.to_string(), "Age le 65");
    }

    #[test]
    fn test_contains() {
        let filter = Filter::field("Name").contains("Corp");
        assert_eq!(filter.to_string(), "contains(Name, 'Corp')");
    }

    #[test]
    fn test_starts_with() {
        let filter = Filter::field("Code").starts_with("ABC");
        assert_eq!(filter.to_string(), "startswith(Code, 'ABC')");
    }

    #[test]
    fn test_ends_with() {
        let filter = Filter::field("Email").ends_with("@test.com");
        assert_eq!(filter.to_string(), "endswith(Email, '@test.com')");
    }

    #[test]
    fn test_is_null() {
        let filter = Filter::field("Email").is_null();
        assert_eq!(filter.to_string(), "Email eq null");
    }

    #[test]
    fn test_is_not_null() {
        let filter = Filter::field("Email").is_not_null();
        assert_eq!(filter.to_string(), "Email ne null");
    }

    #[test]
    fn test_and() {
        let filter = Filter::field("IsActive").eq(true).and(Filter::field("Country").eq("SE"));
        assert_eq!(
            filter.to_string(),
            "(IsActive eq true) and (Country eq 'SE')"
        );
    }

    #[test]
    fn test_or() {
        let filter = Filter::field("Status")
            .eq("Draft")
            .or(Filter::field("Status").eq("Pending"));
        assert_eq!(
            filter.to_string(),
            "(Status eq 'Draft') or (Status eq 'Pending')"
        );
    }

    #[test]
    fn test_not() {
        let filter = Filter::field("IsActive").eq(true).not();
        assert_eq!(filter.to_string(), "not (IsActive eq true)");
    }

    #[test]
    fn test_complex_expression() {
        let filter = Filter::field("IsActive")
            .eq(true)
            .and(
                Filter::field("Country")
                    .eq("SE")
                    .or(Filter::field("Country").eq("NO")),
            );
        assert_eq!(
            filter.to_string(),
            "(IsActive eq true) and ((Country eq 'SE') or (Country eq 'NO'))"
        );
    }

    #[test]
    fn test_raw_filter() {
        let filter = Filter::raw("year(InvoiceDate) eq 2024");
        assert_eq!(filter.to_string(), "year(InvoiceDate) eq 2024");
    }

    #[test]
    fn test_float_formatting() {
        let filter = Filter::field("Amount").gt(1000.0);
        assert_eq!(filter.to_string(), "Amount gt 1000");

        let filter = Filter::field("Amount").gt(1000.50);
        assert_eq!(filter.to_string(), "Amount gt 1000.5");
    }

    #[test]
    fn test_into_string() {
        let filter = Filter::field("IsActive").eq(true);
        let s: String = filter.into();
        assert_eq!(s, "IsActive eq true");
    }
}
