use worker::*;

pub async fn get_handler(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>isqr - QR Code URL Shortener</title>
    <style>
        body { font-family: system-ui; max-width: 600px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .info { background: #f0f0f0; padding: 15px; border-radius: 5px; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>isqr.me</h1>
    <div class="info">
        <p>Fast QR code generation and URL shortening service.</p>
        <p><a href="/create" style="color: #0066cc; text-decoration: none; font-weight: bold;">Create a QR Code →</a></p>
    </div>
</body>
</html>"#;
    
    Response::from_html(html)
} 