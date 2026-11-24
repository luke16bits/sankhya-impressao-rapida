use axum::{
    extract::{Query, RawQuery},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Rota GET /.info
pub async fn info_handler(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    if let Some(val) = params.get("webcn") {
        if val == "true" {
            return (StatusCode::OK, "false").into_response();
        }
    }
    (StatusCode::OK, "dados do sistema:  versao: 1.0 uptime: 0").into_response()
}

/// Rota GET /.pList
pub async fn plist_handler() -> impl IntoResponse {
    let impressoras = printer::listar_impressoras();
    (StatusCode::OK, impressoras)
}

/// Rota GET /.pDefaultPrinter
pub async fn pdefault_printer_handler() -> impl IntoResponse {
    let padrao = printer::impressora_padrao();
    (StatusCode::OK, padrao)
}

/// Rota GET /.systemInfo
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

    Json(sys_info)
}

#[derive(Deserialize)]
pub struct PrintParams {
    pub params: Option<String>,
}

/// Rota GET /.localPrinting?params=JSESSIONID|TRANSACTION_ID
pub async fn local_printing_handler(
    headers: HeaderMap,
    Query(params): Query<PrintParams>,
) -> impl IntoResponse {
    let raw_params = match params.params {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "Parâmetros não informados").into_response(),
    };

    let parts: Vec<&str> = raw_params.split('|').collect();
    if parts.len() < 2 {
        return (StatusCode::BAD_REQUEST, "Formato de parâmetros inválido").into_response();
    }

    let session_id = parts[0];
    let transaction_id = parts[1];

    let origin = headers
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    println!(
        "[Impressão] Solicitada: Transaction={}, Origin={}, Session={}",
        transaction_id, origin, session_id
    );

    // Se tivermos a URL de origem, busca o job de impressão no servidor Sankhya
    if !origin.is_empty() {
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
    Query(params): Query<PrintParams>,
    body: Option<String>,
) -> impl IntoResponse {
    let texto = body.unwrap_or_else(|| {
        params
            .params
            .unwrap_or_default()
            .split('|')
            .nth(1)
            .unwrap_or("")
            .to_string()
    });

    if !texto.is_empty() {
        let padrao = printer::impressora_padrao();
        if !padrao.is_empty() {
            let _ = printer::imprimir_raw(&padrao, "Sankhya Texto", texto.as_bytes());
        }
    }

    (StatusCode::OK, "Impressão direta enviada.").into_response()
}
