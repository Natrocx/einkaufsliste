use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Lit, LitStr, Meta, Variant};

const AT_ATTR_IDENT: &str = "at";
const NOT_FOUND_ATTR_IDENT: &str = "not_found";

const ROUTE_PREFIX_ATTR_IDENT: &str = "route_prefix";

pub struct Routable {
  ident: Ident,
  at_attribute_params: Vec<LitStr>,
  variants: Punctuated<Variant, syn::token::Comma>,
  not_found_route: Option<Ident>,
}

/// Copied 1 to 1 from yew-router
impl Parse for Routable {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let DeriveInput { ident, data, attrs, .. } = input.parse()?;

    let data = match data {
      Data::Enum(data) => data,
      Data::Struct(s) => return Err(syn::Error::new(s.struct_token.span(), "expected enum, found struct")),
      Data::Union(u) => return Err(syn::Error::new(u.union_token.span(), "expected enum, found union")),
    };

    let prefix = parse_struct_attrs(&attrs)?;

    let (not_found_route, ats) = parse_variants_attributes(&data.variants, prefix)?;

    Ok(Self {
      ident,
      variants: data.variants,
      at_attribute_params: ats,
      not_found_route,
    })
  }
}

// Adapted from yew_router 0.16
fn parse_variants_attributes(
  variants: &Punctuated<Variant, syn::token::Comma>,
  prefix: Option<LitStr>,
) -> syn::Result<(Option<Ident>, Vec<LitStr>)> {
  let mut not_founds = vec![];
  let mut at_attribute_params: Vec<LitStr> = vec![];

  let mut not_found_attributes = vec![];

  for variant in variants.iter() {
    if let Fields::Unnamed(ref field) = variant.fields {
      return Err(syn::Error::new(field.span(), "only named fields are supported"));
    }

    let attrs = &variant.attrs;
    let at_attrs = attrs
      .iter()
      .filter(|attr| attr.path.is_ident(AT_ATTR_IDENT))
      .collect::<Vec<_>>();

    let attr = match at_attrs.len() {
      1 => *at_attrs.first().unwrap(),
      0 => {
        return Err(syn::Error::new(
          variant.span(),
          format!("{} attribute must be present on every variant", AT_ATTR_IDENT),
        ))
      }
      _ => {
        return Err(syn::Error::new_spanned(
          quote! { #(#at_attrs)* },
          format!("only one {} attribute must be present", AT_ATTR_IDENT),
        ))
      }
    };

    let lit = attr.parse_args::<LitStr>()?;
    let val = lit.value();

    // handle exceptions regarding URI formation
    if val.find('#').is_some() {
      return Err(syn::Error::new_spanned(
        lit,
        "You cannot use `#` in your routes. Please consider `HashRouter` instead.",
      ));
    }

    //TODO: param over prefix
    if !val.starts_with('/') && prefix.is_none() {
      return Err(syn::Error::new_spanned(
        lit,
        format!(
          "relative paths are not supported at this moment (without a prefix). If you meant to use a prefix, consider \
           using the {ROUTE_PREFIX_ATTR_IDENT} attribute macro."
        ),
      ));
    }

    let lit = match prefix {
      Some(ref prefix) => LitStr::new(&format!("{}{val}", &prefix.value()), lit.span()),
      None => lit,
    };
    at_attribute_params.push(lit);

    for attr in attrs.iter() {
      if attr.path.is_ident(NOT_FOUND_ATTR_IDENT) {
        not_found_attributes.push(attr);
        not_founds.push(variant.ident.clone())
      }
    }
  }

  if not_founds.len() > 1 {
    return Err(syn::Error::new_spanned(
      quote! { #(#not_found_attributes)* },
      format!("there can only be one {}", NOT_FOUND_ATTR_IDENT),
    ));
  }

  Ok((not_founds.into_iter().next(), at_attribute_params))
}

impl Routable {
  fn build_from_path(&self) -> TokenStream {
    let from_path_matches = self.variants.iter().enumerate().map(|(i, variant)| {
      let ident = &variant.ident;
      let right = match &variant.fields {
        Fields::Unit => quote! { Self::#ident },
        Fields::Named(field) => {
          let fields = field.named.iter().map(|it| {
            //named fields have idents
            it.ident.as_ref().unwrap()
          });
          quote! { Self::#ident { #(#fields: params.get(stringify!(#fields))?.parse().ok()?,)* } }
        }
        Fields::Unnamed(_) => unreachable!(), // already checked
      };

      let left = self.at_attribute_params.get(i).unwrap();
      quote! {
          #left => ::std::option::Option::Some(#right)
      }
    });

    quote! {
        fn from_path(path: &str, params: &::std::collections::HashMap<&str, &str>) -> ::std::option::Option<Self> {
            match path {
                #(#from_path_matches),*,
                _ => ::std::option::Option::None,
            }
        }
    }
  }

  fn build_to_path(&self) -> TokenStream {
    let to_path_matches = self.variants.iter().enumerate().map(|(i, variant)| {
      let ident = &variant.ident;
      let mut right = self.at_attribute_params.get(i).unwrap().value();

      match &variant.fields {
        Fields::Unit => quote! { Self::#ident => ::std::string::ToString::to_string(#right) },
        Fields::Named(field) => {
          let fields = field
            .named
            .iter()
            .map(|it| it.ident.as_ref().unwrap())
            .collect::<Vec<_>>();

          for field in fields.iter() {
            // :param -> {param}
            // *param -> {param}
            // so we can pass it to `format!("...", param)`
            right = right.replace(&format!(":{}", field), &format!("{{{}}}", field));
            right = right.replace(&format!("*{}", field), &format!("{{{}}}", field));
          }

          quote! {
              Self::#ident { #(#fields),* } => ::std::format!(#right, #(#fields = #fields),*)
          }
        }
        Fields::Unnamed(_) => unreachable!(), // already checked
      }
    });

    quote! {
        fn to_path(&self) -> ::std::string::String {
            match self {
                #(#to_path_matches),*,
            }
        }
    }
  }
}

pub fn routable_derive_impl(input: Routable) -> TokenStream {
  let Routable {
    at_attribute_params: ats,
    not_found_route,
    ident,
    ..
  } = &input;

  let from_path = input.build_from_path();
  let to_path = input.build_to_path();

  let maybe_not_found_route = match not_found_route {
    Some(route) => quote! { ::std::option::Option::Some(Self::#route) },
    None => quote! { ::std::option::Option::None },
  };

  let maybe_default = match not_found_route {
    Some(route) => {
      quote! {
          impl ::std::default::Default for #ident {
              fn default() -> Self {
                  Self::#route
              }
          }
      }
    }
    None => TokenStream::new(),
  };

  quote! {
      #[automatically_derived]
      impl ::yew_router::Routable for #ident {
          #from_path
          #to_path

          fn routes() -> ::std::vec::Vec<&'static str> {
              ::std::vec![#(#ats),*]
          }

          fn not_found_route() -> ::std::option::Option<Self> {
              #maybe_not_found_route
          }

          fn recognize(pathname: &str) -> ::std::option::Option<Self> {
              ::std::thread_local! {
                  static ROUTER: ::yew_router::__macro::Router = ::yew_router::__macro::build_router::<#ident>();
              }
              ROUTER.with(|router| ::yew_router::__macro::recognize_with_router(router, pathname))
          }
      }

      #maybe_default
  }
}

/**
Parse attributes tagged on the entire struct. Currently this implements the following:

  * Determines prefix for URLs (for Router nesting/not placing the page under the web_root i.e. serving from sub-URIs like `/dev/`). This explicitly serves the purpose of allowing the frontend to be served by the backend as a development environment.
 */
pub fn parse_struct_attrs(attrs: &[Attribute]) -> syn::Result<Option<LitStr>> {
  let full_attribute = attrs
    .iter()
    // we only consider outter attributes
    .filter(|attr| match attr.style {
      syn::AttrStyle::Outer => true,
      syn::AttrStyle::Inner(_) => false,
    })
    .find_map(|attr| attr.path.is_ident(ROUTE_PREFIX_ATTR_IDENT).then(|| attr.parse_meta()));

  match full_attribute {
    // if the attribute is found and correctly formed:
    Some(Ok(Meta::List(list))) => {
      // TODO: recheck path? the rkyv auther does it, but I don't know why
      let prefix = list
        .nested
        .first()
        .map(|token| match token {
          syn::NestedMeta::Lit(Lit::Str(literal)) => Ok(literal),
          _ => Err(syn::Error::new(
            token.span(),
            " Unexpected token: Found an Identifier (meta). Expected a string literal.",
          )),
        })
        .ok_or_else(|| syn::Error::new(list.span(), "You failed to specify a prefix. Exactly one is required."))??;

      Ok(Some(prefix.to_owned()))
    }
    // else: differentiated error handling
    Some(Ok(m)) => Err(syn::Error::new(
      m.span(),
      format!(
        "Incorrect format for route_prefix attribute. You must specify the attribute in List format like so: \
         #[{ROUTE_PREFIX_ATTR_IDENT}(\"/dev/\")]",
      ),
    )),
    Some(Err(e)) => {
      // the user entered a garbage tokentree and it cannot be parsed by syn, so we pass the error through
      Err(e)
    }
    _ => {
      // The user did not supply a route_prefix attribute
      Ok(None)
    }
  }
}
