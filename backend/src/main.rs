// ============================================
// GCP IAM Search Backend API Server
// ============================================

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, middleware, http::header};
use actix_files as af;
use serde::{Deserialize};
use serde_json::json;
use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;

mod error;
mod models;
mod search;

use search::SearchEngine;
use models::{SearchRequest, SearchMode};

/// JSON data structures for loading from file
#[derive(Debug, Deserialize)]
struct IamDataFile {
    roles: Vec<RoleData>,
    permissions: Vec<PermissionData>,
    metadata: MetadataData,
}

#[derive(Debug, Deserialize)]
struct RoleData {
    name: String,
    title: String,
    description: String,
    stage: String,
    included_permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PermissionData {
    name: String,
    service: String,
}

#[derive(Debug, Deserialize)]
struct MetadataData {
    total_roles: usize,
    total_permissions: usize,
}

/// Application state holding the search engine
pub struct AppState {
    search_engine: Mutex<SearchEngine>,
}

/// Health check endpoint
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "version": "0.1.0"
    }))
}

/// Search endpoint - returns permissions with associated roles, and roles with their permissions
async fn search(
    query: web::Query<SearchRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    // Validate query
    let search_query = query.q.trim();
    if search_query.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Query parameter 'q' is required and cannot be empty"
        }));
    }

    if search_query.len() > 100 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Query too long (max 100 characters)"
        }));
    }

    let engine = data.search_engine.lock().unwrap();
    let mode = query.mode;
    let mode_str = match mode {
        SearchMode::Exact => "exact",
        SearchMode::Prefix => "prefix",
        SearchMode::Fuzzy => "fuzzy",
    };

    // Search both permissions and roles
    let permissions = engine.search_permissions(search_query, mode_str, 0.2);
    let roles = engine.search_roles(search_query, mode_str, 0.2);

    HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "permissions": permissions,
            "roles": roles,
            "query": search_query,
            "mode": mode_str,
        }
    }))
}

/// Get statistics endpoint
async fn stats(data: web::Data<AppState>) -> HttpResponse {
    let engine = data.search_engine.lock().unwrap();
    let (perm_count, role_count) = engine.stats();

    HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "total_permissions": perm_count,
            "total_roles": role_count,
            "indexed": true,
            "version": "0.1.0"
        }
    }))
}

/// Not found handler
async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(json!({
        "success": false,
        "error": "Endpoint not found"
    }))
}

/// Serve permission static page
async fn serve_permission_page(path: web::Path<String>) -> HttpResponse {
    let perm_name = path.into_inner();
    let static_dir = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "../data/static".to_string());

    // Convert permission name to filename (replace / with _)
    let filename = format!("{}.html", perm_name.replace('/', "_"));
    let filepath = PathBuf::from(&static_dir).join("permissions").join(&filename);

    match fs::read_to_string(&filepath) {
        Ok(content) => HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(content),
        Err(_) => HttpResponse::NotFound()
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(format!(r#"<!DOCTYPE html>
<html><head><title>Permission Not Found</title></head>
<body><h1>Permission not found: {}</h1><p><a href="/">Back to search</a></p></body></html>"#, perm_name))
    }
}

/// Serve role static page
async fn serve_role_page(path: web::Path<String>) -> HttpResponse {
    let role_name = path.into_inner();
    let static_dir = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "../data/static".to_string());

    // Convert role name to filename (replace / with _)
    let filename = format!("{}.html", role_name.replace('/', "_"));
    let filepath = PathBuf::from(&static_dir).join("roles").join(&filename);

    match fs::read_to_string(&filepath) {
        Ok(content) => HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(content),
        Err(_) => HttpResponse::NotFound()
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(format!(r#"<!DOCTYPE html>
<html><head><title>Role Not Found</title></head>
<body><h1>Role not found: {}</h1><p><a href="/">Back to search</a></p></body></html>"#, role_name))
    }
}

/// Serve sitemap.xml
async fn serve_sitemap() -> HttpResponse {
    let static_dir = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "../data/static".to_string());
    let filepath = PathBuf::from(&static_dir).join("sitemap.xml");

    match fs::read_to_string(&filepath) {
        Ok(content) => HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "application/xml; charset=utf-8"))
            .body(content),
        Err(_) => HttpResponse::NotFound().body("Sitemap not found")
    }
}

/// Load IAM data from JSON file
fn load_iam_data() -> SearchEngine {
    let mut engine = SearchEngine::new();

    // Try to load from data file
    let data_path = std::env::var("IAM_DATA_PATH")
        .unwrap_or_else(|_| "../data/iam-data.json".to_string());

    println!("   Loading data from: {}", data_path);

    match fs::read_to_string(&data_path) {
        Ok(content) => {
            match serde_json::from_str::<IamDataFile>(&content) {
                Ok(data) => {
                    println!("   Found {} roles and {} permissions in data file",
                        data.metadata.total_roles, data.metadata.total_permissions);

                    // Index all roles with their permissions
                    for role in data.roles {
                        engine.index_role(
                            role.name,
                            role.title,
                            role.description,
                            role.stage,
                            role.included_permissions,
                        );
                    }

                    // Finalize indexes
                    engine.finalize();
                }
                Err(e) => {
                    println!("   Warning: Failed to parse data file: {}", e);
                    println!("   Using empty engine");
                }
            }
        }
        Err(e) => {
            println!("   Warning: Could not load data file: {}", e);
            println!("   Using empty engine. Set IAM_DATA_PATH env var to point to iam-data.json");
        }
    }

    engine
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("\n🚀 Starting GCP IAM Search Backend");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize search engine with real IAM data
    let engine = load_iam_data();
    let (perm_count, role_count) = engine.stats();
    println!("✅ Search engine initialized");
    println!("   📋 {} permissions indexed", perm_count);
    println!("   👤 {} roles indexed", role_count);

    let app_state = web::Data::new(AppState {
        search_engine: Mutex::new(engine),
    });

    println!("\n📡 API Endpoints:");
    println!("   GET  /api/v1/health          - Health check");
    println!("   GET  /api/v1/search          - Search (q=query&mode=prefix)");
    println!("   GET  /api/v1/stats           - Statistics");
    println!("\n🌐 Server running on:");
    println!("   http://127.0.0.1:8000");
    println!("   http://localhost:8000");
    println!("\n⏹️  Press Ctrl+C to stop");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    HttpServer::new(move || {
        // CORS configuration for local development
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(
                actix_web::middleware::DefaultHeaders::new()
                    .add(("X-Version", "0.1.0"))
                    .add(("X-Powered-By", "Rust/Actix")),
            )
            // Health check
            .route("/api/v1/health", web::get().to(health_check))
            // Search endpoint
            .route("/api/v1/search", web::get().to(search))
            // Stats endpoint
            .route("/api/v1/stats", web::get().to(stats))
            // Static pages for SEO
            .route("/permissions/{name:.*}", web::get().to(serve_permission_page))
            .route("/roles/{name:.*}", web::get().to(serve_role_page))
            .route("/sitemap.xml", web::get().to(serve_sitemap))
            // Catch all
            .default_service(web::route().to(not_found))
    })
    .bind("127.0.0.1:8000")?
    .workers(4)
    .run()
    .await
}
