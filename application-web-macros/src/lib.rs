mod nest;
mod route;

use proc_macro::TokenStream;

/// Creates resource handler, allowing multiple HTTP method guards.
///
/// # Syntax
/// ```plain
/// #[route("path", method="HTTP_METHOD"[, attributes])]
/// ```
///
/// # Attributes
/// - `"path"`: Raw literal string with path for which to register handler.
/// - `method = "HTTP_METHOD"`: Registers HTTP method to provide guard for. Upper-case string,
///   "GET", "POST" for example.
///
/// # Examples
/// ```
/// # use application_web_macros::route;
/// #[route("/test", method = "GET", method = "HEAD")]
/// async fn example() -> impl IntoResponse {
///     "hello world"
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    route::with_method(None, args, input)
}

/// Creates resource handler, allowing multiple HTTP methods and paths.
///
/// # Syntax
/// ```plain
/// #[routes]
/// #[<method>("path", ...)]
/// #[<method>("path", ...)]
/// ...
/// ```
///
/// # Attributes
/// The `routes` macro itself has no parameters, but allows specifying the attribute macros for
/// the multiple paths and/or methods, e.g. [`GET`](macro@get) and [`POST`](macro@post).
///
/// These helper attributes take the same parameters as the [single method handlers](crate#single-method-handler).
///
/// # Examples
/// ```
/// # use application_web_macros::routes;
/// #[routes]
/// #[get("/test")]
/// #[get("/test2")]
/// #[delete("/test")]
/// async fn example() -> impl IntoResponse {
///     "hello world"
/// }
/// ```
#[proc_macro_attribute]
pub fn routes(_: TokenStream, input: TokenStream) -> TokenStream {
    route::with_methods(input)
}

macro_rules! method_macro {
    ($variant:ident, $method:ident) => {
        ///
        /// # Syntax
        /// ```plain
        #[doc = concat!("#[", stringify!($method), r#"("path"[, attributes])]"#)]
        /// # use spring_web::axum::response::IntoResponse;
        #[doc = concat!("# use spring_macros::", stringify!($method), ";")]
        #[doc = concat!("#[", stringify!($method), r#"("/")]"#)]
        /// async fn example() -> impl IntoResponse {
        ///     "hello world"
        /// }
        /// ```
        #[proc_macro_attribute]
        pub fn $method(args: TokenStream, input: TokenStream) -> TokenStream {
            route::with_method(Some(route::Method::$variant), args, input)
        }
    };
}

method_macro!(Get, get);
method_macro!(Post, post);
method_macro!(Put, put);
method_macro!(Delete, delete);
method_macro!(Head, head);
method_macro!(Options, options);
method_macro!(Trace, trace);
method_macro!(Patch, patch);

/// Prepends a path prefix to all handlers using routing macros inside the attached module.
///
/// # Syntax
///
/// ```
/// # use application_web_macros::nest;
/// #[nest("/prefix")]
/// mod api {
///     // ...
/// }
/// ```
///
/// # Arguments
///
/// - `"/prefix"` - Raw literal string to be prefixed onto contained handlers' paths.
///
/// # Example
///
/// ```
/// # use application_web_macros::{nest, get};
/// #[nest("/api")]
/// mod api {
///     # use super::*;
///     #[get("/hello")]
///     pub async fn hello() -> impl IntoResponse {
///         // this has path /api/hello
///         "Hello, world!"
///     }
/// }
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn nest(args: TokenStream, input: TokenStream) -> TokenStream {
    nest::with_nest(args, input)
}

fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}
