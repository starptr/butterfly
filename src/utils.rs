use cfg_if::cfg_if;
use const_str;
use rand::Rng;
use serde::{Deserialize, Serialize};
use worker::{Request, RouteContext, Response, Secret, wasm_bindgen::JsValue};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

const ALPHABET: [char; 33] = const_str::to_char_array!("0123456789abcdefghjkmnpqrstuvwxyz");

pub fn generate_random_slug() -> String {
    let mut rng = rand::thread_rng();
    (0..6).map(|_| {
        ALPHABET[rng.gen_range(0..ALPHABET.len())]
    }).collect()
}

#[derive(Serialize, Deserialize)]
pub struct PostAddRequestBody {
    target: String,
    token: String,
}

impl PostAddRequestBody {
    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn get_target(&self) -> &str {
        &self.target
    }
}

struct DisplayableError<T>(T) where T: std::fmt::Display;
impl<E> From<E> for DisplayableError<E> where E: std::fmt::Display {
    fn from(err: E) -> Self {
        DisplayableError(err)
    }
}
impl<T> std::fmt::Display for DisplayableError<T> where T: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/*
 * If the trait implements a wrapper type, T is intended to be the inner type.
 * Otherwise, T is intended to be Self.
 */
trait ToResultResponse<T> {
    /*
     * The Ok result should not contain an error response, although it is possible to do so.
     * The Err result should contain an error Response, and no other kind of Response.
     */
    fn from_with(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>>;
}

impl<'a> ToResultResponse<&'a str> for Option<&'a str> {
    fn from_with(self, err_msg: &str, status: u16) -> Result<&'a str, worker::Result<Response>> {
        self.ok_or(Response::error(err_msg, status))
    }
}

impl ToResultResponse<String> for Option<String> {
    fn from_with(self, err_msg: &str, status: u16) -> Result<String, worker::Result<Response>> {
        self.ok_or(Response::error(err_msg, status))
    }
}

impl<T, E> ToResultResponse<T> for Result<T, DisplayableError<E>> where E: std::fmt::Display {
    fn from_with(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>> {
        self.map_err(|e| Response::error(format!("{}: {}", err_msg, e), status))
    }
}

impl<T> ToResultResponse<T> for worker::Result<T> {
    fn from_with(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>> {
        self.map_err(|_| Response::error(err_msg, status))
    }
}

/*
 * The Ok result should not contain an error response, although it is possible to do so.
 * The Err result should contain an error Response, and no other kind of Response.
 */
pub async fn handle_get_link<D>(mut req: Request, ctx: RouteContext<D>) -> Result<worker::Result<Response>, worker::Result<Response>> {
    //let slug = ctx.param("slug").ok_or(Response::error("bad request", 400))?;
    let json = req.json::<PostAddRequestBody>().await.from_with("Json parsing failed", 400)?;
    let token = json.get_token();

    let expected_key = ctx.env.secret("BUTTERFLY_API_TOKEN").from_with("correct API key is not defined", 500)?;
    let expected_key: JsValue = expected_key.into();
    let expected_key = expected_key.as_string().from_with("API key cannot be parsed into string", 500)?;
    if token != expected_key {
        return Err(Response::error("unauthorized", 401));
    }

    let target = json.get_target();
    let slug = generate_random_slug();
    let kv = ctx.kv("KV_FROM_RUST").from_with("failed to retrieve KV store", 500)?;
    let put_builder = kv.put(&slug, target)
        .map_err(|e| DisplayableError::from(e))
        .from_with("failed to create KV store put builder", 500)?;
    let _put_result = put_builder.execute().await
        .map_err(|e| DisplayableError::from(e))
        .from_with("failed to put into KV store", 500)?;
    Ok(Response::ok(format!("your new url: https://yut.to/{}", slug)))
}
