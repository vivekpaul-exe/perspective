////////////////////////////////////////////////////////////////////////////////
//
// Copyright (c) 2018, the Perspective Authors.
//
// This file is part of the Perspective library, distributed under the terms
// of the Apache License 2.0.  The full license can be found in the LICENSE
// file.

/// A macro for implementing the `wasm_bindgen` boilerplate for types which
/// implement `serde::{Serialize, Deserialize}`.
///
/// # Examples
///
/// ```
/// struct MyStruct { .. }
/// derive_wasm_abi!(MyStruct, FromWasmAbi);
///
/// #[wasm_bindgen]
/// pub fn process_my_struct(s: MyStruct) {}
/// ```
#[macro_export]
macro_rules! derive_wasm_abi {
    ($type:ty) => {
        impl wasm_bindgen::describe::WasmDescribe for $type {
            fn describe() {
                <js_sys::Object as wasm_bindgen::describe::WasmDescribe>::describe()
            }
        }
    };

    ($type:ty, FromWasmAbi $(, $symbols:tt)*) => {
        impl wasm_bindgen::convert::FromWasmAbi for $type {
            type Abi = <js_sys::Object as wasm_bindgen::convert::IntoWasmAbi>::Abi;
            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                let obj = js_sys::Object::from_abi(js);
                obj.into_serde().unwrap()
            }
        }

        derive_wasm_abi!($type $(, $symbols)*);
    };

    ($type:ty, IntoWasmAbi $(, $symbols:tt)*) => {
        impl wasm_bindgen::convert::IntoWasmAbi for $type {
            type Abi = <js_sys::Object as wasm_bindgen::convert::IntoWasmAbi>::Abi;
            #[inline]
            fn into_abi(self) -> Self::Abi {
                use wasm_bindgen::JsCast;
                wasm_bindgen::JsValue::from_serde(&self).unwrap().unchecked_into::<js_sys::Object>().into_abi()
            }
        }

        derive_wasm_abi!($type $(, $symbols)*);
    };
}
