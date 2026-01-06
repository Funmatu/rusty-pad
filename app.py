import os
import threading
import webview
from bottle import Bottle, request, static_file
import rusty_pad

app = Bottle()
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
WWW_DIR = os.path.join(BASE_DIR, "www")


# ----------------------------------------------------------------
# API: Health Check (Mode Detection)
# ----------------------------------------------------------------
@app.route("/api/status")
def status():
    """フロントエンドがNativeモードか判定するためのAPI"""
    return {"mode": "native", "backend": "rust-python"}


# ----------------------------------------------------------------
# API: Calculation (Optional fallback, but WASM is preferred)
# ----------------------------------------------------------------
@app.route("/api/calculate", method="POST")
def handle_calculate():
    data = request.json
    expr = data.get("expression", "")
    try:
        result = rusty_pad.calculate(expr)
        return {"result": result, "status": "ok"}
    except Exception as e:
        return {"result": str(e), "status": "error"}


# ----------------------------------------------------------------
# API: File I/O (Streaming for Native)
# ----------------------------------------------------------------
@app.route("/api/open_file", method="POST")
def open_file():
    file_types = ("Text Files (*.txt;*.md;*.json;*.rs;*.py)", "All files (*.*)")
    # create_file_dialog はメインスレッドで呼ばれる必要があるため、
    # 実際の運用ではウィンドウインスタンスの管理が必要だが、ここでは簡易実装
    try:
        file_path = webview.windows[0].create_file_dialog(
            webview.OPEN_DIALOG, file_types=file_types
        )
    except:
        # ウィンドウが特定できない場合などのフォールバック
        return {"error": "Dialog unavailable"}

    if not file_path:
        return {"cancelled": True}

    path = file_path[0]
    try:
        stats = rusty_pad.get_file_stats(path)
        preview_content = rusty_pad.read_file_chunk(path, 0, 100)
        return {
            "filepath": path,
            "filename": os.path.basename(path),
            "stats": stats,
            "content": preview_content,
            "total_lines": stats[2],
        }
    except Exception as e:
        return {"error": str(e)}


@app.route("/api/read_chunk", method="POST")
def read_chunk():
    data = request.json
    path = data.get("filepath")
    start = data.get("start_line", 0)
    count = data.get("num_lines", 100)
    try:
        content = rusty_pad.read_file_chunk(path, start, count)
        return {"content": content}
    except Exception as e:
        return {"error": str(e)}


# ----------------------------------------------------------------
# Static Files
# ----------------------------------------------------------------
@app.route("/")
def index():
    return static_file("index.html", root=WWW_DIR)


@app.route("/<filepath:path>")
def server_static(filepath):
    return static_file(filepath, root=WWW_DIR)


def start_server():
    app.run(host="127.0.0.1", port=23456, quiet=True)


if __name__ == "__main__":
    t = threading.Thread(target=start_server)
    t.daemon = True
    t.start()

    webview.create_window(
        "Rusty Pad (Hybrid Edition)", "http://127.0.0.1:23456", width=1000, height=800
    )
    webview.start()
