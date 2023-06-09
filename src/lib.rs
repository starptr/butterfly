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
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .post_async("/add", |req, ctx| async move {
            let res = utils::handle_post_link(req, ctx);
            match res.await {
                Ok(res) => res,
                Err(res) => res,
            }
        })
        .get_async("/:slug", |req, ctx| async move {
            let res = utils::handle_get_link(req, ctx);
            match res.await {
                Ok(res) => res,
                Err(res) => res,
            }
        })
        .run(req, env)
        .await
}
