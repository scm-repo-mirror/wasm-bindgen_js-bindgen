use proc_macro2::TokenStream;
use quote::quote;

fn test_embed_asm(input: TokenStream, expected: TokenStream) {
	let output = crate::unsafe_embed_asm(input);

	let output = syn::parse2(output).unwrap();
	let output = prettyplease::unparse(&output);
	let expected = syn::parse2(expected).unwrap();
	let expected = prettyplease::unparse(&expected);

	similar_asserts::assert_eq!(expected, output);
}

#[test]
fn minimum() {
	test_embed_asm(
		quote! { "" },
		quote! {
			const _: () = {
				const ARR0: [u8; 1] = *b"\0";

				#[repr(C)]
				struct Layout([u8; 1]);

				#[link_section = "js_bindgen.assembly"]
				static CUSTOM_SECTION: Layout = Layout(ARR0);
			};
		},
	);
}
