use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Include pre-built index at compile time
static INDEX_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/prebuilt_index.bin"));

// Include frontend files at compile time
static INDEX_HTML: &str = include_str!("../../frontend/public/index.html");
static STYLES_CSS: &str = include_str!("../../frontend/public/styles.css");
static APP_JS: &str = include_str!("../../frontend/public/app.js");

// Deserializable search index structures (must match build.rs)
#[derive(Debug, Clone, Deserialize)]
struct Role {
    name: String,
    title: String,
    description: String,
    stage: String,
    included_permissions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Permission {
    name: String,
    service: String,
    resource: String,
    action: String,
    granted_by_roles: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RoleSummary {
    name: String,
    title: String,
    stage: String,
}

#[derive(Debug, Deserialize)]
struct PrebuiltIndex {
    permissions: Vec<Permission>,
    permission_names: Vec<String>,
    roles: Vec<Role>,
    role_names: Vec<String>,
    role_summaries: Vec<RoleSummary>,
    #[allow(dead_code)]
    service_to_permissions: HashMap<String, Vec<u32>>,
    permission_names_lower: Vec<String>,
    role_names_lower: Vec<String>,
    role_titles_lower: Vec<String>,
}

// API response types
#[derive(Serialize)]
struct PermissionSearchResult {
    name: String,
    service: String,
    resource: String,
    action: String,
    score: f64,
    granted_by_roles: Vec<RoleSummary>,
}

#[derive(Serialize)]
struct RoleSearchResult {
    name: String,
    title: String,
    description: String,
    stage: String,
    score: f64,
    permission_count: usize,
    sample_permissions: Vec<String>,
}

#[derive(Serialize)]
struct SearchResponse {
    success: bool,
    data: SearchData,
}

#[derive(Serialize)]
struct SearchData {
    permissions: Vec<PermissionSearchResult>,
    roles: Vec<RoleSearchResult>,
    query: String,
    mode: String,
}

#[derive(Serialize)]
struct StatsResponse {
    success: bool,
    data: StatsData,
}

#[derive(Serialize)]
struct StatsData {
    total_permissions: usize,
    total_roles: usize,
    indexed: bool,
    version: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn main() -> Result<(), Error> {
    let req = Request::from_client();
    let resp = handle_request(req)?;
    resp.send_to_client();
    Ok(())
}

// Allowed domains for access control
const ALLOWED_HOSTS: &[&str] = &["gcpiam.com", "www.gcpiam.com", "localhost", "127.0.0.1"];

fn is_allowed_host(req: &Request) -> bool {
    // Check the Host header
    if let Some(host) = req.get_header_str("host") {
        let host_without_port = host.split(':').next().unwrap_or(host);
        if ALLOWED_HOSTS.iter().any(|&h| h == host_without_port) {
            return true;
        }
    }
    false
}

fn handle_request(req: Request) -> Result<Response, Error> {
    let path = req.get_path();
    let method = req.get_method();

    // Handle OPTIONS preflight
    if method == Method::OPTIONS {
        let mut resp = Response::from_status(StatusCode::NO_CONTENT);
        resp.set_header("Access-Control-Allow-Origin", "https://gcpiam.com");
        resp.set_header("Access-Control-Allow-Methods", "GET, OPTIONS");
        resp.set_header("Access-Control-Allow-Headers", "Content-Type");
        return Ok(resp);
    }

    // Block requests not coming through allowed domains
    if !is_allowed_host(&req) {
        let mut resp = Response::from_status(StatusCode::FORBIDDEN);
        resp.set_header("Content-Type", "application/json");
        resp.set_body(r#"{"error":"Access denied. Please use gcpiam.com"}"#);
        return Ok(resp);
    }

    // Route requests
    match path {
        "/" | "/index.html" => serve_html(INDEX_HTML),
        "/styles.css" => serve_css(STYLES_CSS),
        "/app.js" => serve_js(APP_JS),
        "/api/v1/health" => serve_json(handle_health()),
        "/api/v1/stats" => serve_json(handle_stats()),
        p if p.starts_with("/api/v1/search") => serve_json(handle_search(&req)),
        p if p.starts_with("/permissions/") => serve_permission_page(p),
        p if p.starts_with("/roles/") => serve_role_page(p),
        _ => serve_not_found(),
    }
}

fn serve_html(content: &str) -> Result<Response, Error> {
    let mut resp = Response::from_status(StatusCode::OK);
    resp.set_header("Content-Type", "text/html; charset=utf-8");
    resp.set_header("Cache-Control", "public, max-age=3600");
    resp.set_body(content);
    Ok(resp)
}

fn serve_css(content: &str) -> Result<Response, Error> {
    let mut resp = Response::from_status(StatusCode::OK);
    resp.set_header("Content-Type", "text/css; charset=utf-8");
    resp.set_header("Cache-Control", "public, max-age=86400");
    resp.set_body(content);
    Ok(resp)
}

fn serve_js(content: &str) -> Result<Response, Error> {
    let mut resp = Response::from_status(StatusCode::OK);
    resp.set_header("Content-Type", "application/javascript; charset=utf-8");
    resp.set_header("Cache-Control", "public, max-age=86400");
    resp.set_body(content);
    Ok(resp)
}

fn serve_json(result: Result<String, String>) -> Result<Response, Error> {
    let mut resp = match result {
        Ok(body) => {
            let mut r = Response::from_status(StatusCode::OK);
            r.set_body(body);
            r
        }
        Err(e) => {
            let mut r = Response::from_status(StatusCode::BAD_REQUEST);
            r.set_body(serde_json::to_string(&ErrorResponse { error: e }).unwrap());
            r
        }
    };
    resp.set_header("Content-Type", "application/json");
    resp.set_header("Access-Control-Allow-Origin", "https://gcpiam.com");
    resp.set_header("Cache-Control", "public, max-age=60");
    Ok(resp)
}

fn serve_not_found() -> Result<Response, Error> {
    let mut resp = Response::from_status(StatusCode::NOT_FOUND);
    resp.set_header("Content-Type", "text/html; charset=utf-8");
    resp.set_body(r#"<!DOCTYPE html>
<html><head><title>Not Found</title></head>
<body style="font-family: system-ui; max-width: 600px; margin: 50px auto; padding: 20px;">
<h1>Page Not Found</h1>
<p><a href="/">Back to Search</a></p>
</body></html>"#);
    Ok(resp)
}

fn serve_permission_page(path: &str) -> Result<Response, Error> {
    let perm_name = path.strip_prefix("/permissions/").unwrap_or("");
    if perm_name.is_empty() {
        return serve_not_found();
    }

    let index: PrebuiltIndex = match bincode::deserialize(INDEX_DATA) {
        Ok(idx) => idx,
        Err(_) => return serve_not_found(),
    };

    // Find the permission
    let perm_idx = index.permission_names.iter().position(|n| n == perm_name);
    let perm = match perm_idx {
        Some(idx) => &index.permissions[idx],
        None => return serve_not_found(),
    };

    // Get roles that grant this permission
    let roles_html: String = perm.granted_by_roles
        .iter()
        .filter_map(|&idx| index.roles.get(idx as usize))
        .map(|role| {
            let stage_color = match role.stage.as_str() {
                "GA" => "#4CAF50",
                "BETA" => "#FF9800",
                "ALPHA" => "#2196F3",
                _ => "#9E9E9E",
            };
            format!(
                r#"<div class="role-card">
                    <a href="/roles/{}" class="role-name">{}</a>
                    <div class="role-title">{}</div>
                    <span class="stage-badge" style="background:{};">{}</span>
                </div>"#,
                html_escape(&role.name),
                html_escape(&role.name),
                html_escape(&role.title),
                stage_color,
                html_escape(&role.stage)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - GCP IAM Permission</title>
    <meta name="description" content="GCP IAM permission {} - granted by {} roles">
    <style>
        :root {{ --accent: #1f73e7; }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: system-ui, sans-serif; background: #f5f5f5; color: #333; line-height: 1.6; }}
        .container {{ max-width: 900px; margin: 0 auto; padding: 20px; }}
        .header {{ background: linear-gradient(135deg, var(--accent), #1557b0); color: white; padding: 30px 20px; margin: -20px -20px 20px; }}
        .breadcrumb {{ margin-bottom: 10px; opacity: 0.9; }}
        .breadcrumb a {{ color: white; text-decoration: none; }}
        .breadcrumb a:hover {{ text-decoration: underline; }}
        h1 {{ font-size: 1.5rem; word-break: break-all; }}
        .meta {{ display: flex; gap: 10px; margin-top: 15px; flex-wrap: wrap; }}
        .badge {{ padding: 4px 12px; border-radius: 4px; font-size: 0.85rem; background: rgba(255,255,255,0.2); }}
        .section {{ background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .section-title {{ font-size: 1.1rem; margin-bottom: 15px; color: #555; }}
        .role-card {{ padding: 12px; border: 1px solid #e0e0e0; border-radius: 6px; margin-bottom: 10px; }}
        .role-card:hover {{ border-color: var(--accent); }}
        .role-name {{ color: var(--accent); text-decoration: none; font-weight: 600; }}
        .role-name:hover {{ text-decoration: underline; }}
        .role-title {{ color: #666; font-size: 0.9rem; margin-top: 4px; }}
        .stage-badge {{ display: inline-block; padding: 2px 8px; border-radius: 4px; color: white; font-size: 0.75rem; margin-top: 8px; }}
        .empty {{ color: #999; font-style: italic; }}
        @media (prefers-color-scheme: dark) {{
            body {{ background: #1a1a1a; color: #e0e0e0; }}
            .section {{ background: #2d2d2d; }}
            .role-card {{ border-color: #444; }}
            .role-title {{ color: #aaa; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="breadcrumb"><a href="/">Search</a> / Permission</div>
            <h1>{}</h1>
            <div class="meta">
                <span class="badge">Service: {}</span>
                <span class="badge">Resource: {}</span>
                <span class="badge">Action: {}</span>
            </div>
        </div>
        <div class="section">
            <div class="section-title">Granted by {} role(s)</div>
            {}
        </div>
    </div>
</body>
</html>"#,
        html_escape(perm_name),
        html_escape(perm_name),
        perm.granted_by_roles.len(),
        html_escape(perm_name),
        html_escape(&perm.service),
        html_escape(&perm.resource),
        html_escape(&perm.action),
        perm.granted_by_roles.len(),
        if roles_html.is_empty() { "<p class=\"empty\">No roles grant this permission directly.</p>".to_string() } else { roles_html }
    );

    let mut resp = Response::from_status(StatusCode::OK);
    resp.set_header("Content-Type", "text/html; charset=utf-8");
    resp.set_header("Cache-Control", "public, max-age=3600");
    resp.set_body(html);
    Ok(resp)
}

fn serve_role_page(path: &str) -> Result<Response, Error> {
    let role_name = path.strip_prefix("/roles/").unwrap_or("");
    if role_name.is_empty() {
        return serve_not_found();
    }

    let index: PrebuiltIndex = match bincode::deserialize(INDEX_DATA) {
        Ok(idx) => idx,
        Err(_) => return serve_not_found(),
    };

    // Find the role
    let role_idx = index.role_names.iter().position(|n| n == role_name);
    let role = match role_idx {
        Some(idx) => &index.roles[idx],
        None => return serve_not_found(),
    };

    let stage_color = match role.stage.as_str() {
        "GA" => "#4CAF50",
        "BETA" => "#FF9800",
        "ALPHA" => "#2196F3",
        _ => "#9E9E9E",
    };

    // Generate permissions list
    let perms_html: String = role.included_permissions
        .iter()
        .map(|perm| {
            format!(
                r#"<div class="perm-item"><a href="/permissions/{}" class="perm-name">{}</a></div>"#,
                html_escape(perm),
                html_escape(perm)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - GCP IAM Role</title>
    <meta name="description" content="{} - {}">
    <style>
        :root {{ --accent: #1f73e7; }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: system-ui, sans-serif; background: #f5f5f5; color: #333; line-height: 1.6; }}
        .container {{ max-width: 900px; margin: 0 auto; padding: 20px; }}
        .header {{ background: linear-gradient(135deg, var(--accent), #1557b0); color: white; padding: 30px 20px; margin: -20px -20px 20px; }}
        .breadcrumb {{ margin-bottom: 10px; opacity: 0.9; }}
        .breadcrumb a {{ color: white; text-decoration: none; }}
        .breadcrumb a:hover {{ text-decoration: underline; }}
        h1 {{ font-size: 1.5rem; word-break: break-all; }}
        .role-title {{ font-size: 1.1rem; opacity: 0.95; margin-top: 8px; }}
        .role-desc {{ margin-top: 10px; opacity: 0.9; font-size: 0.95rem; }}
        .meta {{ display: flex; gap: 10px; margin-top: 15px; flex-wrap: wrap; }}
        .badge {{ padding: 4px 12px; border-radius: 4px; font-size: 0.85rem; }}
        .section {{ background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .section-title {{ font-size: 1.1rem; margin-bottom: 15px; color: #555; }}
        .perm-item {{ padding: 8px 12px; border-bottom: 1px solid #eee; }}
        .perm-item:last-child {{ border-bottom: none; }}
        .perm-name {{ color: var(--accent); text-decoration: none; font-family: monospace; font-size: 0.9rem; }}
        .perm-name:hover {{ text-decoration: underline; }}
        @media (prefers-color-scheme: dark) {{
            body {{ background: #1a1a1a; color: #e0e0e0; }}
            .section {{ background: #2d2d2d; }}
            .perm-item {{ border-color: #444; }}
            .section-title {{ color: #aaa; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="breadcrumb"><a href="/">Search</a> / Role</div>
            <h1>{}</h1>
            <div class="role-title">{}</div>
            <div class="role-desc">{}</div>
            <div class="meta">
                <span class="badge" style="background:{}; color:white;">{}</span>
                <span class="badge" style="background:rgba(255,255,255,0.2);">{} permissions</span>
            </div>
        </div>
        <div class="section">
            <div class="section-title">Included Permissions</div>
            {}
        </div>
    </div>
</body>
</html>"#,
        html_escape(&role.name),
        html_escape(&role.title),
        html_escape(&role.description),
        html_escape(&role.name),
        html_escape(&role.title),
        html_escape(&role.description),
        stage_color,
        html_escape(&role.stage),
        role.included_permissions.len(),
        perms_html
    );

    let mut resp = Response::from_status(StatusCode::OK);
    resp.set_header("Content-Type", "text/html; charset=utf-8");
    resp.set_header("Cache-Control", "public, max-age=3600");
    resp.set_body(html);
    Ok(resp)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn handle_health() -> Result<String, String> {
    serde_json::to_string(&HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0-edge".to_string(),
    })
    .map_err(|e| e.to_string())
}

fn handle_stats() -> Result<String, String> {
    let index: PrebuiltIndex = bincode::deserialize(INDEX_DATA).map_err(|e| e.to_string())?;

    serde_json::to_string(&StatsResponse {
        success: true,
        data: StatsData {
            total_permissions: index.permissions.len(),
            total_roles: index.roles.len(),
            indexed: true,
            version: "0.1.0-edge".to_string(),
        },
    })
    .map_err(|e| e.to_string())
}

fn handle_search(req: &Request) -> Result<String, String> {
    let query_string = req.get_query_str().unwrap_or("");
    let params: HashMap<String, String> = url::form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect();

    let query = params.get("q").map(|s: &String| s.as_str()).unwrap_or("").trim();
    if query.is_empty() {
        return Err("Query parameter 'q' is required".to_string());
    }
    if query.len() > 100 {
        return Err("Query too long (max 100 characters)".to_string());
    }

    let mode = params.get("mode").map(|s: &String| s.as_str()).unwrap_or("prefix");

    let index: PrebuiltIndex = bincode::deserialize(INDEX_DATA).map_err(|e| e.to_string())?;

    let permissions = search_permissions(&index, query, mode);
    let roles = search_roles(&index, query, mode);

    serde_json::to_string(&SearchResponse {
        success: true,
        data: SearchData {
            permissions,
            roles,
            query: query.to_string(),
            mode: mode.to_string(),
        },
    })
    .map_err(|e| e.to_string())
}

fn search_permissions(index: &PrebuiltIndex, query: &str, mode: &str) -> Vec<PermissionSearchResult> {
    let query_lower = query.to_lowercase();
    let mut results: Vec<(usize, f64)> = Vec::new();

    match mode {
        "exact" => {
            if let Ok(idx) = index.permission_names.binary_search(&query.to_string()) {
                results.push((idx, 1.0));
            }
        }
        "prefix" => {
            for (idx, name_lower) in index.permission_names_lower.iter().enumerate() {
                if name_lower.starts_with(&query_lower) {
                    results.push((idx, 0.9));
                }
            }
        }
        _ => {
            for (idx, name_lower) in index.permission_names_lower.iter().enumerate() {
                if name_lower.contains(&query_lower) {
                    results.push((idx, 0.85));
                }
            }
        }
    }

    results
        .into_iter()
        .take(20)
        .map(|(idx, score)| {
            let perm = &index.permissions[idx];
            let granted_by_roles: Vec<RoleSummary> = perm
                .granted_by_roles
                .iter()
                .take(5)
                .filter_map(|&role_idx| index.role_summaries.get(role_idx as usize).cloned())
                .collect();

            PermissionSearchResult {
                name: perm.name.clone(),
                service: perm.service.clone(),
                resource: perm.resource.clone(),
                action: perm.action.clone(),
                score,
                granted_by_roles,
            }
        })
        .collect()
}

fn search_roles(index: &PrebuiltIndex, query: &str, mode: &str) -> Vec<RoleSearchResult> {
    let query_lower = query.to_lowercase();
    let mut results: Vec<(usize, f64)> = Vec::new();

    match mode {
        "exact" => {
            if let Ok(idx) = index.role_names.binary_search(&query.to_string()) {
                results.push((idx, 1.0));
            }
        }
        "prefix" => {
            for (idx, name_lower) in index.role_names_lower.iter().enumerate() {
                if name_lower.starts_with(&query_lower)
                    || index.role_titles_lower[idx].starts_with(&query_lower)
                {
                    results.push((idx, 0.9));
                }
            }
        }
        _ => {
            for (idx, name_lower) in index.role_names_lower.iter().enumerate() {
                if name_lower.contains(&query_lower)
                    || index.role_titles_lower[idx].contains(&query_lower)
                {
                    results.push((idx, 0.85));
                }
            }
        }
    }

    results
        .into_iter()
        .take(20)
        .map(|(idx, score)| {
            let role = &index.roles[idx];
            RoleSearchResult {
                name: role.name.clone(),
                title: role.title.clone(),
                description: role.description.clone(),
                stage: role.stage.clone(),
                score,
                permission_count: role.included_permissions.len(),
                sample_permissions: role.included_permissions.iter().take(5).cloned().collect(),
            }
        })
        .collect()
}
