use assets_common::{CssOptions, FileAsset, FileSource};
use quote::{quote, ToTokens};
use syn::{braced, bracketed, parse::Parse};

use crate::add_asset;

#[derive(Default)]
struct FontFamilies {
    families: Vec<String>,
}

impl Parse for FontFamilies {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inside;
        bracketed!(inside in input);
        let array =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_separated_nonempty(
                &inside,
            )?;
        Ok(FontFamilies {
            families: array.into_iter().map(|f| f.value()).collect(),
        })
    }
}

#[derive(Default)]
struct FontWeights {
    weights: Vec<u32>,
}

impl Parse for FontWeights {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inside;
        bracketed!(inside in input);
        let array =
            syn::punctuated::Punctuated::<syn::LitInt, syn::Token![,]>::parse_separated_nonempty(
                &inside,
            )?;
        Ok(FontWeights {
            weights: array
                .into_iter()
                .map(|f| f.base10_parse().unwrap())
                .collect(),
        })
    }
}

struct ParseFontOptions {
    families: FontFamilies,
    weights: FontWeights,
    text: Option<String>,
    display: Option<String>,
}

impl ParseFontOptions {
    fn url(&self) -> String {
        let mut segments = Vec::new();

        let families: Vec<_> = self
            .families
            .families
            .iter()
            .map(|f| f.replace(' ', "+"))
            .collect();
        if !families.is_empty() {
            segments.push(format!("family={}", families.join("&")));
        }

        let weights: Vec<_> = self.weights.weights.iter().map(|w| w.to_string()).collect();
        if !weights.is_empty() {
            segments.push(format!("weight={}", weights.join(",")));
        }

        if let Some(text) = &self.text {
            segments.push(format!("text={}", text.replace(' ', "+")));
        }

        if let Some(display) = &self.display {
            segments.push(format!("display={}", display.replace(' ', "+")));
        }

        let query = if segments.is_empty() {
            String::new()
        } else {
            format!("?{}", segments.join("&"))
        };

        format!("https://fonts.googleapis.com/css2{}", query)
    }
}

impl Parse for ParseFontOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inside;
        braced!(inside in input);
        let mut families = None;
        let mut weights = None;
        let mut text = None;
        let mut display = None;
        loop {
            if inside.is_empty() {
                break;
            }
            let ident = inside.parse::<syn::Ident>()?;
            let _ = inside.parse::<syn::Token![:]>()?;
            match ident.to_string().to_lowercase().as_str() {
                "families" => {
                    families = Some(inside.parse::<FontFamilies>()?);
                }
                "weights" => {
                    weights = Some(inside.parse::<FontWeights>()?);
                }
                "text" => {
                    text = Some(inside.parse::<syn::LitStr>()?.value());
                }
                "display" => {
                    display = Some(inside.parse::<syn::LitStr>()?.value());
                }
                _ => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Unknown font option: {ident}. Supported options are families, weights, text, display"),
                    ))
                }
            }
            let _ = inside.parse::<syn::Token![,]>();
        }

        Ok(ParseFontOptions {
            families: families.unwrap_or_default(),
            weights: weights.unwrap_or_default(),
            text,
            display,
        })
    }
}

pub struct FontAssetParser {
    file_name: String,
}

impl Parse for FontAssetParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let options = input.parse::<ParseFontOptions>()?;

        let url = options.url();
        let url: FileSource = match url.parse() {
            Ok(url) => url,
            Err(e) => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Failed to parse url: {url:?}\n{e}"),
                ))
            }
        };
        let asset = FileAsset::new_with_options(
            url.clone(),
            assets_common::FileOptions::Css(CssOptions::default()),
        );
        match asset {
            Ok( this_file) => {
                let asset = add_asset(assets_common::AssetType::File(this_file.clone()));
                let this_file = match asset {
                    assets_common::AssetType::File(this_file) => this_file,
                    _ => unreachable!(),
                };
                let file_name = this_file.served_location();

                Ok(FontAssetParser {file_name})
            }
            Err(e) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to locate asset: {url:?}\nAny relative paths are resolved relative to the manifest directory\n{e}"),
            ))
        }
    }
}

impl ToTokens for FontAssetParser {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let file_name = &self.file_name;

        tokens.extend(quote! {
            #file_name
        })
    }
}