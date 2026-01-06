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
# API: Calculation
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
# API: File I/O
# ----------------------------------------------------------------
@app.route("/api/open_file", method="POST")
def open_file():
    file_types = ("Text Files (*.txt;*.md;*.json;*.rs;*.py)", "All files (*.*)")
    try:
        file_path = webview.windows[0].create_file_dialog(
            webview.OPEN_DIALOG, file_types=file_types
        )
    except:
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


# ----------------------------------------------------------------
# API: Advanced Native Features (Search & Save)
# ----------------------------------------------------------------
@app.route("/api/search_next", method="POST")
def handle_search_next():
    data = request.json
    path = data.get("filepath")
    query = data.get("query")
    start_line = data.get("start_line", 0)

    if not query:
        return {"error": "Query is empty"}
    try:
        found_line = rusty_pad.search_next(path, query, start_line)
        if found_line is not None:
            # 見つかったら、その行が見えるようにチャンクを読み込んで返す
            display_start = max(0, found_line - 50)
            content = rusty_pad.read_file_chunk(path, display_start, 100)
            return {
                "found": True,
                "line": found_line,
                "display_start": display_start,
                "content": content,
            }
        else:
            return {"found": False}
    except Exception as e:
        return {"error": str(e)}


@app.route("/api/search_prev", method="POST")
def handle_search_prev():
    data = request.json
    path = data.get("filepath")
    query = data.get("query")
    start_line = data.get("start_line", 0)

    if not query:
        return {"error": "Query is empty"}
    try:
        found_line = rusty_pad.search_prev(path, query, start_line)
        if found_line is not None:
            display_start = max(0, found_line - 50)
            content = rusty_pad.read_file_chunk(path, display_start, 100)
            return {
                "found": True,
                "line": found_line,
                "display_start": display_start,
                "content": content,
            }
        else:
            return {"found": False}
    except Exception as e:
        return {"error": str(e)}


# ✅ 修正: 範囲指定保存 (開始行・終了行を指定)
@app.route("/api/save_range", method="POST")
def handle_save_range():
    data = request.json
    src_path = data.get("filepath")

    # 0-indexed で受け取る想定だが、UI入力の解釈はJS側で行う
    start_line = data.get("start_line")
    end_line = data.get("end_line")

    file_types = ("Text Files (*.txt)", "All files (*.*)")
    dest_path_obj = webview.windows[0].create_file_dialog(
        webview.SAVE_DIALOG, file_types=file_types, save_filename="extracted.txt"
    )

    if not dest_path_obj:
        return {"cancelled": True}

    if isinstance(dest_path_obj, (tuple, list)):
        dest_path = dest_path_obj[0]
    else:
        dest_path = str(dest_path_obj)

    try:
        # start_line, end_line をそのままRustへ
        count = rusty_pad.save_range(src_path, dest_path, start_line, end_line)
        return {"saved": True, "lines_written": count, "path": dest_path}
    except Exception as e:
        return {"error": str(e)}


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
