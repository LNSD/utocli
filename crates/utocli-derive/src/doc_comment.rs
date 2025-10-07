//! Utilities for parsing doc comments into OpenCLI descriptions.

use syn::Attribute;

/// Extract description from doc comment attributes.
///
/// Combines multiple doc comment lines into a single description string.
pub fn parse_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Ok(meta) = attr.meta.require_name_value()
            && let syn::Expr::Lit(expr_lit) = &meta.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            let comment = lit_str.value();

            // Remove leading space that rustdoc adds
            let trimmed = comment.strip_prefix(' ').unwrap_or(&comment);
            docs.push(trimmed.to_string());
        }
    }

    if docs.is_empty() {
        return None;
    }

    Some(docs.join("\n").trim().to_string())
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse_doc_comments_with_single_comment_returns_trimmed_string() {
        //* Given
        let attrs: Vec<Attribute> = vec![parse_quote! { #[doc = " This is a comment"] }];

        //* When
        let result = parse_doc_comments(&attrs);

        //* Then
        assert_eq!(
            result,
            Some("This is a comment".to_string()),
            "should extract and trim single doc comment"
        );
    }

    #[test]
    fn parse_doc_comments_with_multiple_lines_joins_with_newline() {
        //* Given
        let attrs: Vec<Attribute> = vec![
            parse_quote! { #[doc = " Line 1"] },
            parse_quote! { #[doc = " Line 2"] },
        ];

        //* When
        let result = parse_doc_comments(&attrs);

        //* Then
        assert_eq!(
            result,
            Some("Line 1\nLine 2".to_string()),
            "should join multiple doc comments with newlines"
        );
    }

    #[test]
    fn parse_doc_comments_with_no_doc_attrs_returns_none() {
        //* Given
        let attrs: Vec<Attribute> = vec![parse_quote! { #[derive(Debug)] }];

        //* When
        let result = parse_doc_comments(&attrs);

        //* Then
        assert_eq!(
            result, None,
            "should return None when no doc comments are present"
        );
    }
}
