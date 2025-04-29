use crate::api::error::APIResponseErrorDetail;

/// Represents a trait for validating request data.
/// This trait can be implemented by any struct that requires validation for business logic.
/// It provides a method to validate the data and return a result indicating success or failure.
pub trait Validation<T>
where
    T: std::default::Default,
{
    fn validate(&self) -> Result<T, Vec<APIResponseErrorDetail>>;
}
