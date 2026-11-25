// Módulo de comunicação com o sistema de impressão (Windows Spooler e Linux CUPS)
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use std::ptr::null_mut;

#[cfg(windows)]
#[repr(C)]
struct DOC_INFO_1W {
    pDocName: *const u16,
    pOutputFile: *const u16,
    pDatatype: *const u16,
}

#[cfg(windows)]
#[link(name = "winspool")]
extern "system" {
    fn OpenPrinterW(
        pPrinterName: *const u16,
        phPrinter: *mut *mut c_void,
        pDefault: *const c_void,
    ) -> i32;

    fn StartDocPrinterW(hPrinter: *mut c_void, Level: u32, pDocInfo: *const DOC_INFO_1W) -> u32;
    fn StartPagePrinter(hPrinter: *mut c_void) -> i32;
    fn WritePrinter(
        hPrinter: *mut c_void,
        pBuf: *const c_void,
        cbBuf: u32,
        pcWritten: *mut u32,
    ) -> i32;
    fn EndPagePrinter(hPrinter: *mut c_void) -> i32;
    fn EndDocPrinter(hPrinter: *mut c_void) -> i32;
    fn ClosePrinter(hPrinter: *mut c_void) -> i32;
    fn GetDefaultPrinterW(pszBuffer: *mut u16, pcchBuffer: *mut u32) -> i32;
    fn EnumPrintersW(
        Flags: u32,
        Name: *const u16,
        Level: u32,
        pPrinterEnum: *mut u8,
        cbBuf: u32,
        pcbNeeded: *mut u32,
        pcReturned: *mut u32,
    ) -> i32;
}

#[cfg(windows)]
const PRINTER_ENUM_LOCAL: u32 = 0x00000002;
#[cfg(windows)]
const PRINTER_ENUM_CONNECTIONS: u32 = 0x00000004;

/// Retorna a lista de todas as impressoras instaladas no sistema
pub fn listar_impressoras() -> String {
    #[cfg(windows)]
    {
        let mut needed: u32 = 0;
        let mut returned: u32 = 0;

        unsafe {
            let _ = EnumPrintersW(
                PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
                null_mut(),
                4,
                null_mut(),
                0,
                &mut needed,
                &mut returned,
            );

            if needed == 0 {
                return "Microsoft Print to PDF".to_string();
            }

            let mut buffer = vec![0u8; needed as usize];
            let res = EnumPrintersW(
                PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
                null_mut(),
                4,
                buffer.as_mut_ptr(),
                needed,
                &mut needed,
                &mut returned,
            );

            if res == 0 || returned == 0 {
                return "Microsoft Print to PDF".to_string();
            }

            #[repr(C)]
            struct PRINTER_INFO_4W {
                pPrinterName: *const u16,
                pServerName: *const u16,
                Attributes: u32,
            }

            let slice = std::slice::from_raw_parts(
                buffer.as_ptr() as *const PRINTER_INFO_4W,
                returned as usize,
            );

            let mut nomes: Vec<String> = Vec::new();
            for info in slice {
                if !info.pPrinterName.is_null() {
                    let mut len = 0;
                    while *info.pPrinterName.add(len) != 0 {
                        len += 1;
                    }
                    let u16_slice = std::slice::from_raw_parts(info.pPrinterName, len);
                    if let Ok(nome) = String::from_utf16(u16_slice) {
                        nomes.push(nome);
                    }
                }
            }

            if nomes.is_empty() {
                "Microsoft Print to PDF".to_string()
            } else {
                nomes.join(",")
            }
        }
    }

    #[cfg(not(windows))]
    {
        // No Linux, consulta o CUPS via lpstat
        if let Ok(output) = std::process::Command::new("lpstat").arg("-e").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let printers: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                if !printers.is_empty() {
                    return printers.join(",");
                }
            }
        }
        "PDF,Padrao".to_string()
    }
}

/// Retorna o nome da impressora padrão do sistema
pub fn impressora_padrao() -> String {
    #[cfg(windows)]
    {
        let mut size: u32 = 0;
        unsafe {
            GetDefaultPrinterW(null_mut(), &mut size);
            if size == 0 {
                return "Microsoft Print to PDF".to_string();
            }

            let mut buffer = vec![0u16; size as usize];
            if GetDefaultPrinterW(buffer.as_mut_ptr(), &mut size) != 0 {
                if let Some(pos) = buffer.iter().position(|&c| c == 0) {
                    buffer.truncate(pos);
                }
                String::from_utf16_lossy(&buffer)
            } else {
                "Microsoft Print to PDF".to_string()
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(output) = std::process::Command::new("lpstat").arg("-d").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Exemplo: "system default destination: HP_LaserJet"
                if let Some(pos) = stdout.find(':') {
                    return stdout[pos + 1..].trim().to_string();
                }
            }
        }
        "PDF".to_string()
    }
}

