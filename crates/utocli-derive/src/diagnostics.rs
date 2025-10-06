use std::{
    borrow::Cow,
    error::Error,
    fmt::{self, Display},
    iter::FromIterator,
};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote_spanned};

pub(crate) trait ToTokensDiagnostics {
    fn to_tokens(&self, tokens: &mut TokenStream2) -> Result<(), Diagnostics>;

    #[allow(unused)]
    fn into_token_stream(self) -> TokenStream2
    where
        Self: std::marker::Sized,
    {
        ToTokensDiagnostics::to_token_stream(&self)
    }

    fn to_token_stream(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();
        match ToTokensDiagnostics::to_tokens(self, &mut tokens) {
            Ok(_) => tokens,
            Err(error_stream) => Into::<Diagnostics>::into(error_stream).into_token_stream(),
        }
    }

    #[allow(dead_code)]
    fn try_to_token_stream(&self) -> Result<TokenStream2, Diagnostics> {
        let mut tokens = TokenStream2::new();
        match ToTokensDiagnostics::to_tokens(self, &mut tokens) {
            Ok(_) => Ok(tokens),
            Err(diagnostics) => Err(diagnostics),
        }
    }
}

#[allow(unused_macros)]
macro_rules! as_tokens_or_diagnostics {
    ( $type:expr ) => {{
        let mut _tokens = proc_macro2::TokenStream::new();
        match crate::diagnostics::ToTokensDiagnostics::to_tokens($type, &mut _tokens) {
            Ok(_) => _tokens,
            Err(diagnostics) => return Err(diagnostics),
        }
    }};
}

#[allow(unused_imports)]
pub(crate) use as_tokens_or_diagnostics;

#[derive(Debug)]
pub(crate) struct Diagnostics {
    diagnostics: Vec<DiangosticsInner>,
}

#[derive(Debug)]
struct DiangosticsInner {
    span: Span,
    message: Cow<'static, str>,
    suggestions: Vec<Suggestion>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Suggestion {
    Help(Cow<'static, str>),
    Note(Cow<'static, str>),
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help(help) => {
                let s: &str = help.as_ref();
                write!(f, "help = {}", s)
            }
            Self::Note(note) => {
                let s: &str = note.as_ref();
                write!(f, "note = {}", s)
            }
        }
    }
}

impl Diagnostics {
    fn message(&self) -> Cow<'static, str> {
        self.diagnostics
            .first()
            .map(|diagnostics| diagnostics.message.clone())
            .unwrap_or_else(|| Cow::Borrowed(""))
    }

    pub fn new<S: Into<Cow<'static, str>>>(message: S) -> Self {
        Self::with_span(Span::call_site(), message)
    }

    pub fn with_span<S: Into<Cow<'static, str>>>(span: Span, message: S) -> Self {
        Self {
            diagnostics: vec![DiangosticsInner {
                span,
                message: message.into(),
                suggestions: Vec::new(),
            }],
        }
    }

    pub fn help<S: Into<Cow<'static, str>>>(mut self, help: S) -> Self {
        if let Some(diagnostics) = self.diagnostics.first_mut() {
            diagnostics.suggestions.push(Suggestion::Help(help.into()));
            diagnostics.suggestions.sort();
        }

        self
    }

    pub fn note<S: Into<Cow<'static, str>>>(mut self, note: S) -> Self {
        if let Some(diagnostics) = self.diagnostics.first_mut() {
            diagnostics.suggestions.push(Suggestion::Note(note.into()));
            diagnostics.suggestions.sort();
        }

        self
    }
}

impl From<syn::Error> for Diagnostics {
    fn from(value: syn::Error) -> Self {
        Self::with_span(value.span(), value.to_string())
    }
}

impl From<Diagnostics> for syn::Error {
    fn from(value: Diagnostics) -> Self {
        // Convert diagnostics to syn::Error by generating the error message
        // This is needed for compatibility with syn::Result
        let message = value.message();
        let span = value
            .diagnostics
            .first()
            .map(|d| d.span)
            .unwrap_or_else(Span::call_site);

        let mut error = syn::Error::new(span, message);

        // Add suggestions as notes (syn::Error doesn't distinguish help vs note)
        if let Some(first) = value.diagnostics.first() {
            for suggestion in &first.suggestions {
                match suggestion {
                    Suggestion::Help(help) => {
                        error.combine(syn::Error::new(span, format!("help: {}", help)));
                    }
                    Suggestion::Note(note) => {
                        error.combine(syn::Error::new(span, format!("note: {}", note)));
                    }
                }
            }
        }

        error
    }
}

impl ToTokens for Diagnostics {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for diagnostics in &self.diagnostics {
            let span = diagnostics.span;
            let message: &str = diagnostics.message.as_ref();

            let suggestions = diagnostics
                .suggestions
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n");

            let diagnostics = if !suggestions.is_empty() {
                Cow::Owned(format!("{message}\n\n{suggestions}"))
            } else {
                Cow::Borrowed(message)
            };

            tokens.extend(quote_spanned! {span=>
                ::core::compile_error!(#diagnostics);
            })
        }
    }
}

impl Error for Diagnostics {}

impl FromIterator<Diagnostics> for Option<Diagnostics> {
    fn from_iter<T: IntoIterator<Item = Diagnostics>>(iter: T) -> Self {
        iter.into_iter().reduce(|mut acc, diagnostics| {
            acc.diagnostics.extend(diagnostics.diagnostics);
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_token_stream_with_help_and_note_orders_help_before_note() {
        //* Given
        let diagnostics = Diagnostics::new("this an error")
            .note("you could do this to solve the error")
            .help("try this thing");

        //* When
        let tokens = diagnostics.into_token_stream();

        //* Then
        let expected_tokens = quote::quote!(::core::compile_error!(
            "this an error\n\nhelp = try this thing\nnote = you could do this to solve the error"
        ););

        assert_eq!(
            tokens.to_string(),
            expected_tokens.to_string(),
            "help should come before note in diagnostic output"
        );
    }
}
