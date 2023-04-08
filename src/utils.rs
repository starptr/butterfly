use cfg_if::cfg_if;
use const_str;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::{Request, RouteContext, Response, Secret, wasm_bindgen::JsValue, kv::{KvError, KvStore}, Url};
use url::ParseError;

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
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>>;
}

impl<'a> ToResultResponse<&'a str> for Option<&'a str> {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<&'a str, worker::Result<Response>> {
        self.ok_or(Response::error(err_msg, status))
    }
}

impl ToResultResponse<String> for Option<String> {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<String, worker::Result<Response>> {
        self.ok_or(Response::error(err_msg, status))
    }
}

impl<'a> ToResultResponse<&'a String> for Option<&'a String> {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<&'a String, worker::Result<Response>> {
        self.ok_or(Response::error(err_msg, status))
    }
}

impl<T, E> ToResultResponse<T> for Result<T, DisplayableError<E>> where E: std::fmt::Display {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>> {
        self.map_err(|e| Response::error(format!("{}: {}", err_msg, e), status))
    }
}

impl<T> ToResultResponse<T> for Result<T, KvError> {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>> {
        self.map_err(|e| Response::error(format!("{}: {}", err_msg, e), status))
    }
}

impl<T> ToResultResponse<T> for Result<T, ParseError> {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>> {
        self.map_err(|e| Response::error(format!("{}: {}", err_msg, e), status))
    }
}

impl<T> ToResultResponse<T> for worker::Result<T> {
    fn map_err_to_err_res_with_msg(self, err_msg: &str, status: u16) -> Result<T, worker::Result<Response>> {
        self.map_err(|_| Response::error(err_msg, status))
    }
}

async fn check_is_slug_used(slug: &str, kv: &KvStore) -> Result<bool, worker::Result<Response>> {
    if check_hardcoded_slug(slug).is_some() {
        return Ok(true);
    }

    let get_builder = kv.get(slug);
    let url = get_builder.text().await
        .map_err_to_err_res_with_msg("failed to retrieve KV store for checking unused", 500)?;
    Ok(url.is_some())
}

/*
 * The Ok result should not contain an error response, although it is possible to do so.
 * The Err result should contain an error Response, and no other kind of Response.
 */
pub async fn handle_post_link<D>(mut req: Request, ctx: RouteContext<D>) -> Result<worker::Result<Response>, worker::Result<Response>> {
    let json = req.json::<PostAddRequestBody>().await
        .map_err_to_err_res_with_msg("Json parsing failed", 400)?;
    let token = json.get_token();

    let expected_key = ctx.env.secret("BUTTERFLY_API_TOKEN")
        .map_err_to_err_res_with_msg("correct API key is not defined", 500)?;
    let expected_key: JsValue = expected_key.into();
    let expected_key = expected_key.as_string()
        .map_err_to_err_res_with_msg("API key cannot be parsed into string", 500)?;
    if token != expected_key {
        return Err(Response::error("unauthorized", 401));
    }

    let kv = ctx.kv("KV_FROM_RUST")
        .map_err_to_err_res_with_msg("failed to retrieve KV store", 500)?;
    let mut slug: String;
    while {
        slug = generate_random_slug();
        let is_slug_used = check_is_slug_used(&slug, &kv).await?;
        is_slug_used
    } {}
    let slug = slug;

    let target = json.get_target();
    let put_builder = kv.put(&slug, target)
        .map_err_to_err_res_with_msg("failed to create KV store put builder", 500)?;
    let _put_result = put_builder.execute().await
        .map_err_to_err_res_with_msg("failed to put into KV store", 500)?;
    Ok(Response::from_json(&json!({
        "slug": slug,
        "url": format!("https://yut.to/{}", slug),
    })))
}

fn check_hardcoded_slug(slug: &str) -> Option<String> {
    match slug {
        "tiktok" => Some("https://tiktok.com/@really_yuto".to_owned()),
        "twitter" => Some("https://twitter.com/really_yuto".to_owned()),
        "instagram" => Some("https://instagram.com/really_yuto/".to_owned()),
        "ig" => Some("https://instagram.com/really_yuto/".to_owned()),
        "linkedin" => Some("https://linkedin.com/in/yuton/".to_owned()),
        "facebook" => Some("https://facebook.com/yuto314/".to_owned()),
        "fb" => Some("https://facebook.com/yuto314/".to_owned()),
        _ => None,
    }
}

async fn get_url_from_slug(slug: &str, kv: KvStore) -> Result<String, worker::Result<Response>> {
    if let Some(url) = check_hardcoded_slug(slug) {
        return Ok(url);
    }

    let get_builder = kv.get(slug);
    let url = get_builder.text().await
        .map_err_to_err_res_with_msg("failed to retrieve KV store", 500)?;
    let url = url.map_err_to_err_res_with_msg(&format!("no value for {}", slug), 500)?;
    Ok(url)
}

/*
 * The Ok result should not contain an error Response, although it is possible to do so.
 * The Errr result should contain an error Response, and no other kind of Response.
 */
pub async fn handle_get_link<D>(_req: Request, ctx: RouteContext<D>) -> Result<worker::Result<Response>, worker::Result<Response>> {
    let slug = ctx.param("slug")
        .map_err_to_err_res_with_msg("f", 500)?;
    let kv = ctx.kv("KV_FROM_RUST")
        .map_err_to_err_res_with_msg("failed to retrieve KV store", 500)?;

    let url = get_url_from_slug(slug, kv).await?;
    let parsed_url = Url::parse(&url)
        .or_else(|e| match e {
            // Attempt to fix URL
            ParseError::RelativeUrlWithoutBase => {
                let with_base = "https://".to_owned() + &url;
                Url::parse(&with_base)
            }
            _ => Err(e)
        })
        .map_err_to_err_res_with_msg("failed to parse url", 500)?;
    Ok(Response::redirect_with_status(parsed_url, 301))
}
