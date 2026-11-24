// Módulo de comunicação nativa com o Spooler de Impressão do Windows
use std::ffi::c_void;
use std::ptr::null_mut;

#[repr(C)]
struct DOC_INFO_1W {
    pDocName: *const u16,
    pOutputFile: *const u16,
    pDatatype: *const u16,
}

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

const PRINTER_ENUM_LOCAL: u32 = 0x00000002;
const PRINTER_ENUM_CONNECTIONS: u32 = 0x00000004;

/// Retorna a lista de todas as impressoras instaladas no Windows, separadas por vírgula
pub fn listar_impressoras() -> String {
    let mut needed: u32 = 0;
    let mut returned: u32 = 0;

    unsafe {
        // Primeira chamada para descobrir o tamanho do buffer necessário
        let _ = EnumPrintersW(
            PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
            null_mut(),
            4, // PRINTER_INFO_4 (retorna nomes das impressoras locais e de rede)
            null_mut(),
            0,
            &mut needed,
            &mut returned,
        );

        if needed == 0 {
            return String::new();
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
            return String::new();
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

        nomes.join(",")
    }
}

/// Retorna o nome da impressora padrão do Windows
pub fn impressora_padrao() -> String {
    let mut size: u32 = 0;
    unsafe {
        // Pega o tamanho necessário
        GetDefaultPrinterW(null_mut(), &mut size);
        if size == 0 {
            return String::new();
        }

        let mut buffer = vec![0u16; size as usize];
        if GetDefaultPrinterW(buffer.as_mut_ptr(), &mut size) != 0 {
            // Remove o nulo terminador se existir
            if let Some(pos) = buffer.iter().position(|&c| c == 0) {
                buffer.truncate(pos);
            }
            String::from_utf16_lossy(&buffer)
        } else {
            String::new()
        }
    }
}

/// Envia bytes brutos (RAW) diretamente para o driver da impressora (ZPL, ESC/POS, etc.)
pub fn imprimir_raw(nome_impressora: &str, documento_nome: &str, dados: &[u8]) -> Result<(), String> {
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

    let mut h_printer: *mut c_void = null_mut();

    unsafe {
        if OpenPrinterW(printer_wide.as_ptr(), &mut h_printer, null_mut()) == 0 {
            return Err(format!("Não foi possível abrir a impressora '{}'", nome_impressora));
        }

        let doc_info = DOC_INFO_1W {
            pDocName: doc_name_wide.as_ptr(),
            pOutputFile: null_mut(),
            pDatatype: raw_type_wide.as_ptr(),
        };

        let job_id = StartDocPrinterW(h_printer, 1, &doc_info);
        if job_id == 0 {
            ClosePrinter(h_printer);
            return Err("Falha ao iniciar documento de impressão no Spooler".into());
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
            return Err("Falha ao gravar todos os bytes no Spooler".into());
        }

        Ok(())
    }
}
