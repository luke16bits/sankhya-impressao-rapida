mod handlers;
mod printer;
mod service;

use axum::{
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

// Embutir o certificado e chave privada direto dentro do binário (Single Executable)
const CERT_PEM: &[u8] = include_bytes!("../certs/cert.pem");
const KEY_PEM: &[u8] = include_bytes!("../certs/key.pem");

pub async fn iniciar_servidor_https() {
    let port = std::env::var("PORTA_PRINT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9096);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([axum::http::HeaderName::from_static("access-control-allow-private-network")]);

    let app = Router::new()
        .route("/.info", get(handlers::info_handler))
        .route("/.pList", get(handlers::plist_handler))
        .route("/.pDefaultPrinter", get(handlers::pdefault_printer_handler))
        .route("/.systemInfo", get(handlers::system_info_handler))
        .route("/.localPrinting", get(handlers::local_printing_handler))
        .route(
            "/.impressaoDiretaTexto",
            get(handlers::impressao_direta_texto_handler).post(handlers::impressao_direta_texto_handler),
        )
        .layer(cors);

    println!("=====================================================");
    println!("  Sankhya Impressão Rápida v1.0.0 (Rust)");
    println!("  Substituto leve e de alta performance do Web Connection");
    println!("  Aguardando conexões HTTPS em https://{}", addr);
    println!("=====================================================");

    let tls_config = RustlsConfig::from_pem(CERT_PEM.to_vec(), KEY_PEM.to_vec())
        .await
        .expect("Falha ao carregar certificados TLS embutidos");

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "install" | "--install" => {
            println!("Registrando o serviço no Windows...");
            let exe_path = std::env::current_exe().unwrap();
            let bin_path = format!("\"{}\" --service", exe_path.display());

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
                }
                _ => eprintln!("Falha ao registrar serviço. Certifique-se de executar como Administrador."),
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
                _ => eprintln!("Falha ao remover o serviço."),
            }
        }
        "service" | "--service" => {
            // Chamado pelo Gerenciador de Serviços do Windows
            if let Err(e) = service::start_dispatcher() {
                eprintln!("Erro no despachante do serviço: {:?}", e);
            }
        }
        _ => {
            // Modo console interativo
            println!("Iniciando em modo direto (pressione Ctrl+C para encerrar)...");
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(iniciar_servidor_https());
        }
    }
}
