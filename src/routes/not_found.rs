use worker::*;

pub async fn handler(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::error("Not found", 404)
} 