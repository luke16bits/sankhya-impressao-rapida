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

const JASPER_TO_PDF_CLASS: &[u8] = include_bytes!("../helpers/JasperToPdf.class");

/// Converte JasperPrint XML em PDF utilizando o helper Java e snklocalapp.jar
fn converter_jasper_para_pdf(xml_data: &[u8]) -> Option<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let class_file = temp_dir.join("JasperToPdf.class");
    if !class_file.exists() {
        let _ = std::fs::write(&class_file, JASPER_TO_PDF_CLASS);
    }

    let timestamp = chrono_or_simple_time();
    let temp_xml = temp_dir.join(format!("snk_doc_{}.xml", timestamp));
    let temp_pdf = temp_dir.join(format!("snk_doc_{}.pdf", timestamp));

    if std::fs::write(&temp_xml, xml_data).is_err() {
        return None;
    }

    let java_candidates = [
        "C:\\SankhyaW\\WebConnection\\Sankhya_web_connectionx64\\jre\\bin\\java.exe",
        "C:\\Program Files\\Sankhya Impressao Rapida\\jre\\bin\\java.exe",
        "java",
    ];

    let jar_candidates = [
        "C:\\SankhyaW\\WebConnection\\Sankhya_web_connectionx64\\executar\\build\\snklocalapp.jar",
        "C:\\Program Files\\Sankhya Impressao Rapida\\snklocalapp.jar",
    ];

    let mut java_exe = "java";
    for j in &java_candidates {
        if std::path::Path::new(j).exists() {
            java_exe = *j;
            break;
        }
    }

    let mut jar_path = None;
    for jar in &jar_candidates {
        if std::path::Path::new(jar).exists() {
            jar_path = Some(*jar);
            break;
        }
    }

    if let Some(jar) = jar_path {
        let cp = format!("{};{}", temp_dir.display(), jar);
        let status = std::process::Command::new(java_exe)
            .args(["-cp", &cp, "JasperToPdf", temp_xml.to_str().unwrap(), temp_pdf.to_str().unwrap()])
            .status();

        let _ = std::fs::remove_file(&temp_xml);

        if let Ok(s) = status {
            if s.success() && temp_pdf.exists() {
                if let Ok(pdf_bytes) = std::fs::read(&temp_pdf) {
                    let _ = std::fs::remove_file(&temp_pdf);
                    log_msg(&format!(
                        "[localPrinting] Conversão JasperXML -> PDF concluída com sucesso! ({} bytes)",
                        pdf_bytes.len()
                    ));
                    return Some(pdf_bytes);
                }
            }
        }
        let _ = std::fs::remove_file(&temp_pdf);
    } else {
        log_msg("[localPrinting] Aviso: snklocalapp.jar não encontrado para converter JasperXML.");
        let _ = std::fs::remove_file(&temp_xml);
    }

    None
}

