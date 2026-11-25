use axum::{
    http::{HeaderMap, StatusCode, Uri},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::io::{Read, Write};
use std::sync::Mutex;

use crate::printer;

static LOG_MUTEX: Mutex<()> = Mutex::new(());

/// Grava mensagem de log no console e em arquivo de log no disco
pub fn log_msg(msg: &str) {
    let now = chrono_or_simple_time();
    let formatted = format!("[{}] {}", now, msg);
    println!("{}", formatted);

    if let Ok(_lock) = LOG_MUTEX.lock() {
        #[cfg(windows)]
        let log_dir = "C:\\ProgramData\\SankhyaImpressaoRapida";
        #[cfg(not(windows))]
        let log_dir = "/tmp";

        let _ = std::fs::create_dir_all(log_dir);
        let log_path = format!("{}/service.log", log_dir);

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(file, "{}", formatted);
        }
    }
}

fn chrono_or_simple_time() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

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
    log_msg(&format!("Requisição recebida em: {}", uri));
    if uri.query().map(|q| q.contains("webcn=true")).unwrap_or(false) {
        return (StatusCode::OK, "false").into_response();
    }
    (StatusCode::OK, "dados do sistema:  versao: 1.0 uptime: 0").into_response()
}

/// Rota GET/POST /.pList
pub async fn plist_handler() -> impl IntoResponse {
    let impressoras = printer::listar_impressoras();
    log_msg(&format!("[pList] Impressoras encontradas: {}", impressoras));
    (StatusCode::OK, impressoras).into_response()
}

/// Rota GET/POST /.pDefaultPrinter
pub async fn pdefault_printer_handler() -> impl IntoResponse {
    let padrao = printer::impressora_padrao();
    log_msg(&format!("[pDefaultPrinter] Impressora padrão: {}", padrao));
    (StatusCode::OK, padrao).into_response()
}

/// Rota GET/POST /.systemInfo
pub async fn system_info_handler() -> impl IntoResponse {
    let client_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "LOCALHOST".into());
    let user_name = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "USUARIO".into());
    let domain_name = std::env::var("USERDOMAIN").unwrap_or_else(|_| "WORKGROUP".into());

    let sys_info = SystemInfoResponse {
        info_pc: format!("{} ({})", client_name, user_name),
        client_ip: "127.0.0.1".to_string(),
        client_name,
        client_mac: "00:00:00:00:00:00".to_string(),
        user_name,
        domain_name,
    };

    log_msg(&format!("[systemInfo] Respondendo identificação: {}", sys_info.info_pc));
    Json(sys_info).into_response()
}

