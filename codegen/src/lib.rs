use derive_routable::routable_derive_impl;
use syn::parse_macro_input;

mod derive_routable;

/// Derive macro used to mark an enum as Routable.
///
/// This macro can only be used on enums. Every varient of the macro needs to be marked
/// with the `at` attribute to specify the URL of the route. It generates an implementation of
///  `yew_router::Routable` trait and `const`s for the routes passed which are used with `Route`
/// component.
///
/// # Example
///
/// ```
/// # use yew_router::Routable;
/// #[derive(Debug, Clone, Copy, PartialEq, Routable)]
/// enum Routes {
///   #[at("/")]
///   Home,
///   #[at("/secure")]
///   Secure,
///   #[at("/404")]
///   NotFound,
/// }
/// ```
#[proc_macro_derive(Routable, attributes(at, not_found, route_prefix))]
pub fn derive_routable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as crate::derive_routable::Routable);
  routable_derive_impl(input).into()
}
