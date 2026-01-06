import init, { wasm_calculate, wasm_text_stats } from './pkg/rusty_pad.js';

const notepad = document.getElementById('notepad');
const statusBar = document.getElementById('status-bar');
const calcDisplay = document.getElementById('calc-display');

// Initialize WASM
async function run() {
    await init();
    statusBar.innerText = "Rust Core Loaded. Ready.";
    
    // Notepad Event Listeners
    notepad.addEventListener('input', updateStats);
    
    // Calculator Event Listener (Enter key)
    document.addEventListener('keydown', (e) => {
        if (document.activeElement !== notepad) {
            if (e.key === 'Enter') window.compute();
            if (e.key === 'Escape') window.clearCalc();
        }
    });
}

// Update Notepad Stats using Rust
function updateStats() {
    const text = notepad.value;
    // Rust側で高速に集計
    const stats = wasm_text_stats(text);
    statusBar.innerText = `Chars: ${stats.chars} | Words: ${stats.words} | Lines: ${stats.lines} | Mem: Efficient`;
}

// Calculator Logic calling Rust
window.compute = () => {
    const expr = calcDisplay.value;
    try {
        const result = wasm_calculate(expr);
        calcDisplay.value = result;
    } catch (e) {
        calcDisplay.value = "Error";
    }
};

// File Save Logic (Native JS for IO)
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
    updateStats();
}

run();