/// Rota GET/POST /.localPrinting
pub async fn local_printing_handler(
    headers: HeaderMap,
    uri: Uri,
) -> impl IntoResponse {
    let raw_query = uri.query().unwrap_or("");
    log_msg(&format!("[localPrinting] Requisição recebida: query={}", raw_query));

    let mut session_id = String::new();
    let mut transaction_id = String::new();
    let mut printer_chosen = String::new();

    // Extrair o parâmetro ?params=
    if let Some(idx) = raw_query.find("params=") {
        let p_val = &raw_query[idx + 7..];
        let p_end = p_val.find('&').unwrap_or(p_val.len());
        let val_encoded = &p_val[..p_end];
        let decoded = urlencoding_decode(val_encoded);

        // Tentar decodificar Base64
        if let Some(base64_decoded) = base64_decode(&decoded) {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&base64_decoded) {
                log_msg(&format!("[localPrinting] JSON decodificado de Base64: {:?}", json));
                if let Some(s) = json.get("jsessionid").and_then(|v| v.as_str()) {
                    session_id = s.to_string();
                }
                if let Some(t) = json.get("transactionIds").or_else(|| json.get("transactionId")).and_then(|v| v.as_str()) {
                    transaction_id = t.to_string();
                }
                if let Some(p) = json.get("printer").or_else(|| json.get("printerName")).and_then(|v| v.as_str()) {
                    printer_chosen = p.to_string();
                }
            }
        }

        // Se não foi Base64 JSON, tenta formato delimitado por pipe (sessionId|transactionId|printer)
        if transaction_id.is_empty() {
            let parts: Vec<&str> = decoded.split('|').collect();
            if parts.len() >= 2 {
                session_id = parts[0].to_string();
                transaction_id = parts[1].to_string();
                if parts.len() >= 3 {
                    printer_chosen = parts[2].to_string();
                }
            }
        }
    }

    if printer_chosen.is_empty() {
        printer_chosen = printer::impressora_padrao();
    }

    let origin = headers
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    log_msg(&format!(
        "[localPrinting] Processando: Transaction='{}', Impressora='{}', Origin='{}'",
        transaction_id, printer_chosen, origin
    ));

    if !origin.is_empty() && !transaction_id.is_empty() {
        let print_url = format!(
            "{}/mge/impressaoNotaServlet.mge?acao=execLocalPrint&transactionIds={}&incluirJobsEnviados=true",
            origin, transaction_id
        );

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build();

        if let Ok(http_client) = client {
            let mut req = http_client.get(&print_url);
            if !session_id.is_empty() {
                req = req.header("Cookie", format!("JSESSIONID={}", session_id));
            }

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            log_msg(&format!(
                                "[localPrinting] Recebidos {} bytes do Sankhya.",
                                bytes.len()
                            ));

                            let mut dados_finais = bytes.to_vec();

                            // Descompactar ZIP do Sankhya (ex: ImpressaoNotaServletResponse)
                            if dados_finais.starts_with(b"PK\x03\x04") {
                                let reader = std::io::Cursor::new(&dados_finais);
                                if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                                    if archive.len() > 0 {
                                        if let Ok(mut file) = archive.by_index(0) {
                                            let mut decompressed = Vec::new();
                                            if std::io::copy(&mut file, &mut decompressed).is_ok() {
                                                log_msg(&format!(
                                                    "[localPrinting] ZIP descompactado com sucesso! {} bytes compactados -> {} bytes reais (arquivo: {})",
                                                    dados_finais.len(),
                                                    decompressed.len(),
                                                    file.name()
                                                ));
                                                dados_finais = decompressed;
                                            }
                                        }
                                    }
                                }
                            }

                            log_msg(&format!(
                                "[localPrinting] Enviando {} bytes para impressora '{}'...",
                                dados_finais.len(), printer_chosen
                            ));

                            match printer::imprimir_raw(&printer_chosen, "Sankhya DANFE", &dados_finais) {
                                Ok(res) => log_msg(&format!("[localPrinting] Sucesso: {}", res)),
                                Err(err) => log_msg(&format!("[localPrinting] Erro no driver: {}", err)),
                            }
                        }
                    } else {
                        log_msg(&format!("[localPrinting] Servlet retornou status HTTP: {}", status));
                    }
                }
                Err(e) => {
                    log_msg(&format!("[localPrinting] Falha ao baixar job do servlet: {:?}", e));
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
            log_msg(&format!("[impressaoDiretaTexto] Enviando {} bytes para {}", texto.len(), padrao));
            let _ = printer::imprimir_raw(&padrao, "Sankhya Texto", texto.as_bytes());
        }
    }
    (StatusCode::OK, "Impressão direta realizada.").into_response()
}

/// Rota de Fallback para qualquer outro endpoint do Sankhya (com ou sem ;sessionId=...)
pub async fn fallback_handler(
    headers: HeaderMap,
    uri: Uri,
) -> impl IntoResponse {
    let path = uri.path().to_string();
    log_msg(&format!("[Fallback Dispatcher] Atendendo requisição: path='{}', query='{:?}'", path, uri.query()));

    if path.contains("systemInfo") {
        log_msg("[Fallback -> systemInfo] Respondendo com JSON de identificação");
        return system_info_handler().await.into_response();
    }
    if path.contains("pList") {
        log_msg("[Fallback -> pList] Respondendo com lista de impressoras");
        return plist_handler().await.into_response();
    }
    if path.contains("pDefaultPrinter") {
        log_msg("[Fallback -> pDefaultPrinter] Respondendo com impressora padrão");
        return pdefault_printer_handler().await.into_response();
    }
    if path.contains("localPrinting") {
        log_msg("[Fallback -> localPrinting] Executando impressão");
        return local_printing_handler(headers, uri).await.into_response();
    }
    if path.contains("impressaoDiretaTexto") {
        return impressao_direta_texto_handler(None).await.into_response();
    }

    if uri.query().map(|q| q.contains("webcn=true")).unwrap_or(false) {
        return (StatusCode::OK, "false").into_response();
    }

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

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.is_empty() {
        return None;
    }

    let mut buf = Vec::with_capacity(clean.len() * 3 / 4);
    let mut sextets = [0u8; 4];
    let mut count = 0;

    for b in clean.drain(..) {
        if b == b'=' {
            break;
        }
        let val = TABLE.iter().position(|&x| x == b)? as u8;
        sextets[count] = val;
        count += 1;
        if count == 4 {
            buf.push((sextets[0] << 2) | (sextets[1] >> 4));
            buf.push((sextets[1] << 4) | (sextets[2] >> 2));
            buf.push((sextets[2] << 6) | sextets[3]);
            count = 0;
        }
    }

    if count == 2 {
        buf.push((sextets[0] << 2) | (sextets[1] >> 4));
    } else if count == 3 {
        buf.push((sextets[0] << 2) | (sextets[1] >> 4));
        buf.push((sextets[1] << 4) | (sextets[2] >> 2));
    }

    Some(buf)
}
