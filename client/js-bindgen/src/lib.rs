use std::borrow::Cow;
use std::fmt::Display;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

#[proc_macro]
pub fn embed_asm(input: TokenStream) -> TokenStream {
	embed_asm_internal(input).unwrap_or_else(|e| e)
}

fn embed_asm_internal(input: TokenStream) -> Result<TokenStream, TokenStream> {
	let span = Span::mixed_site();
	let mut assembly = String::new();
	let mut input = input.into_iter().peekable();

	while let Some(token) = input.next() {
		match token {
			TokenTree::Literal(lit) => {
				let span = lit.span();
				let lit = lit.to_string();
				let string = parse_string_literal(span, &lit)?;
				assembly.extend([string, Cow::Borrowed("\n")]);

				match input.peek() {
					Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
						input.next();
					}
					Some(token) => return Err(compile_error(token.span(), "expecting a `,`")),
					None => (),
				}
			}
			token => return Err(compile_error(token.span(), "expecting a string literal")),
		}
	}

	if assembly.is_empty() {
		return Err(compile_error(span, "requires at least a string argument"));
	}

	assembly.push('\0');

	Ok(TokenStream::from_iter([
		TokenTree::Ident(Ident::new("const", span)),
		TokenTree::Ident(Ident::new("_", span)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
		TokenTree::Punct(Punct::new('=', Spacing::Alone)),
		TokenTree::Group(Group::new(
			Delimiter::Brace,
			TokenStream::from_iter([
				TokenTree::Punct(Punct::new('#', Spacing::Alone)),
				TokenTree::Group(Group::new(
					Delimiter::Bracket,
					TokenStream::from_iter([
						TokenTree::Ident(Ident::new("link_section", span)),
						TokenTree::Punct(Punct::new('=', Spacing::Alone)),
						TokenTree::Literal(Literal::string("js_bindgen.assembly")),
					]),
				)),
				TokenTree::Ident(Ident::new("static", span)),
				TokenTree::Ident(Ident::new("ASSEMBLY", span)),
				TokenTree::Punct(Punct::new(':', Spacing::Alone)),
				TokenTree::Group(Group::new(
					Delimiter::Bracket,
					TokenStream::from_iter([
						TokenTree::Ident(Ident::new("u8", span)),
						TokenTree::Punct(Punct::new(';', Spacing::Alone)),
						TokenTree::Literal(Literal::usize_unsuffixed(assembly.len())),
					]),
				)),
				TokenTree::Punct(Punct::new('=', Spacing::Alone)),
				TokenTree::Punct(Punct::new('*', Spacing::Alone)),
				TokenTree::Literal(Literal::byte_string(assembly.as_bytes())),
				TokenTree::Punct(Punct::new(';', Spacing::Alone)),
			]),
		)),
		TokenTree::Punct(Punct::new(';', Spacing::Alone)),
	]))
}

fn parse_string_literal(span: Span, lit: &str) -> Result<Cow<'_, str>, TokenStream> {
	// Strip starting and ending `"`.
	let Some(stripped) = lit.strip_prefix('"').and_then(|lit| lit.strip_suffix('"')) else {
		return Err(compile_error(span, "expecting a string literal"));
	};

	// Apply escaping `\`.
	let sanitized = if stripped.contains("\\\\") {
		Cow::Owned(stripped.replace("\\\\", "\\"))
	} else {
		Cow::Borrowed(stripped)
	};
	// Apply escaping `"`.
	let sanitized = if stripped.contains("\\\"") {
		Cow::Owned(sanitized.replace("\\\"", "\""))
	} else {
		sanitized
	};

	// Don't allow escaping anything else.
	if sanitized.contains('\\') {
		return Err(compile_error(span, "only escaping `\"` is supported"));
	}

	// Don't allow null characters, as we use those as delimiters between assembly
	// blocks.
	if sanitized.contains('\0') {
		return Err(compile_error(span, " null characters are not supported"));
	}

	Ok(sanitized)
}

fn compile_error<E: Display>(span: Span, error: E) -> TokenStream {
	TokenStream::from_iter([
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("core", span)),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("compile_error", span)),
		TokenTree::Punct(Punct::new('!', Spacing::Alone)),
		TokenTree::Group(Group::new(
			Delimiter::Parenthesis,
			TokenTree::Literal(Literal::string(&error.to_string())).into(),
		)),
		TokenTree::Punct(Punct::new(';', Spacing::Alone)),
	])
}
