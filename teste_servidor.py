import http.server
import ssl
import json
import os
import subprocess
import threading

PORT_HTTP = 9096
PORT_HTTPS = 9196
CERT_FILE = os.path.join(os.path.dirname(__file__), "certs", "cert.pem")
KEY_FILE = os.path.join(os.path.dirname(__file__), "certs", "key.pem")

def get_windows_printers():
    try:
        cmd = 'powershell -NoProfile -Command "Get-Printer | Select-Object -ExpandProperty Name"'
        out = subprocess.check_output(cmd, shell=True).decode('utf-8', errors='ignore')
        printers = [p.strip() for p in out.splitlines() if p.strip()]
        return ",".join(printers)
    except Exception:
        return "Microsoft Print to PDF"

class SankhyaHandler(http.server.BaseHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With, Origin, Accept")
        self.send_header("Access-Control-Allow-Credentials", "true")
        self.send_header("Access-Control-Allow-Private-Network", "true")
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

    def handle_request(self):
        path = self.path
        method = self.command
        print(f"[{method}] Recebido: {path}")

        if "/.info" in path:
            self.send_response(200)
            self.send_header("Content-Type", "text/plain;charset=UTF-8")
            self.end_headers()
            if "webcn=true" in path:
                self.wfile.write(b"false")
            else:
                self.wfile.write(b"dados do sistema:  versao: 1.0 uptime: 0")

        elif "/.pList" in path:
            printers = get_windows_printers()
            self.send_response(200)
            self.send_header("Content-Type", "text/plain;charset=UTF-8")
            self.end_headers()
            self.wfile.write(printers.encode("utf-8"))

        elif "/.pDefaultPrinter" in path:
            self.send_response(200)
            self.send_header("Content-Type", "text/plain;charset=UTF-8")
            self.end_headers()
            self.wfile.write(b"Microsoft Print to PDF")

        elif "/.systemInfo" in path:
            sys_info = {
                "infoPC": os.environ.get("COMPUTERNAME", "LOCALHOST"),
                "clientIp": "127.0.0.1",
                "clientName": os.environ.get("COMPUTERNAME", "LOCALHOST"),
                "clientMAC": "00:11:22:33:44:55",
                "userName": os.environ.get("USERNAME", "usuario"),
                "domainName": os.environ.get("USERDOMAIN", "WORKGROUP")
            }
            body = json.dumps(sys_info).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json;charset=UTF-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        elif "/.localPrinting" in path or "/.impressaoDireta" in path:
            resp = "Servidor realizando impressão.".encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/plain;charset=UTF-8")
            self.send_header("Content-Length", str(len(resp)))
            self.end_headers()
            self.wfile.write(resp)

        else:
            self.send_response(200)
            self.send_header("Content-Type", "text/plain;charset=UTF-8")
            self.end_headers()
            self.wfile.write(b"OK")

    def do_GET(self):
        self.handle_request()

    def do_POST(self):
        # Lê o body se houver
        content_len = int(self.headers.get('Content-Length', 0))
        if content_len > 0:
            _ = self.rfile.read(content_len)
        self.handle_request()

def run_http():
    server = http.server.ThreadingHTTPServer(("0.0.0.0", PORT_HTTP), SankhyaHandler)
    print(f"[*] Servidor HTTP ativo em http://0.0.0.0:{PORT_HTTP}")
    server.serve_forever()

def run_https():
    if not os.path.exists(CERT_FILE) or not os.path.exists(KEY_FILE):
        return
    server = http.server.ThreadingHTTPServer(("0.0.0.0", PORT_HTTPS), SankhyaHandler)
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(certfile=CERT_FILE, keyfile=KEY_FILE)
    server.socket = ctx.wrap_socket(server.socket, server_side=True)
    print(f"[*] Servidor HTTPS ativo em https://0.0.0.0:{PORT_HTTPS}")
    server.serve_forever()

if __name__ == "__main__":
    t_http = threading.Thread(target=run_http, daemon=True)
    t_https = threading.Thread(target=run_https, daemon=True)
    t_http.start()
    t_https.start()
    print("==================================================")
    print(f" Portas ativas: HTTP {PORT_HTTP} e HTTPS {PORT_HTTPS}")
    print(" Aguardando chamadas do Sankhya...")
    print("==================================================")
    try:
        t_http.join()
    except KeyboardInterrupt:
        print("\nFinalizado.")
