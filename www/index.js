// Import RustySession instead of simple functions
import init, { RustySession, wasm_text_stats } from './pkg/rusty_pad.js';

const notepad = document.getElementById('notepad');
const statusBar = document.getElementById('status-bar');
const calcDisplay = document.getElementById('calc-display');
const consoleInput = document.getElementById('console-input');
const consoleHistory = document.getElementById('console-history');

// Global Session Instance
let session = null;

async function run() {
    await init();
    
    // Create a persistent session
    session = RustySession.new();
    
    statusBar.innerText = "Rust Core Loaded. Ready.";
    
    // Notepad Listeners
    notepad.addEventListener('input', updateStats);
    
    // --- Console Event Listeners ---
    consoleInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            const cmd = consoleInput.value;
            if (!cmd) return;
            
            // Log user input
            addLog(`> ${cmd}`, 'user');
            
            // Execute in Rust
            const result = session.evaluate(cmd);
            
            // Log result
            addLog(result, result.startsWith("Error") ? 'error' : 'result');
            
            consoleInput.value = "";
            
            // Auto-scroll
            consoleHistory.scrollTop = consoleHistory.scrollHeight;
        }
    });

    // --- Calculator Buttons Event Listeners ---
    // (Existing global helper overrides)
    
    window.compute = () => {
        if (!session) return;
        const expr = calcDisplay.value;
        try {
            // Use the SAME session as the console
            const result = session.evaluate(expr);
            calcDisplay.value = result;
            
            // Optionally log calculator actions to console history too
            addLog(`[GUI] ${expr}`, 'system');
            addLog(`= ${result}`, 'system');
            consoleHistory.scrollTop = consoleHistory.scrollHeight;
            
        } catch (e) {
            calcDisplay.value = "Error";
        }
    };
}

function updateStats() {
    const text = notepad.value;
    const stats = wasm_text_stats(text);
    statusBar.innerText = `Chars: ${stats.chars} | Words: ${stats.words} | Lines: ${stats.lines} | Mem: Efficient`;
}

// Console Helper
function addLog(text, type) {
    const div = document.createElement('div');
    div.className = `log-entry ${type}`;
    div.innerText = text;
    consoleHistory.appendChild(div);
}

// Global helpers for GUI buttons
window.append = (val) => {
    const disp = document.getElementById('calc-display');
    if (disp.value === 'Ready' || disp.value.startsWith('Error')) disp.value = '';
    disp.value += val;
};
window.clearCalc = () => document.getElementById('calc-display').value = '';
window.backspace = () => {
    const disp = document.getElementById('calc-display');
    disp.value = disp.value.slice(0, -1);
};

// File Save Logic
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