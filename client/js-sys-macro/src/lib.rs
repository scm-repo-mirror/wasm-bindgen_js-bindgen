#[cfg(test)]
extern crate proc_macro2 as proc_macro;
#[cfg(test)]
use shared as js_bindgen_shared;

// There is currently no way to execute proc-macros in non-proc-macro crates.
// However, we need it for testing. So we somehow have to enable `proc-macro2`,
// even in dependencies. It turns out that this is quite difficult to accomplish
// in dependencies, e.g. via crate features. Including the crate via a module is
// what worked for now. `rust-analyzer` doesn't seem to like `path`s outside the
// crate though, so we added a symlink.
#[cfg(test)]
#[path = "shared/lib.rs"]
mod shared;
#[cfg(test)]
mod test;

use std::borrow::Cow;
#[cfg(not(test))]
use std::env;
use std::iter;

use js_bindgen_shared::*;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

#[cfg_attr(not(test), proc_macro_attribute)]
pub fn js_sys(attr: TokenStream, original_item: TokenStream) -> TokenStream {
	js_sys_internal(attr, original_item.clone()).unwrap_or_else(|mut e| {
		e.extend(original_item);
		e
	})
}

fn js_sys_internal(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
	let mut attr = attr.into_iter().peekable();
	let mut item = item.into_iter().peekable();

	let namespace = if attr.peek().is_some() {
		Some(parse_meta_name_value(&mut attr, "namespace")?)
	} else {
		None
	};

	let r#extern = expect_ident(
		&mut item,
		"extern",
		Span::mixed_site(),
		"`extern \"C\" { ... }` item",
	)?;

	let abi_span = match item.next() {
		Some(TokenTree::Literal(l)) if l.to_string() == "\"C\"" => l.span(),
		Some(tok) => return Err(compile_error(tok.span(), "expected `\"C\"` after `extern`")),
		None => {
			return Err(compile_error(
				r#extern.span(),
				"expected `\"C\"` after `extern`",
			))
		}
	};

	let fns_group = expect_group(&mut item, Delimiter::Brace, abi_span, "braces after ABI")?;
	let mut fns = fns_group.stream().into_iter().peekable();
	let mut output = TokenStream::new();

	while let Some(tok) = fns.peek() {
		let mut real_name = None;

		if let TokenTree::Punct(p) = tok {
			if p.as_char() == '#' {
				let hash = expect_punct(&mut fns, '#', Span::mixed_site(), "`js_sys` attribute")?;
				let meta = expect_group(&mut fns, Delimiter::Bracket, hash.span(), "`[...]`")?;
				let mut inner = meta.stream().into_iter();
				let js_sys = expect_ident(&mut inner, "js_sys", meta.span(), "`js_sys(...)")?;
				let meta =
					expect_group(&mut inner, Delimiter::Parenthesis, js_sys.span(), "`(...)`")?;
				let mut inner = meta.stream().into_iter();
				let name = expect_ident(&mut inner, "name", meta.span(), "`name = \"...\"`")?;
				let equal = expect_punct(&mut inner, '=', name.span(), "`= \"...\"`")?;
				let mut name = String::new();
				parse_string_literal(&mut inner, equal.span(), &mut name)?;
				real_name = Some(name);
			}
		}

		let ExternFn {
			visibility,
			r#fn,
			name,
			parms: parms_group,
			ret_ty,
		} = parse_extern_fn(&mut fns, Span::mixed_site())?;

		let mut parms = Vec::new();
		let mut parms_stream = parms_group.stream().into_iter().peekable();

		while parms_stream.peek().is_some() {
			let name = parse_ident(&mut parms_stream, parms_group.span(), "parameter name")?;
			let colon = expect_punct(
				&mut parms_stream,
				':',
				name.span(),
				"colon after parameter name",
			)?;
			let ty = parse_ty_or_value(&mut parms_stream, colon.span())?;

			if parms_stream.peek().is_some() {
				expect_punct(
					&mut parms_stream,
					',',
					name.span(),
					"`,` after parameter type",
				)?;
			}

			parms.push((name, ty));
		}

		let name_string = name.to_string();
		let comp_name = if let Some(namespace) = &namespace {
			format!("{namespace}.{name_string}")
		} else {
			name_string.clone()
		};

		#[cfg(not(test))]
		let package = env::var("CARGO_CRATE_NAME").expect("`CARGO_CRATE_NAME` not found");
		#[cfg(test)]
		let package = String::from("test_crate");
		let mut comp_parms = String::new();
		let comp_ret = ret_ty.as_ref().map(|_| "{}").unwrap_or_default();

		for (_, _) in &parms {
			if comp_parms.is_empty() {
				comp_parms.push_str("{}");
			} else {
				comp_parms.push_str(", {}");
			}
		}

		let parms_input = parms.iter().enumerate().flat_map(|(index, _)| {
			[
				Cow::Owned(format!("\tlocal.get {index}")),
				Cow::Borrowed("\t{}"),
			]
		});

		let strings = [
			Cow::Owned(format!(
				".import_module {package}.import.{comp_name}, {package}"
			)),
			Cow::Owned(format!(
				".import_name {package}.import.{comp_name}, {comp_name}"
			)),
			Cow::Owned(format!(
				".functype {package}.import.{comp_name} ({comp_parms}) -> ({comp_ret})"
			)),
			Cow::Borrowed(""),
		]
		.into_iter()
		.chain(
			parms
				.iter()
				.flat_map(|_| [Cow::Borrowed("{}"), Cow::Borrowed("")]),
		)
		.chain([
			Cow::Owned(format!(".globl {package}.{comp_name}")),
			Cow::Owned(format!("{package}.{comp_name}:")),
			Cow::Owned(format!(
				"\t.functype {package}.{comp_name} ({comp_parms}) -> ({comp_ret})",
			)),
		])
		.chain(parms_input)
		.chain([
			Cow::Owned(format!("\tcall {package}.import.{comp_name}")),
			Cow::Borrowed("\tend_function"),
		]);

		let import_fmt = parms.iter().flat_map(|(name, _)| {
			js_sys_input("IMPORT_TYPE", name.span())
				.chain(iter::once(Punct::new(',', Spacing::Alone).into()))
		});
		let parms_fmt = parms.iter().flat_map(|(name, _)| {
			js_sys_input("IMPORT_FUNC", name.span())
				.chain(iter::once(Punct::new(',', Spacing::Alone).into()))
		});
		let type_fmt = parms.iter().flat_map(|(name, _)| {
			js_sys_input("TYPE", name.span())
				.chain(iter::once(Punct::new(',', Spacing::Alone).into()))
		});
		let conv_fmt = parms.iter().flat_map(|(name, _)| {
			js_sys_input("CONV", name.span())
				.chain(iter::once(Punct::new(',', Spacing::Alone).into()))
		});

		let assembly = path(["js_sys", "js_bindgen", "unsafe_embed_asm"], name.span()).chain([
			Punct::new('!', Spacing::Alone).into(),
			Group::new(
				Delimiter::Parenthesis,
				TokenStream::from_iter(
					strings
						.flat_map(|string| {
							[
								TokenTree::from(Literal::string(&string)),
								Punct::new(',', Spacing::Alone).into(),
							]
						})
						.chain(import_fmt)
						.chain(parms_fmt)
						.chain(type_fmt)
						.chain(conv_fmt),
				),
			)
			.into(),
			Punct::new(';', Spacing::Alone).into(),
		]);

		let comp_real_name = if let Some(real_name) = real_name {
			if let Some(namespace) = &namespace {
				Cow::Owned(format!("{namespace}.{real_name}"))
			} else {
				Cow::Owned(real_name)
			}
		} else {
			Cow::Borrowed(&comp_name)
		};

		let js_glue = path(["js_sys", "js_bindgen", "js_import"], name.span()).chain([
			Punct::new('!', Spacing::Alone).into(),
			Group::new(
				Delimiter::Parenthesis,
				TokenStream::from_iter([
					TokenTree::from(Ident::new("name", name.span())),
					Punct::new('=', Spacing::Alone).into(),
					Literal::string(&comp_name).into(),
					Punct::new(',', Spacing::Alone).into(),
					Literal::string(&comp_real_name).into(),
				]),
			)
			.into(),
			Punct::new(';', Spacing::Alone).into(),
		]);

		let import_parms = parms.iter().flat_map(|(name, _)| {
			[
				TokenTree::from(name.clone()),
				Punct::new(':', Spacing::Alone).into(),
			]
			.into_iter()
			.chain(js_sys_input("Type", name.span()))
			.chain(iter::once(Punct::new(',', Spacing::Alone).into()))
		});

		let import = [
			TokenTree::from(Ident::new("extern", name.span())),
			Literal::string("C").into(),
			Group::new(
				Delimiter::Brace,
				TokenStream::from_iter([
					TokenTree::from(Punct::new('#', Spacing::Alone)),
					Group::new(
						Delimiter::Bracket,
						TokenStream::from_iter([
							TokenTree::from(Ident::new("link_name", name.span())),
							Punct::new('=', Spacing::Alone).into(),
							Literal::string(&format!("{package}.{comp_name}")).into(),
						]),
					)
					.into(),
					r#fn.clone().into(),
					name.clone().into(),
					Group::new(Delimiter::Parenthesis, import_parms.collect()).into(),
					Punct::new(';', Spacing::Alone).into(),
				]),
			)
			.into(),
		];

		let call_parms = parms.into_iter().flat_map(|(name, _)| {
			js_sys_input("as_raw", name.span()).chain([
				Group::new(
					Delimiter::Parenthesis,
					TokenStream::from_iter(iter::once(TokenTree::from(name))),
				)
				.into(),
				Punct::new(',', Spacing::Alone).into(),
			])
		});

		let call = [
			TokenTree::from(Ident::new("unsafe", name.span())),
			Group::new(
				Delimiter::Brace,
				TokenStream::from_iter([
					TokenTree::from(Ident::new(&name_string, name.span())),
					Group::new(Delimiter::Parenthesis, call_parms.collect()).into(),
				]),
			)
			.into(),
			Punct::new(';', Spacing::Alone).into(),
		];

		output.extend(visibility.map(TokenTree::from));
		output.extend([TokenTree::from(r#fn), name.into(), parms_group.into()]);
		output.extend(ret_ty);
		output.extend(iter::once(TokenTree::from(Group::new(
			Delimiter::Brace,
			TokenStream::from_iter(assembly.chain(js_glue).chain(import).chain(call)),
		))));
	}

	Ok(output)
}

struct ExternFn {
	visibility: Option<Ident>,
	r#fn: Ident,
	name: Ident,
	parms: Group,
	ret_ty: Option<TokenStream>,
}

fn parse_extern_fn(
	mut stream: impl Iterator<Item = TokenTree>,
	span: Span,
) -> Result<ExternFn, TokenStream> {
	let ident = parse_ident(&mut stream, span, "function item")?;
	let ident_span = ident.span();
	let ident_string = ident.to_string();

	let (visibility, r#fn) = match ident_string.as_str() {
		"pub" => (
			Some(ident),
			expect_ident(&mut stream, "fn", ident_span, "function item")?,
		),
		"fn" => (None, ident),
		_ => return Err(compile_error(ident_span, "expected function item")),
	};

	let name = parse_ident(&mut stream, r#fn.span(), "identifier after `fn`")?;

	let parms = expect_group(
		&mut stream,
		Delimiter::Parenthesis,
		name.span(),
		"paranthesis after function identifier",
	)?;

	let punct = parse_punct(&mut stream, parms.span(), "`;` or a return type")?;

	let ret_ty = match punct.as_char() {
		';' => None,
		'-' => {
			let closing = expect_punct(&mut stream, '>', parms.span(), "`->` for the return type")?;

			let mut ret_ty = TokenStream::new();

			loop {
				match stream.next() {
					Some(TokenTree::Punct(p)) if p.as_char() == ';' => break,
					Some(tok) => ret_ty.extend(iter::once(tok)),
					None => return Err(compile_error(closing.span(), "expected `;`")),
				}
			}

			Some(ret_ty)
		}
		_ => return Err(compile_error(parms.span(), "expected `;` or `->`")),
	};

	Ok(ExternFn {
		visibility,
		r#fn,
		name,
		parms,
		ret_ty,
	})
}

fn js_sys_input(field: &'static str, span: Span) -> impl Iterator<Item = TokenTree> {
	iter::once(Punct::new('<', Spacing::Alone).into())
		.chain(path(["js_sys", "JsValue"], span))
		.chain(iter::once(Ident::new("as", span).into()))
		.chain(path(["js_sys", "hazard", "Input"], span))
		.chain(iter::once(Punct::new('>', Spacing::Alone).into()))
		.chain(path(iter::once(field), span))
}

fn parse_punct(
	mut stream: impl Iterator<Item = TokenTree>,
	previous_span: Span,
	expected: &str,
) -> Result<Punct, TokenStream> {
	match stream.next() {
		Some(TokenTree::Punct(p)) => Ok(p),
		Some(tok) => Err(compile_error(tok.span(), format!("expected {expected}"))),
		None => Err(compile_error(previous_span, format!("expected {expected}"))),
	}
}

fn expect_group(
	mut stream: impl Iterator<Item = TokenTree>,
	delimiter: Delimiter,
	previous_span: Span,
	expected: &str,
) -> Result<Group, TokenStream> {
	match stream.next() {
		Some(TokenTree::Group(g)) if g.delimiter() == delimiter => Ok(g),
		Some(tok) => Err(compile_error(tok.span(), format!("expected {expected}"))),
		None => Err(compile_error(previous_span, format!("expected {expected}"))),
	}
}
