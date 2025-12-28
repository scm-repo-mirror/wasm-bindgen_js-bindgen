#![no_std]

extern crate alloc;

use core::arch::wasm32;

use web_sys::{console, js_sys};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
	wasm32::unreachable()
}

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[unsafe(no_mangle)]
extern "C" fn foo() {
	console::log(&js_sys::is_nan());
}