/// Imprime bytes brutos na impressora de destino ou salva diretamente em PDF caso aplicável
pub fn imprimir_raw(nome_impressora: &str, documento_nome: &str, dados: &[u8]) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let doc_clean = documento_nome.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|', ' '], "_");
    let is_pdf_stream = dados.starts_with(b"%PDF-");
    let is_pdf_printer = nome_impressora.to_lowercase().contains("pdf");

    // 1. Caso o documento já seja PDF ou a impressora seja virtual de PDF: Salva direto no Desktop/Downloads
    if is_pdf_stream || is_pdf_printer {
        #[cfg(windows)]
        let save_path = format!("C:\\Users\\Public\\Desktop\\Sankhya_{}_{}.pdf", doc_clean, now);

        #[cfg(not(windows))]
        let save_path = {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            let downloads = format!("{}/Downloads", home);
            if std::path::Path::new(&downloads).exists() {
                format!("{}/Sankhya_{}_{}.pdf", downloads, doc_clean, now)
            } else {
                format!("{}/Sankhya_{}_{}.pdf", home, doc_clean, now)
            }
        };

        if is_pdf_stream {
            match std::fs::write(&save_path, dados) {
                Ok(_) => {
                    crate::handlers::log_msg(&format!(
                        "[PDF Salvo com Sucesso] Arquivo gerado em: {}",
                        save_path
                    ));
                    return Ok(format!("Arquivo PDF salvo em {}", save_path));
                }
                Err(e) => {
                    crate::handlers::log_msg(&format!(
                        "[Aviso] Falha ao gravar PDF em {}: {:?}",
                        save_path, e
                    ));
                }
            }
        }
    }

    // 2. Envio físico no Windows via Spooler (winspool.drv)
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let printer_wide: Vec<u16> = OsStr::new(nome_impressora)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let doc_name_wide: Vec<u16> = OsStr::new(documento_nome)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let raw_type_wide: Vec<u16> = OsStr::new("RAW")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Para impressoras virtuais PDF no Windows Service, define o arquivo de saída sem prompt GUI
        let output_file_wide: Option<Vec<u16>> = if is_pdf_printer {
            let target = format!("C:\\Users\\Public\\Desktop\\Sankhya_{}_{}.pdf", doc_clean, now);
            Some(OsStr::new(&target).encode_wide().chain(std::iter::once(0)).collect())
        } else {
            None
        };

        let mut h_printer: *mut c_void = null_mut();

        unsafe {
            if OpenPrinterW(printer_wide.as_ptr(), &mut h_printer, null_mut()) == 0 {
                return Err(format!("Não foi possível abrir a impressora '{}'", nome_impressora));
            }

            let doc_info = DOC_INFO_1W {
                pDocName: doc_name_wide.as_ptr(),
                pOutputFile: output_file_wide.as_ref().map(|v| v.as_ptr()).unwrap_or(null_mut()),
                pDatatype: raw_type_wide.as_ptr(),
            };

            let job_id = StartDocPrinterW(h_printer, 1, &doc_info);
            if job_id == 0 {
                ClosePrinter(h_printer);
                return Err("Falha ao iniciar documento de impressão no Spooler do Windows".into());
            }

            StartPagePrinter(h_printer);

            let mut written: u32 = 0;
            let ok = WritePrinter(
                h_printer,
                dados.as_ptr() as *const c_void,
                dados.len() as u32,
                &mut written,
            );

            EndPagePrinter(h_printer);
            EndDocPrinter(h_printer);
            ClosePrinter(h_printer);

            if ok == 0 || written != dados.len() as u32 {
                return Err("Falha ao gravar os bytes no Spooler da impressora".into());
            }

            Ok("Impressão enviada ao Spooler com sucesso".into())
        }
    }

    // 3. Envio no Linux via CUPS (`lp`)
    #[cfg(not(windows))]
    {
        use std::io::Write;
        let temp_path = format!("/tmp/sankhya_print_{}.bin", now);
        if let Ok(mut f) = std::fs::File::create(&temp_path) {
            let _ = f.write_all(dados);
            let _ = std::process::Command::new("lp")
                .arg("-d")
                .arg(nome_impressora)
                .arg(&temp_path)
                .status();
            let _ = std::fs::remove_file(temp_path);
            return Ok("Trabalho enviado ao CUPS (Linux)".into());
        }
        Err("Falha ao despachar para o CUPS no Linux".into())
    }
}
