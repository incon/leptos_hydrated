use server_fn::codec::{Encoding, Post};
use server_fn::{ContentType, Format, FormatType, Encodes, Decodes};
use server_fn::error::ServerFnError;
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use http::Method;

/// A codec that uses the browser's native `JSON.parse` and `JSON.stringify` on the client,
/// and `serde_json` on the server.
pub struct BrowserJson;

impl ContentType for BrowserJson {
    const CONTENT_TYPE: &'static str = "application/json";
}

impl FormatType for BrowserJson {
    const FORMAT_TYPE: Format = Format::Text;
}

#[allow(dead_code)]
fn ser_err(e: impl ToString) -> ServerFnError {
    ServerFnError::Serialization(e.to_string())
}

#[allow(dead_code)]
fn de_err(e: impl ToString) -> ServerFnError {
    ServerFnError::Deserialization(e.to_string())
}

impl<T> Encodes<T> for BrowserJson
where
    T: Serialize + Send,
{
    type Error = ServerFnError;

    fn encode(output: &T) -> Result<Bytes, ServerFnError> {
        #[cfg(feature = "ssr")]
        {
            serde_json::to_vec(output)
                .map(Bytes::from)
                .map_err(ser_err)
        }
        #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
        {
            let val = serde_wasm_bindgen::to_value(output).map_err(ser_err)?;
            let json = js_sys::JSON::stringify(&val)
                .map_err(|e| ser_err(format!("{:?}", e)))?;
            let string: String = json.into();
            Ok(Bytes::from(string))
        }
        #[cfg(not(any(feature = "ssr", all(target_arch = "wasm32", not(feature = "ssr")))))]
        {
            let _ = output;
            panic!("BrowserJson::encode is only supported on SSR or WASM client")
        }
    }
}

impl<T> Decodes<T> for BrowserJson
where
    T: DeserializeOwned + Send,
{
    type Error = ServerFnError;

    fn decode(bytes: Bytes) -> Result<T, ServerFnError> {
        #[cfg(feature = "ssr")]
        {
            serde_json::from_slice(&bytes)
                .map_err(de_err)
        }
        #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
        {
            let string = String::from_utf8(bytes.to_vec()).map_err(de_err)?;
            let parsed = js_sys::JSON::parse(&string).map_err(|e| de_err(format!("{:?}", e)))?;
            serde_wasm_bindgen::from_value(parsed).map_err(de_err)
        }
        #[cfg(not(any(feature = "ssr", all(target_arch = "wasm32", not(feature = "ssr")))))]
        {
             let _ = bytes;
             panic!("BrowserJson::decode is only supported on SSR or WASM client")
        }
    }
}

impl Encoding for BrowserJson {
    const METHOD: Method = Method::POST;
}

/// A type alias for a `POST` request using the `BrowserJson` codec.
pub type BrowserJsonPost = Post<BrowserJson>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        count: i32,
    }

    #[test]
    fn test_browser_json_ssr_roundtrip() {
        let data = TestData {
            name: "Leptos".to_string(),
            count: 42,
        };

        // On SSR, BrowserJson uses serde_json
        let encoded = BrowserJson::encode(&data).expect("Failed to encode");
        let decoded: TestData = BrowserJson::decode(encoded).expect("Failed to decode");

        assert_eq!(data, decoded);
    }
}
