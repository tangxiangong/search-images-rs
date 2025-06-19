pub mod api;
pub mod configration;
pub mod database;
pub mod error;
pub mod model;
pub mod response;
pub mod service;
pub mod state;
pub mod utils;

// macros
#[macro_export]
macro_rules! impl_from_axum_error {
    ($($error:ty),+ $(,)?) => {
        $(
            impl From<$error> for AppError {
            fn from(e: $error) -> Self {
                Self::new(e.status(), e.to_string())
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_into_internal_error {
    ($($error:ty),+ $(,)?) => {
        $(
            impl From<$error> for AppError {
            fn from(e: $error) -> Self {
                Self::internal(e.to_string())
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_into_bad_request_error {
    ($($error:ty),+ $(,)?) => {
        $(
            impl From<$error> for AppError {
            fn from(e: $error) -> Self {
                Self::bad_request(e.to_string())
                }
            }
        )+
    };
}
