#![no_std]

pub use js_sys;

use js_sys::JsValue;

pub mod console {

    use super::*;

    pub fn log(par1: &JsValue) {
        extern "C" {
            #[link_name = "web_sys.console.log"]
            fn log(par1: isize);
        }

        unsafe { log(par1.as_raw()) };
    }
}
