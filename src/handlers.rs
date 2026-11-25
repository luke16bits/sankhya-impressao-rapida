use axum::{
    http::{HeaderMap, StatusCode, Uri},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::printer;

#[derive(Serialize)]
pub struct SystemInfoResponse {
    #[serde(rename = "infoPC")]
    pub info_pc: String,
    #[serde(rename = "clientIp")]
    pub client_ip: String,
    #[serde(rename = "clientName")]
    pub client_name: String,
    #[serde(rename = "clientMAC")]
    pub client_mac: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "domainName")]
    pub domain_name: String,
}

/// Rota GET/POST /.info
pub async fn info_handler(uri: Uri) -> impl IntoResponse {
    if uri.query().map(|q| q.contains("webcn=true")).unwrap_or(false) {
        return (StatusCode::OK, "false").into_response();
    }
    (StatusCode::OK, "dados do sistema:  versao: 1.0 uptime: 0").into_response()
}

/// Rota GET/POST /.pList
pub async fn plist_handler() -> impl IntoResponse {
    let impressoras = printer::listar_impressoras();
    (StatusCode::OK, impressoras).into_response()
}

/// Rota GET/POST /.pDefaultPrinter
pub async fn pdefault_printer_handler() -> impl IntoResponse {
    let padrao = printer::impressora_padrao();
    (StatusCode::OK, padrao).into_response()
}

/// Rota GET/POST /.systemInfo
pub async fn system_info_handler() -> impl IntoResponse {
    let client_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "LOCALHOST".into());
    let user_name = std::env::var("USERNAME").unwrap_or_else(|_| "USUARIO".into());
    let domain_name = std::env::var("USERDOMAIN").unwrap_or_else(|_| "WORKGROUP".into());

    let sys_info = SystemInfoResponse {
        info_pc: format!("{} ({})", client_name, user_name),
        client_ip: "127.0.0.1".to_string(),
        client_name,
        client_mac: "00:00:00:00:00:00".to_string(),
        user_name,
        domain_name,
    };

    Json(sys_info).into_response()
}

/// Rota GET/POST /.localPrinting
pub async fn local_printing_handler(
    headers: HeaderMap,
    uri: Uri,
) -> impl IntoResponse {
    let raw_query = uri.query().unwrap_or("");
    let mut session_id = String::new();
    let mut transaction_id = String::new();

    // Tentar extrair do formato ?params=JSESSIONID|TRANSACTION_ID
    if let Some(idx) = raw_query.find("params=") {
        let p_val = &raw_query[idx + 7..];
        let p_end = p_val.find('&').unwrap_or(p_val.len());
        let val_encoded = &p_val[..p_end];
        let decoded = urlencoding_decode(val_encoded);
        let parts: Vec<&str> = decoded.split('|').collect();
        if parts.len() >= 2 {
            session_id = parts[0].to_string();
            transaction_id = parts[1].to_string();
        }
    }

    let origin = headers
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    println!(
        "[Impressão] Solicitada: Transaction={}, Origin={}, Session={}",
        transaction_id, origin, session_id
    );

    if !origin.is_empty() && !transaction_id.is_empty() {
        let print_url = format!(
            "{}/mge/impressaoNotaServlet.mge?acao=execLocalPrint&transactionIds={}&incluirJobsEnviados=true",
            origin, transaction_id
        );

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build();

        if let Ok(http_client) = client {
            let res = http_client
                .get(&print_url)
                .header("Cookie", format!("JSESSIONID={}", session_id))
                .send()
                .await;

            if let Ok(resp) = res {
                if let Ok(bytes) = resp.bytes().await {
                    let padrao = printer::impressora_padrao();
                    if !padrao.is_empty() {
                        let _ = printer::imprimir_raw(&padrao, "Sankhya Doc", &bytes);
                    }
                }
            }
        }
    }

    (StatusCode::OK, "Servidor realizando impressão.").into_response()
}

/// Rota GET/POST /.impressaoDiretaTexto
pub async fn impressao_direta_texto_handler(
    body: Option<String>,
) -> impl IntoResponse {
    if let Some(texto) = body {
        if !texto.is_empty() {
            let padrao = printer::impressora_padrao();
            if !padrao.is_empty() {
                let _ = printer::imprimir_raw(&padrao, "Sankhya Texto", texto.as_bytes());
            }
        }
    }
    (StatusCode::OK, "Impressão direta realizada.").into_response()
}

/// Rota de Fallback para qualquer outro endpoint do Sankhya
pub async fn fallback_handler(uri: Uri) -> impl IntoResponse {
    println!("[Fallback] Requisição recebida em: {}", uri);
    (StatusCode::OK, "dados do sistema:  versao: 1.0 uptime: 0").into_response()
}

fn urlencoding_decode(val: &str) -> String {
    let mut result = String::new();
    let mut chars = val.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}
