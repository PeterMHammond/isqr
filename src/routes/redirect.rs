use worker::*;

pub async fn get_handler(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").map_or("", |v| v);
    
    if id.is_empty() {
        return Response::error("Not found", 404);
    }
    
    let kv = ctx.kv("KV")?;
    
    match kv.get(id).text().await? {
        Some(url) => {
            Response::redirect(Url::parse(&url)?)
        },
        None => Response::error("Not found", 404),
    }
} 