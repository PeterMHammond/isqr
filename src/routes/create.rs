use worker::*;
use qrcode::{QrCode, render::svg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CreateQrRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct CreateQrResponse {
    short_url: String,
    qr_code: String,
}

fn generate_short_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let timestamp = Date::now().as_millis() as u64;
    let mut id = String::new();
    let mut num = timestamp;
    for _ in 0..6 {
        id.push(CHARSET[(num % CHARSET.len() as u64) as usize] as char);
        num /= CHARSET.len() as u64;
    }
    id
}

pub async fn get_handler(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Create QR Code - isqr</title>
    <style>
        body { font-family: system-ui; max-width: 600px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        input, button { padding: 10px; margin: 10px 0; width: 100%; box-sizing: border-box; }
        input[type="url"] { font-size: 16px; }
        button { background: #0066cc; color: white; border: none; cursor: pointer; font-size: 16px; }
        button:hover { background: #0052cc; }
        #result { margin-top: 20px; }
        .error { color: red; }
        .success { color: green; }
        #qrcode { margin: 20px 0; text-align: center; cursor: pointer; }
        #qrcode svg { border: 2px solid transparent; transition: border-color 0.2s; }
        #qrcode:hover svg { border-color: #0066cc; }
        .save-hint { font-size: 12px; color: #666; text-align: center; margin-top: 10px; }
    </style>
</head>
<body>
    <h1>Create QR Code</h1>
    <form id="createForm">
        <input type="url" id="url" placeholder="Enter URL (e.g., https://example.com)" required>
        <button type="submit">Generate QR Code</button>
    </form>
    <div id="result"></div>
    
    <script>
        document.getElementById('createForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const url = document.getElementById('url').value;
            const result = document.getElementById('result');
            
            result.innerHTML = 'Creating...';
            
            try {
                const response = await fetch('/create', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ url })
                });
                
                if (response.ok) {
                    const data = await response.json();
                    result.innerHTML = `
                        <div class="success">
                            <p>Short URL: <a href="${data.short_url}" target="_blank">${data.short_url}</a></p>
                            <div id="qrcode" title="Click to save QR code">${data.qr_code}</div>
                            <p class="save-hint">Click the QR code to save as image</p>
                        </div>
                    `;
                    
                    document.getElementById('qrcode').addEventListener('click', function() {
                        const svg = this.querySelector('svg');
                        const svgData = new XMLSerializer().serializeToString(svg);
                        const canvas = document.createElement('canvas');
                        const ctx = canvas.getContext('2d');
                        const img = new Image();
                        
                        img.onload = function() {
                            canvas.width = img.width;
                            canvas.height = img.height;
                            ctx.fillStyle = 'white';
                            ctx.fillRect(0, 0, canvas.width, canvas.height);
                            ctx.drawImage(img, 0, 0);
                            
                            canvas.toBlob(function(blob) {
                                const url = URL.createObjectURL(blob);
                                const a = document.createElement('a');
                                a.href = url;
                                a.download = 'qrcode-' + data.short_url.split('/').pop() + '.png';
                                document.body.appendChild(a);
                                a.click();
                                document.body.removeChild(a);
                                URL.revokeObjectURL(url);
                            });
                        };
                        
                        img.src = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(svgData)));
                    });
                } else {
                    const error = await response.text();
                    result.innerHTML = `<div class="error">Error: ${error}</div>`;
                }
            } catch (error) {
                result.innerHTML = `<div class="error">Error: ${error.message}</div>`;
            }
        });
    </script>
</body>
</html>"#;
    
    Response::from_html(html)
}

pub async fn post_handler(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let email = req.headers().get("cf-access-authenticated-user-email")?
        .unwrap_or_else(|| "local@example.com".to_string());
    
    let body: CreateQrRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => return Response::error("Invalid request body", 400),
    };
    
    if body.url.is_empty() {
        return Response::error("URL is required", 400);
    }
    
    let short_id = generate_short_id();
    let kv = ctx.kv("KV")?;
    
    kv.put(&short_id, &body.url)?
        .metadata(serde_json::json!({
            "created_by": email,
            "created_at": Date::now().to_string(),
        }))?
        .execute()
        .await?;
    
    // Get the host from the request headers to make URLs work both locally and deployed
    let host = req.headers().get("host")?
        .unwrap_or_else(|| "localhost:8787".to_string());
    let protocol = if host.contains("localhost") { "http" } else { "https" };
    
    let short_url = format!("{}://{}/{}", protocol, host, short_id);
    let code = QrCode::new(short_url.as_bytes())
        .map_err(|_| Error::from("Failed to generate QR code"))?;
    
    let qr_svg = code.render::<svg::Color>()
        .min_dimensions(300, 300)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    
    let response = CreateQrResponse {
        short_url,
        qr_code: qr_svg,
    };
    
    Response::from_json(&response)
} 