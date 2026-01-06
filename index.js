import init, { RustySession, wasm_text_stats } from './pkg/rusty_pad.js';

const notepad = document.getElementById('notepad');
const lineNumbers = document.getElementById('line-numbers');
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
                    document.querySelectorAll('.native-only').forEach(el => el.style.display = 'inline');
                }
            }
        } catch (e) {
            addLog("Mode: Web Browser (Local Storage Only)", "system");
            statusBar.innerText = "Ready (Web Mode)";
        }

        // Notepad Stats & Line Numbers
        notepad.addEventListener('input', () => {
            const text = notepad.value;
            const stats = wasm_text_stats(text);
            updateStatusBar(stats);
            updateLineNumbers();
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

function updateLineNumbers() {
    const text = notepad.value;
    const lines = text.split('\n').length;
    const offset = isNativeMode ? nativeFileState.currentLine : 0;
    
    let html = '';
    for (let i = 0; i < lines; i++) {
        html += `<div>${offset + i + 1}</div>`;
    }
    lineNumbers.innerHTML = html;
}

window.syncScroll = () => {
    lineNumbers.scrollTop = notepad.scrollTop;
};

// ----------------------
// Unified Calculator
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
        await openNativeFile();
    } else {
        webFileInput.click();
    }
};

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
        updateLineNumbers();
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
        updateLineNumbers();
        statusBar.innerText = `Loaded lines up to ${nextStart + nativeFileState.chunkSize}`;
    }
};

webFileInput.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (evt) => {
        const text = evt.target.result;
        notepad.value = text;
        const stats = wasm_text_stats(text);
        updateStatusBar(stats);
        updateLineNumbers();
        addLog(`Loaded ${file.name} (Web Mode)`, 'system');
        btnLoadMore.style.display = 'none';
    };
    reader.readAsText(file);
});

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
    updateLineNumbers();
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

// ----------------------
// Native Search Logic
// ----------------------
window.findNext = async () => {
    if (!nativeFileState.path) return;
    const query = document.getElementById('search-query').value;
    if (!query) return;

    statusBar.innerText = "Searching Next...";
    const startFrom = nativeFileState.currentLine + 1;

    const res = await (await fetch('/api/search_next', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            filepath: nativeFileState.path,
            query: query,
            start_line: startFrom
        })
    })).json();

    handleSearchResult(res, query);
};

window.findPrev = async () => {
    if (!nativeFileState.path) return;
    const query = document.getElementById('search-query').value;
    if (!query) return;

    statusBar.innerText = "Searching Prev...";
    const startFrom = nativeFileState.currentLine;

    const res = await (await fetch('/api/search_prev', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            filepath: nativeFileState.path,
            query: query,
            start_line: startFrom
        })
    })).json();

    handleSearchResult(res, query);
};

// ✅ 修正: 検索ヒット時にその位置までスクロールして選択する
function handleSearchResult(res, query) {
    if (res.found) {
        nativeFileState.currentLine = res.display_start;
        notepad.value = res.content;
        updateLineNumbers();
        statusBar.innerText = `Found at line ${res.line + 1}. Displaying chunk starting at ${res.display_start + 1}`;

        // カーソル移動とスクロール処理
        try {
            // 1. ヒットした行が、現在表示中のチャンク内の何行目にあるか計算 (0-indexed)
            const relativeLineIndex = res.line - res.display_start;
            
            // 2. その行の開始位置(文字数)を計算
            const lines = res.content.split('\n');
            let charOffset = 0;
            for (let i = 0; i < relativeLineIndex; i++) {
                // 行の長さ + 改行コード分(1)
                charOffset += lines[i].length + 1;
            }
            
            // 3. その行の中でクエリ文字列がどこにあるか探す
            const targetLineText = lines[relativeLineIndex];
            const queryIndexInLine = targetLineText.indexOf(query);
            
            if (queryIndexInLine !== -1) {
                const finalPos = charOffset + queryIndexInLine;
                
                // 4. フォーカスして選択範囲を設定 (これで自動スクロールされる)
                notepad.focus();
                notepad.setSelectionRange(finalPos, finalPos + query.length);
            }
        } catch (e) {
            console.error("Auto-scroll failed:", e);
        }

    } else {
        statusBar.innerText = "Not found.";
    }
}

// ----------------------
// Native Extraction Logic
// ----------------------
window.saveRange = async () => {
    if (!nativeFileState.path) return;

    // ✅ 修正: 開始行-終了行を入力させる
    const rangeInput = prompt("Enter range to extract (Start-End):", 
        `${nativeFileState.currentLine + 1}-${nativeFileState.currentLine + 100}`);
    
    if (!rangeInput) return;

    // 入力パース ("100-200" -> start:100, end:200)
    const parts = rangeInput.split('-');
    if (parts.length !== 2) {
        alert("Invalid format. Use: Start-End (e.g., 100-200)");
        return;
    }

    const startInput = parseInt(parts[0].trim());
    const endInput = parseInt(parts[1].trim());

    if (isNaN(startInput) || isNaN(endInput)) {
        alert("Invalid numbers.");
        return;
    }

    statusBar.innerText = "Extracting...";
    
    const res = await (await fetch('/api/save_range', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            filepath: nativeFileState.path,
            // 1-indexed (UI) -> 0-indexed (Internal)
            start_line: startInput - 1,
            // ユーザーは「200行目まで」と指定するが、プログラムはexclusiveな場合が多い。
            // しかしRust側の実装は `index >= end_line` でbreakする(exclusive)。
            // 「1行目から1行目まで」→ 1-1 → start=0, end=1. Rust: index 0 ok, index 1 break. 正しい。
            // なので、終了行はそのまま渡せば「指定した行番号まで（その行を含む）」動作になる。
            end_line: endInput
        })
    })).json();

    if (res.saved) {
        alert(`Successfully extracted ${res.lines_written} lines to:\n${res.path}`);
        statusBar.innerText = "Extraction complete.";
    } else if (res.cancelled) {
        statusBar.innerText = "Save cancelled.";
    } else {
        alert("Error: " + res.error);
    }
};

run();