/// Abre janela nativa do sistema operacional para o usuário escolher onde salvar o PDF
#[cfg(windows)]
fn pedir_caminho_salvar_pdf(sugestao_nome: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::ptr::null_mut;

    #[repr(C)]
    struct OPENFILENAMEW {
        lStructSize: u32,
        hwndOwner: *mut std::ffi::c_void,
        hInstance: *mut std::ffi::c_void,
        lpstrFilter: *const u16,
        lpstrCustomFilter: *mut u16,
        nMaxCustFilter: u32,
        nFilterIndex: u32,
        lpstrFile: *mut u16,
        nMaxFile: u32,
        lpstrFileTitle: *mut u16,
        nMaxFileTitle: u32,
        lpstrInitialDir: *const u16,
        lpstrTitle: *const u16,
        Flags: u32,
        nFileOffset: u16,
        nFileExtension: u16,
        lpstrDefExt: *const u16,
        lCustData: usize,
        lpfnHook: *mut std::ffi::c_void,
        lpTemplateName: *const u16,
        pvReserved: *mut std::ffi::c_void,
        dwReserved: u32,
        FlagsEx: u32,
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn GetSaveFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
    }

    let filter_str = "PDF Document (*.pdf)\0*.pdf\0All Files (*.*)\0*.*\0\0";
    let filter: Vec<u16> = filter_str.encode_utf16().collect();
    let title: Vec<u16> = OsStr::new("Save Print Output As\0").encode_wide().collect();
    let def_ext: Vec<u16> = OsStr::new("pdf\0").encode_wide().collect();

    let mut file_buf = vec![0u16; 1024];
    let sugestao_wide: Vec<u16> = OsStr::new(sugestao_nome).encode_wide().collect();
    for (i, &w) in sugestao_wide.iter().take(1023).enumerate() {
        file_buf[i] = w;
    }

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: null_mut(),
        hInstance: null_mut(),
        lpstrFilter: filter.as_ptr(),
        lpstrCustomFilter: null_mut(),
        nMaxCustFilter: 0,
        nFilterIndex: 1,
        lpstrFile: file_buf.as_mut_ptr(),
        nMaxFile: file_buf.len() as u32,
        lpstrFileTitle: null_mut(),
        nMaxFileTitle: 0,
        lpstrInitialDir: null_mut(),
        lpstrTitle: title.as_ptr(),
        Flags: 0x00000002 | 0x00000800 | 0x00000008, // OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR
        nFileOffset: 0,
        nFileExtension: 0,
        lpstrDefExt: def_ext.as_ptr(),
        lCustData: 0,
        lpfnHook: null_mut(),
        lpTemplateName: null_mut(),
        pvReserved: null_mut(),
        dwReserved: 0,
        FlagsEx: 0,
    };

    let ok = unsafe { GetSaveFileNameW(&mut ofn) };
    if ok != 0 {
        let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
        let os_str = std::ffi::OsString::from_wide(&file_buf[..len]);
        if let Ok(path) = os_str.into_string() {
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

    None
}

#[cfg(not(windows))]
fn pedir_caminho_salvar_pdf(sugestao_nome: &str) -> Option<String> {
    let output = std::process::Command::new("zenity")
        .args([
            "--file-selection",
            "--save",
            "--confirm-overwrite",
            "--file-filter=PDF Document (*.pdf) | *.pdf",
            &format!("--filename={}", sugestao_nome),
            "--title=Save Print Output As",
        ])
        .output();

    if let Ok(out) = output {
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Serializa um array de String para o formato nativo Java Serialization ([Ljava.lang.String;)
fn serialize_string_array(items: &[&str]) -> Vec<u8> {
    let mut payload = vec![
        0xac, 0xed, 0x00, 0x05, // Magic header (STREAM_MAGIC + STREAM_VERSION)
        0x75, 0x72,             // TC_ARRAY, TC_CLASSDESC
        0x00, 0x13,             // Name length (19)
        0x5b, 0x4c, 0x6a, 0x61, 0x76, 0x61, 0x2e, 0x6c, 0x61, 0x6e, 0x67, 0x2e, 0x53, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x3b, // [Ljava.lang.String;
        0xad, 0xd2, 0x56, 0xe7, 0xe9, 0x1d, 0x7b, 0x47, // serialVersionUID
        0x02, 0x00, 0x00,       // flags (SC_SERIALIZABLE), 0 fields
        0x78, 0x70,             // TC_ENDBLOCKDATA, TC_NULL
        ((items.len() >> 24) & 0xff) as u8,
        ((items.len() >> 16) & 0xff) as u8,
        ((items.len() >> 8) & 0xff) as u8,
        (items.len() & 0xff) as u8,
    ];
    for s in items {
        let b = s.as_bytes();
        payload.push(0x74); // TC_STRING
        payload.push(((b.len() >> 8) & 0xff) as u8);
        payload.push((b.len() & 0xff) as u8);
        payload.extend_from_slice(b);
    }
    payload
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
            "{}/mge/impressaoNotaServlet.mge?acao=doLocalPrinting&incluirJobsEnviados=true",
            origin
        );

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build();

        if let Ok(http_client) = client {
            let body = serialize_string_array(&[&transaction_id]);
            let mut req = http_client
                .post(&print_url)
                .header("Content-Type", "application/x-java-serialized-object")
                .body(body);

            if !session_id.is_empty() {
                req = req.header("Cookie", format!("JSESSIONID={}", session_id));
            }

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            log_msg(&format!(
                                "[localPrinting] Recebidos {} bytes do servlet Sankhya.",
                                bytes.len()
                            ));

                            let mut dados_finais = bytes.to_vec();

                            // Descompactar ZIP do Sankhya (ex: ImpressaoNotaServletResponse)
                            if dados_finais.starts_with(b"PK\x03\x04") {
                                let mut decompressed_opt = None;
                                {
                                    let reader = std::io::Cursor::new(&dados_finais);
                                    if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                                        if archive.len() > 0 {
                                            if let Ok(mut file) = archive.by_index(0) {
                                                let mut decompressed = Vec::new();
                                                if std::io::copy(&mut file, &mut decompressed).is_ok() {
                                                    decompressed_opt = Some((decompressed, file.name().to_string()));
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some((decompressed, file_name)) = decompressed_opt {
                                    log_msg(&format!(
                                        "[localPrinting] ZIP descompactado! {} bytes compactados -> {} bytes (arquivo: {})",
                                        dados_finais.len(),
                                        decompressed.len(),
                                        file_name
                                    ));
                                    dados_finais = decompressed;
                                }
                            }

                            // 2. Extrair PDF puro se embutido dentro do objeto serializado
                            let mut pdf_bytes: Option<Vec<u8>> = None;
                            if let Some(pos) = dados_finais.windows(5).position(|w| w == b"%PDF-") {
                                let slice = &dados_finais[pos..];
                                if let Some(eof_pos) = slice.windows(5).rposition(|w| w == b"%%EOF") {
                                    pdf_bytes = Some(slice[..eof_pos + 5].to_vec());
                                } else {
                                    pdf_bytes = Some(slice.to_vec());
                                }
                                log_msg(&format!(
                                    "[localPrinting] Documento PDF puro extraído com sucesso! ({} bytes)",
                                    pdf_bytes.as_ref().unwrap().len()
                                ));
                            }

                            // 3. Se não achou PDF direto, verifica se dentro há stream GZIP (Jasper XML)
                            if pdf_bytes.is_none() {
                                if let Some(pos) = dados_finais.windows(2).position(|w| w == [0x1f, 0x8b]) {
                                    use flate2::read::GzDecoder;
                                    let mut decoder = GzDecoder::new(&dados_finais[pos..]);
                                    let mut decompressed_doc = Vec::new();
                                    if decoder.read_to_end(&mut decompressed_doc).is_ok() && !decompressed_doc.is_empty() {
                                        log_msg(&format!(
                                            "[localPrinting] Stream GZIP descompactado! ({} bytes)",
                                            decompressed_doc.len()
                                        ));
                                        dados_finais = decompressed_doc;
                                    }
                                }

                                // Se o conteúdo é Jasper XML, converter para PDF com o JasperToPdf
                                let is_jasper = dados_finais.starts_with(b"<?xml")
                                    || dados_finais.starts_with(b"<!DOCTYPE jasperPrint")
                                    || dados_finais.windows(12).any(|w| w == b"<jasperPrint");

                                if is_jasper {
                                    log_msg("[localPrinting] JasperPrint XML detectado. Iniciando conversão para PDF...");
                                    if let Some(converted) = converter_jasper_para_pdf(&dados_finais) {
                                        pdf_bytes = Some(converted);
                                    }
                                }
                            }

                            // 4. Despacho inteligente para impressão ou abertura de documento
                            let is_pdf_printer = printer_chosen.to_lowercase().contains("pdf")
                                || printer_chosen.to_lowercase().contains("xps")
                                || printer_chosen.is_empty();

                            if let Some(pdf) = pdf_bytes {
                                if is_pdf_printer {
                                    let sugestao = format!("Sankhya_Doc_{}.pdf", chrono_or_simple_time());
                                    log_msg("[localPrinting] Solicitando ao usuário onde salvar o arquivo PDF...");
                                    let destino = pedir_caminho_salvar_pdf(&sugestao);

                                    if let Some(caminho) = destino {
                                        if std::fs::write(&caminho, &pdf).is_ok() {
                                            log_msg(&format!("[localPrinting] PDF salvo com sucesso em: {}", caminho));
                                            #[cfg(windows)]
                                            let _ = std::process::Command::new("cmd")
                                                .args(["/c", "start", "", &caminho])
                                                .spawn();
                                            #[cfg(not(windows))]
                                            let _ = std::process::Command::new("xdg-open")
                                                .arg(&caminho)
                                                .spawn();
                                        }
                                    } else {
                                        // Fallback se o diálogo foi cancelado ou fechado
                                        #[cfg(windows)]
                                        let base_dir = std::env::var("USERPROFILE")
                                            .map(|p| format!("{}/Downloads", p))
                                            .unwrap_or_else(|_| "C:/Users/Public/Downloads".into());
                                        #[cfg(not(windows))]
                                        let base_dir = std::env::var("HOME")
                                            .map(|p| format!("{}/Downloads", p))
                                            .unwrap_or_else(|_| "/tmp".into());

                                        let _ = std::fs::create_dir_all(&base_dir);
                                        let fallback_file = format!("{}/{}", base_dir, sugestao);
                                        if std::fs::write(&fallback_file, &pdf).is_ok() {
                                            log_msg(&format!(
                                                "[localPrinting] Diálogo cancelado ou indisponível. Arquivo salvo em Downloads: {}",
                                                fallback_file
                                            ));
                                        }
                                    }
                                } else {
                                    // Envio para impressora física (Brother, etc.)
                                    #[cfg(windows)]
                                    let base_dir = std::env::var("USERPROFILE")
                                        .map(|p| format!("{}/Downloads", p))
                                        .unwrap_or_else(|_| "C:/Users/Public/Downloads".into());
                                    #[cfg(not(windows))]
                                    let base_dir = std::env::var("HOME")
                                        .map(|p| format!("{}/Downloads", p))
                                        .unwrap_or_else(|_| "/tmp".into());

                                    let _ = std::fs::create_dir_all(&base_dir);
                                    let doc_file = format!("{}/Sankhya_Doc_{}.pdf", base_dir, chrono_or_simple_time());

                                    if std::fs::write(&doc_file, &pdf).is_ok() {
                                        log_msg(&format!("[localPrinting] PDF preparado para envio físico em: {}", doc_file));

                                        #[cfg(windows)]
                                        {
                                            let print_cmd = format!(
                                                "Start-Process -FilePath '{}' -Verb PrintTo -ArgumentList '\"{}\"'",
                                                doc_file.replace('/', "\\"),
                                                printer_chosen
                                            );
                                            log_msg(&format!("[localPrinting] Enviando PDF para impressora '{}' via Windows Shell...", printer_chosen));
                                            let _ = std::process::Command::new("powershell")
                                                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &print_cmd])
                                                .spawn();
                                        }
                                        #[cfg(not(windows))]
                                        {
                                            log_msg(&format!("[localPrinting] Enviando PDF para CUPS '{}'", printer_chosen));
                                            let _ = std::process::Command::new("lp")
                                                .arg("-d")
                                                .arg(&printer_chosen)
                                                .arg(&doc_file)
                                                .status();
                                        }
                                    }
                                }
                            } else {
                                // Cupom fiscal ou dados brutos (ESC/POS, ZPL)
                                log_msg(&format!(
                                    "[localPrinting] Enviando {} bytes brutos para impressora '{}'...",
                                    dados_finais.len(), printer_chosen
                                ));
                                match printer::imprimir_raw(&printer_chosen, "Sankhya Documento", &dados_finais) {
                                    Ok(res) => log_msg(&format!("[localPrinting] Sucesso no spooler: {}", res)),
                                    Err(err) => log_msg(&format!("[localPrinting] Erro no driver: {}", err)),
                                }
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

/// Rota GET /.checkUpdate e /.update
pub async fn check_update_handler() -> impl IntoResponse {
    let client = match reqwest::Client::builder()
        .user_agent("SankhyaImpressaoRapida")
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao criar cliente HTTP").into_response(),
    };

    let url = "https://api.github.com/repos/luke16bits/sankhya-impressao-rapida/releases/latest";
    let versao_atual = format!("v{}", env!("CARGO_PKG_VERSION"));

    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(tag) = json.get("tag_name").and_then(|t| t.as_str()) {
                let html_url = json.get("html_url").and_then(|u| u.as_str()).unwrap_or("");
                let resp_json = serde_json::json!({
                    "versao_atual": versao_atual,
                    "versao_recente": tag,
                    "atualizado": (tag == versao_atual),
                    "download_url": html_url
                });
                return Json(resp_json).into_response();
            }
        }
    }

    (StatusCode::OK, format!("Versão atual: {}", versao_atual)).into_response()
}

/// Verifica atualizações no GitHub silenciosamente em background
pub async fn verificar_atualizacoes_github() {
    let client = match reqwest::Client::builder()
        .user_agent("SankhyaImpressaoRapida")
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let url = "https://api.github.com/repos/luke16bits/sankhya-impressao-rapida/releases/latest";
    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(tag) = json.get("tag_name").and_then(|t| t.as_str()) {
                let versao_atual = format!("v{}", env!("CARGO_PKG_VERSION"));
                if tag != versao_atual && !versao_atual.contains(tag) {
                    log_msg(&format!(
                        "[Update] Nova versão disponível: {} (versão instalada: {})! Baixe a atualização em: https://github.com/luke16bits/sankhya-impressao-rapida/releases",
                        tag, versao_atual
                    ));
                } else {
                    log_msg(&format!("[Update] O sistema está rodando a versão mais recente ({}).", versao_atual));
                }
            }
        }
    }
}
