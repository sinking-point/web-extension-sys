mod utils {
    use wasm_bindgen::prelude::*;
    use js_sys::{Object, Reflect};
    use crate::error::Error;

    pub fn map_to_js_value<T: Into<JsValue>>(vec: Vec<T>) -> Vec<JsValue> {
        vec
            .into_iter()
            .map(|x| x.into())
            .collect()
    }

    pub fn create_object_with_property<T: Into<JsValue>>(
        key: String,
        value: T,
    ) -> Result<Object, Error> {
        let data = Object::new();
        Reflect::set(&data, &key.into(), &value.into())?;

        Ok(data)
    }
}

pub mod storage {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsValue;
    use js_sys::Reflect;

    pub mod local {
        use wasm_bindgen::prelude::*;
        use crate::utils::{map_to_js_value, create_object_with_property};
        use serde_wasm_bindgen;
        use crate::error::Error;
        use serde::Serialize;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = get)]
            pub fn get_one(key: &str, callback: &Closure<dyn FnMut(JsValue)>);

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = get)]
            fn _get_multiple(keys: Vec<JsValue>, callback: &Closure<dyn FnMut(JsValue)>);

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = set)]
            fn _set(data: JsValue);

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = set)]
            fn _set_and_then(data: JsValue, callback: &Closure<dyn FnMut()>);
        }

        pub fn get_multiple(keys: Vec<String>, callback: &Closure<dyn FnMut(JsValue)>) {
            let keys = map_to_js_value(keys);

            _get_multiple(keys, callback)
        }

        fn _set_optional_callback(data: JsValue, callback: Option<&Closure<dyn FnMut()>>) {
            match callback {
                None => {
                    _set(data);
                }
                Some(c) => {
                    _set_and_then(data, c);
                }
            }
        }

        pub fn set_one<T: Into<JsValue>>(
            key: String,
            value: T,
            callback: Option<&Closure<dyn FnMut()>>
        ) -> Result<(), Error> {
            let data = create_object_with_property(key, value)?;

            _set_optional_callback(data.into(), callback);

            Ok(())
        }

        pub fn set_multiple<T: Serialize>(
            data: T,
            callback: Option<&Closure<dyn FnMut()>>
        ) -> Result<(), Error> {
            _set_optional_callback(serde_wasm_bindgen::to_value(&data)?, callback);

            Ok(())
        }
    }

    pub mod sync {
        use wasm_bindgen::prelude::*;
        use crate::utils::{map_to_js_value, create_object_with_property};
        use serde_wasm_bindgen;
        use crate::error::Error;
        use serde::Serialize;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = ["chrome", "storage", "sync"], js_name = get)]
            pub fn get_one(key: &str, callback: &Closure<dyn FnMut(JsValue)>);

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "sync"], js_name = get)]
            fn _get_multiple(keys: Vec<JsValue>, callback: &Closure<dyn FnMut(JsValue)>);

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "sync"], js_name = set)]
            fn _set(data: JsValue);

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "sync"], js_name = set)]
            fn _set_and_then(data: JsValue, callback: &Closure<dyn FnMut()>);
        }

        pub fn get_multiple(keys: Vec<String>, callback: &Closure<dyn FnMut(JsValue)>) {
            let keys = map_to_js_value(keys);

            _get_multiple(keys, callback)
        }

        fn _set_optional_callback(data: JsValue, callback: Option<&Closure<dyn FnMut()>>) {
            match callback {
                None => {
                    _set(data);
                }
                Some(c) => {
                    _set_and_then(data, c);
                }
            }
        }

        pub fn set_one<T: Into<JsValue>>(
            key: String,
            value: T,
            callback: Option<&Closure<dyn FnMut()>>
        ) -> Result<(), Error> {
            let data = create_object_with_property(key, value)?;

            _set_optional_callback(data.into(), callback);

            Ok(())
        }

        pub fn set_multiple<T: Serialize>(
            data: T,
            callback: Option<&Closure<dyn FnMut()>>
        ) -> Result<(), Error> {
            _set_optional_callback(serde_wasm_bindgen::to_value(&data)?, callback);

            Ok(())
        }
    }

    pub mod on_changed {
        use wasm_bindgen::prelude::*;
        use std::collections::HashMap;
        use js_sys::Object;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = ["chrome", "storage"])]
            pub type StorageChange;

            #[wasm_bindgen(method, getter, js_name = oldValue)]
            pub fn old_value(this: &StorageChange) -> JsValue;

            #[wasm_bindgen(method, getter, js_name = newValue)]
            pub fn new_value(this: &StorageChange) -> JsValue;

            #[wasm_bindgen(js_namespace = ["chrome", "storage", "onChanged"], js_name = addListener)]
            pub fn add_listener(callback: &Closure<dyn FnMut(JsValue, String)>);
        }

        pub fn create_listener<T>(mut callback: T) -> Closure<dyn FnMut(JsValue, String)>
            where T: FnMut(HashMap<String, StorageChange>, String) + 'static,
        {
            Closure::wrap(Box::new(move |changes, namespace| {
                let changes: Object = changes.into();
                let keys = Object::keys(&changes).to_vec().into_iter().map(|v| v.as_string().unwrap());
                let values = Object::values(&changes).to_vec().into_iter().map(|v| StorageChange::from(v));
                let changes: HashMap<String, StorageChange> = keys.zip(values).collect();

                callback(changes, namespace);
            }))
        }
    }

    pub fn create_get_one_closure<T>(mut callback: T, key: &str) -> Closure<dyn FnMut(JsValue)>
        where T: FnMut(Option<JsValue>) + 'static,
    {
        let key: JsValue = key.into();

        Closure::wrap(Box::new(move | data | {
            let value = Reflect::get(&data, &key);

            let value = match value {
                Ok(v) => {
                    if v.is_undefined() {
                        None
                    } else {
                        Some(v)
                    }
                },
                Err(_) => None,
            };

            callback(value);
        }))
    }
}

pub mod error {
    use std::fmt::{self, Debug};
    use serde_wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[derive(Debug)]
    pub enum Error {
        SerdeWasmBindgen(serde_wasm_bindgen::Error),
        JsValue(JsValue),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::SerdeWasmBindgen(e) => write!(f, "SerdeWasmBindgen error: {}", e),
                Error::JsValue(e) => {
                    write!(f, "JsValue error: ")?;
                    e.fmt(f)
                },
            }
        }
    }

    impl std::error::Error for Error {}

    impl From<serde_wasm_bindgen::Error> for Error {
        fn from(e: serde_wasm_bindgen::Error) -> Self {
            Self::SerdeWasmBindgen(e)
        }
    }

    impl From<JsValue> for Error {
        fn from(e: JsValue) -> Self {
            Self::JsValue(e)
        }
    }
}