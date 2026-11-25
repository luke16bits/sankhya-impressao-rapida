mod handlers;
mod printer;
mod service;

use axum::{
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

// Embutir o certificado e chave privada direto dentro do binário (Single Executable)
const CERT_PEM: &[u8] = include_bytes!("../certs/cert.pem");
const KEY_PEM: &[u8] = include_bytes!("../certs/key.pem");

async fn cors_and_pna_middleware(mut req: axum::extract::Request, next: Next) -> Response {
    // 1. Limpar matrix parameters da URL (ex: /.info;sessionId=123 -> /.info)
    let uri = req.uri().clone();
    let path = uri.path();
    if let Some(idx) = path.find(';') {
        let clean_path = &path[..idx];
        let new_uri_str = if let Some(query) = uri.query() {
            format!("{}?{}", clean_path, query)
        } else {
            clean_path.to_string()
        };
        if let Ok(parsed) = new_uri_str.parse::<axum::http::Uri>() {
            *req.uri_mut() = parsed;
        }
    }

    let origin = req.headers().get("origin").cloned();
    let is_options = req.method() == axum::http::Method::OPTIONS;

    let mut resp = if is_options {
        axum::response::IntoResponse::into_response(axum::http::StatusCode::OK)
    } else {
        next.run(req).await
    };

    let headers = resp.headers_mut();
    headers.insert(
        "access-control-allow-private-network",
        axum::http::HeaderValue::from_static("true"),
    );
    headers.insert(
        "access-control-allow-credentials",
        axum::http::HeaderValue::from_static("true"),
    );
    headers.insert(
        "access-control-allow-methods",
        axum::http::HeaderValue::from_static("GET, POST, OPTIONS, HEAD"),
    );
    headers.insert(
        "access-control-allow-headers",
        axum::http::HeaderValue::from_static("Content-Type, Authorization, X-Requested-With, Origin, Accept, Cookie"),
    );
    if let Some(orig) = origin {
        headers.insert("access-control-allow-origin", orig);
    } else {
        headers.insert("access-control-allow-origin", axum::http::HeaderValue::from_static("*"));
    }
    resp
}

pub async fn iniciar_servidores() {
    // Inicializar o provedor criptográfico Ring para o Rustls 0.23
    let _ = rustls::crypto::ring::default_provider().install_default();

    let port_http = std::env::var("PORTA_HTTP")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9096);

    let port_https = std::env::var("PORTA_HTTPS")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9196);

    let app = Router::new()
        .route(
            "/.info",
            get(handlers::info_handler).post(handlers::info_handler),
        )
        .route(
            "/.pList",
            get(handlers::plist_handler).post(handlers::plist_handler),
        )
        .route(
            "/.pDefaultPrinter",
            get(handlers::pdefault_printer_handler).post(handlers::pdefault_printer_handler),
        )
        .route(
            "/.systemInfo",
            get(handlers::system_info_handler).post(handlers::system_info_handler),
        )
        .route(
            "/.localPrinting",
            get(handlers::local_printing_handler).post(handlers::local_printing_handler),
        )
        .route(
            "/.impressaoDiretaTexto",
            get(handlers::impressao_direta_texto_handler).post(handlers::impressao_direta_texto_handler),
        )
        .fallback(handlers::fallback_handler)
        .layer(middleware::from_fn(cors_and_pna_middleware));

    println!("=====================================================");
    println!("  Sankhya Impressão Rápida v1.0.2 (Rust)");
    println!("  Substituto leve e de alta performance do Web Connection");
    println!("  Servidor HTTP  em http://127.0.0.1:{}", port_http);
    println!("  Servidor HTTPS em https://127.0.0.1:{}", port_https);
    println!("=====================================================");

    let tls_config = RustlsConfig::from_pem(CERT_PEM.to_vec(), KEY_PEM.to_vec())
        .await
        .expect("Falha ao carregar certificados TLS embutidos");

    let addr_http = SocketAddr::from(([127, 0, 0, 1], port_http));
    let addr_https = SocketAddr::from(([127, 0, 0, 1], port_https));

    let app_http = app.clone();
    let app_https = app;

    // Servidor HTTP porta 9096 em background
    tokio::spawn(async move {
        match tokio::net::TcpListener::bind(addr_http).await {
            Ok(listener) => {
                let _ = axum::serve(listener, app_http).await;
            }
            Err(e) => {
                eprintln!("Aviso: Falha ao abrir porta HTTP {}: {:?}", port_http, e);
            }
        }
    });

    // Servidor HTTPS porta 9196 principal
    if let Err(e) = axum_server::bind_rustls(addr_https, tls_config)
        .serve(app_https.into_make_service())
        .await
    {
        eprintln!("Erro no servidor HTTPS: {:?}", e);
    }
}

fn instalar_certificado_raiz() {
    println!("Instalando certificado SSL na lista de Autoridades Confiáveis do Windows...");
    let temp_cert = std::env::temp_dir().join("sankhya_cert_temp.pem");
    if std::fs::write(&temp_cert, CERT_PEM).is_ok() {
        let _ = std::process::Command::new("certutil.exe")
            .args(["-addstore", "-f", "Root", temp_cert.to_str().unwrap()])
            .status();
        let _ = std::fs::remove_file(temp_cert);
    }
}

fn elevar_uac(comando: &str) -> bool {
    let exe = std::env::current_exe().unwrap();
    let script = format!(
        "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -Wait",
        exe.display(),
        comando
    );
    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .status();
    status.map(|s| s.success()).unwrap_or(false)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "install" | "--install" => {
            println!("Registrando o serviço no Windows...");
            let exe_path = std::env::current_exe().unwrap();
            let bin_path = format!("\"{}\" --service", exe_path.display());

            instalar_certificado_raiz();

            let status = std::process::Command::new("sc.exe")
                .args([
                    "create",
                    service::SERVICE_NAME,
                    &format!("binPath={}", bin_path),
                    "start=auto",
                    &format!("DisplayName={}", service::SERVICE_DISPLAY_NAME),
                ])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("Serviço '{}' registrado com sucesso!", service::SERVICE_NAME);
                    println!("Iniciando o serviço agora...");
                    let _ = std::process::Command::new("sc.exe")
                        .args(["start", service::SERVICE_NAME])
                        .status();
                    println!("Tudo pronto! O serviço está ativo em segundo plano.");
                }
                _ => {
                    println!("Solicitando permissão de Administrador (UAC)...");
                    if !elevar_uac("install") {
                        eprintln!("Falha ao registrar serviço. Execute o terminal como Administrador.");
                    }
                }
            }
        }
        "uninstall" | "--uninstall" => {
            println!("Removendo o serviço do Windows...");
            let _ = std::process::Command::new("sc.exe")
                .args(["stop", service::SERVICE_NAME])
                .status();

            let status = std::process::Command::new("sc.exe")
                .args(["delete", service::SERVICE_NAME])
                .status();

            match status {
                Ok(s) if s.success() => println!("Serviço '{}' removido com sucesso!", service::SERVICE_NAME),
                _ => {
                    println!("Solicitando permissão de Administrador (UAC)...");
                    if !elevar_uac("uninstall") {
                        eprintln!("Falha ao remover o serviço.");
                    }
                }
            }
        }
        "service" | "--service" => {
            if let Err(e) = service::start_dispatcher() {
                eprintln!("Erro no despachante do serviço: {:?}", e);
            }
        }
        _ => {
            println!("Iniciando em modo direto (pressione Ctrl+C para encerrar)...");
            println!("Dica: Para instalar como serviço silencioso do Windows, execute com 'install'.");
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(iniciar_servidores());
        }
    }
}
