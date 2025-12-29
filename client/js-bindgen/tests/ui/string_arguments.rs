js_bindgen::unsafe_embed_asm!();
//~^ ERROR: requires at least a string argument

js_bindgen::unsafe_embed_asm!(42);
//~^ ERROR: expected a string literal

js_bindgen::unsafe_embed_asm!("", "\r");
//~^ ERROR: escaping `r` is not supported

js_bindgen::unsafe_embed_asm!("" 5);
//~^ ERROR: expected a `,` after string literal
