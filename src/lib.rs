use serde_json::json;
use worker::{*, wasm_bindgen::JsValue};

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

macro_rules! attach_get_common_permanent_redirect {
    ($router:expr, $slug:expr, $target:expr) => {
        {
            $router.get($slug, |_, _| {
                Response::redirect_with_status(Url::parse($target).unwrap(), 301)
            })
        }
    };
}
//fn attach_get_common_permanent_redirect<'a, D>(router: Router<'a, D>, slug: &str, target: &str) -> worker::Router<'a, D> {
//    macro_rules! splice_target {
//        () => {
//        };
//    };
//
//    router.get(slug, |_, _| {
//        Response::redirect_with_status(Url::parse(target).unwrap(), 301)
//    })
//}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    //let router = attach_get_common_permanent_redirect!(router, "/tiktok", "https://tiktok.com/@really_yuto");
    //let router = attach_get_common_permanent_redirect!(router, "/twitter", "https://twitter.com/really_yuto");
    //let router = attach_get_common_permanent_redirect!(router, "/instagram", "https://instagram.com/really_yuto/");
    //let router = attach_get_common_permanent_redirect!(router, "/ig", "https://instagram.com/really_yuto/");
    //let router = attach_get_common_permanent_redirect!(router, "/linkedin", "https://linkedin.com/in/yuton/");
    //let router = attach_get_common_permanent_redirect!(router, "/facebook", "https://facebook.com/yuto314/");
    //let router = attach_get_common_permanent_redirect!(router, "/fb", "https://facebook.com/yuto314/");
    router
        //.get("/", |_, _| Response::ok("Hello from Workers!"))
        .post_async("/form/:field", |mut req, ctx| async move {
            if let Some(name) = ctx.param("field") {
                let form = req.form_data().await?;
                match form.get(name) {
                    Some(FormEntry::Field(value)) => {
                        return Response::from_json(&json!({ name: value }))
                    }
                    Some(FormEntry::File(_)) => {
                        return Response::error("`field` param in form shouldn't be a File", 422);
                    }
                    None => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .post_async("/add", |mut req, ctx| async move {
            match req.json::<utils::PostAddRequestBody>().await {
                Ok(json) => {
                    let token = json.get_token();
                    let expected_key = ctx.env.secret("BUTTERFLY_API_TOKEN");
                    if expected_key.is_err() {
                        return Response::error("correct API key is not defined", 500);
                    }
                    let expected_key = expected_key.unwrap();
                    let expected_key: JsValue = expected_key.into();
                    let expected_key: Option<String> = expected_key.as_string();
                    if expected_key.is_none() {
                        return Response::error("API key cannot be parsed into string", 500);
                    }
                    let expected_key = expected_key.unwrap();
                    if token != expected_key {
                        return Response::error("unauthorized", 401);
                    }

                    let target = json.get_target();
                    let slug = utils::generate_random_slug();
                    let kv = ctx.kv("KV_FROM_RUST");
                    if kv.is_err() {
                        return Response::error("failed to retrieve KV store", 500);
                    }
                    let kv = kv.unwrap();
                    let put_builder = kv.put(&slug, target);
                    if put_builder.is_err() {
                        return Response::error(format!("failed to create KV store put builder: {}", put_builder.unwrap_err()), 500);
                    }
                    let put_builder = put_builder.unwrap();
                    let put_result = put_builder.execute().await;
                    if put_result.is_err() {
                        return Response::error(format!("failed to put into KV store: {}", put_result.unwrap_err()), 500);
                    }
                    Response::ok(format!("your new url: https://yut.to/{}", slug))
                },
                Err(e) => Response::error(format!("Json parsing failed: {}", e), 400),
            }
        })
        .get_async("/:slug", |mut req, ctx| async move {
            if let Some(slug) = ctx.param("slug") {
                let kv = ctx.kv("KV_FROM_RUST");
                if kv.is_err() {
                    return Response::error("failed to retrieve KV store", 500);
                }
                let kv = kv.unwrap();
                let get_builder = kv.get(slug);
                let url = get_builder.text().await;
                if url.is_err() {
                    return Response::error(format!("failed to retrieve KV store: {}", url.unwrap_err()), 500);
                }
                let url: Option<String> = url.unwrap();
                if url.is_none() {
                    return Response::error(format!("no value for {}", slug), 500);
                }
                let url: String = url.unwrap();
                let mut parsed_url = Url::parse(&url);
                if parsed_url.is_err() {
                    let with_protocol = "https://".to_owned() + &url;
                    let parsed_url_with_protocol = Url::parse(&with_protocol);
                    if parsed_url_with_protocol.is_err() {
                        return Response::error(format!("failed to parse url: {}", with_protocol), 500);
                    }
                    parsed_url = parsed_url_with_protocol;
                }
                let url = parsed_url.unwrap();
                return Response::redirect_with_status(url, 301);
            }
            Response::error("Bad Request", 400)
        })
        //.get("/worker-version", |_, ctx| {
        //    let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
        //    Response::ok(version)
        //})
        .run(req, env)
        .await
}
