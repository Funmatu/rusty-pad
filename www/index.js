import init, { RustySession, wasm_text_stats } from './pkg/rusty_pad.js';

const notepad = document.getElementById('notepad');
const statusBar = document.getElementById('status-bar');
const calcDisplay = document.getElementById('calc-display');
const consoleInput = document.getElementById('console-input');
const consoleHistory = document.getElementById('console-history');
const webFileInput = document.getElementById('web-file-input');
const btnLoadMore = document.getElementById('btn-load-more');

let session = null;
let isNativeMode = false;

// State for Native Pagination
let nativeFileState = {
    path: null,
    totalLines: 0,
    currentLine: 0,
    chunkSize: 100
};

// Initialize
async function run() {
    try {
        await init();
        session = RustySession.new();
        addLog("WASM Core Loaded.", "system");

        // Native環境かチェック
        try {
            const res = await fetch('/api/status');
            if (res.ok) {
                const data = await res.json();
                if (data.mode === 'native') {
                    isNativeMode = true;
                    addLog("Mode: Native Python Backend (Streaming IO Enabled)", "system");
                    statusBar.innerText = "Ready (Native Mode)";
                }
            }
        } catch (e) {
            // Fetch失敗 = Web Mode
            addLog("Mode: Web Browser (Local Storage Only)", "system");
            statusBar.innerText = "Ready (Web Mode)";
        }

        // Notepad Stats (Web/Native共通でWASMを使用)
        notepad.addEventListener('input', () => {
            const text = notepad.value;
            const stats = wasm_text_stats(text);
            updateStatusBar(stats);
        });

    } catch (e) {
        console.error(e);
        statusBar.innerText = "Error loading WASM.";
    }
}

function updateStatusBar(stats) {
    let modeText = isNativeMode ? "Native" : "Web";
    statusBar.innerText = `Chars: ${stats.chars} | Words: ${stats.words} | Lines: ${stats.lines} | Mode: ${modeText}`;
}

// ----------------------
// Unified Calculator & Console (All WASM)
// ----------------------
window.compute = () => {
    if (!session) return;
    const expr = calcDisplay.value;
    const result = session.evaluate(expr);
    calcDisplay.value = result.startsWith("Error") ? "Error" : result;
    if (!result.startsWith("Error")) {
        addLog(`[GUI] ${expr} = ${result}`, 'system');
    }
};

consoleInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
        const cmd = consoleInput.value;
        if (!cmd) return;
        addLog(`> ${cmd}`, 'user');
        const result = session.evaluate(cmd);
        addLog(result, result.startsWith("Error") ? 'error' : 'result');
        consoleInput.value = "";
    }
});

// ----------------------
// Smart File I/O
// ----------------------
window.openSmartFile = async () => {
    if (isNativeMode) {
        // Native: Python APIを呼ぶ
        await openNativeFile();
    } else {
        // Web: ファイル選択ダイアログを起動
        webFileInput.click();
    }
};

// Strategy A: Native (Streaming)
async function openNativeFile() {
    statusBar.innerText = "Opening via Rust Backend...";
    try {
        const res = await (await fetch('/api/open_file', { method: 'POST' })).json();
        
        if (res.cancelled) {
            statusBar.innerText = "Cancelled.";
            return;
        }
        if (res.error) throw new Error(res.error);

        nativeFileState.path = res.filepath;
        nativeFileState.totalLines = res.total_lines;
        nativeFileState.currentLine = 0;
        
        notepad.value = res.content;
        btnLoadMore.style.display = 'inline-block';
        
        const s = res.stats;
        statusBar.innerText = `File: ${res.filename} | Total Lines: ${s[2]} | Chars: ${s[0]}`;
        addLog(`Opened ${res.filename} (Streaming Mode)`, 'system');
        
    } catch (e) {
        alert("Native Open Error: " + e.message);
    }
}

window.loadNextChunk = async () => {
    if (!isNativeMode || !nativeFileState.path) return;
    
    const nextStart = nativeFileState.currentLine + nativeFileState.chunkSize;
    if (nextStart >= nativeFileState.totalLines) return;
    
    statusBar.innerText = "Loading chunk...";
    const res = await (await fetch('/api/read_chunk', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            filepath: nativeFileState.path,
            start_line: nextStart,
            num_lines: nativeFileState.chunkSize
        })
    })).json();
    
    if (res.content) {
        notepad.value += "\n" + res.content;
        nativeFileState.currentLine = nextStart;
        statusBar.innerText = `Loaded lines up to ${nextStart + nativeFileState.chunkSize}`;
    }
};

// Strategy B: Web (FileReader)
webFileInput.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (evt) => {
        const text = evt.target.result;
        notepad.value = text;
        
        // 統計はWASMで即時計算
        const stats = wasm_text_stats(text);
        updateStatusBar(stats);
        
        addLog(`Loaded ${file.name} (Web Mode)`, 'system');
        btnLoadMore.style.display = 'none'; // Webモードではチャンク読み込み不可
    };
    reader.readAsText(file);
});

// Utils
window.saveFile = () => {
    const text = notepad.value;
    const blob = new Blob([text], { type: 'text/plain' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'note.txt';
    a.click();
    URL.revokeObjectURL(a.href);
};

window.clearPad = () => {
    notepad.value = "";
    statusBar.innerText = "Ready";
    btnLoadMore.style.display = 'none';
};

function addLog(text, type) {
    const div = document.createElement('div');
    div.className = `log-entry ${type}`;
    div.innerText = text;
    consoleHistory.appendChild(div);
    consoleHistory.scrollTop = consoleHistory.scrollHeight;
}

run();