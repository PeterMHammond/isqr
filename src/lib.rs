use worker::*;

mod routes {
    pub mod home;
    pub mod create;
    pub mod redirect;
    pub mod not_found;
}



#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/", routes::home::get_handler)
        .get_async("/create", routes::create::get_handler)
        .post_async("/create", routes::create::post_handler)
        .get_async("/:id", routes::redirect::get_handler)
        .or_else_any_method_async("/*path", routes::not_found::handler)
        .run(req, env)
        .await
}