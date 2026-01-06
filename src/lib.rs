use std::io::{self, BufRead};
use std::fs::File;

// 共通の計算ロジック
use fend_core;

// -----------------------------------------------------------------------------
// Core Logic: Pure Rust
// -----------------------------------------------------------------------------

/// 文字列の統計情報を計算する (テスト & リアルタイム入力用)
fn core_text_stats(text: &str) -> (usize, usize, usize) {
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    let lines = text.lines().count();
    (chars, words, lines)
}

/// ファイルパスを受け取り、行数・単語数・文字数をストリーミングでカウントする。
/// メモリ消費量はバッファサイズ（数KB）に限定される。
fn core_stream_stats(path: &str) -> Result<(usize, usize, usize), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    let mut chars = 0;
    let mut words = 0;
    let mut lines = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        lines += 1;
        // 改行文字分(+1)を考慮するかは仕様によるが、ここでは簡易的に char数 + 1 とする
        chars += line.chars().count() + 1; 
        words += line.split_whitespace().count();
    }

    Ok((chars, words, lines))
}

/// 指定された行範囲（start_line から num_lines 分）だけを読み込む。
fn core_read_lines(path: &str, start_line: usize, num_lines: usize) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    let mut result = String::new();
    let mut current_line = 0;

    for line_res in reader.lines() {
        if current_line >= start_line + num_lines {
            break;
        }

        if current_line >= start_line {
            let line = line_res.map_err(|e| e.to_string())?;
            result.push_str(&line);
            result.push('\n');
        } else {
            if let Err(e) = line_res { return Err(e.to_string()); }
        }
        current_line += 1;
    }

    Ok(result)
}

// -----------------------------------------------------------------------------
// Module: Python Interface (PyO3)
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
fn calculate(expression: String) -> PyResult<String> {
    let mut context = fend_core::Context::new();
    match fend_core::evaluate(&expression, &mut context) {
        Ok(res) => Ok(res.get_main_result().to_string()),
        Err(e) => Ok(format!("Error: {}", e)),
    }
}

// Pythonから文字列を直接渡して統計を取る関数
#[cfg(feature = "python")]
#[pyfunction]
fn get_text_stats(text: String) -> PyResult<(usize, usize, usize)> {
    Ok(core_text_stats(&text))
}

// ファイルパスを渡して統計を取る関数 (巨大ファイル用)
#[cfg(feature = "python")]
#[pyfunction]
fn get_file_stats(path: String) -> PyResult<(usize, usize, usize)> {
    core_stream_stats(&path).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e))
}

#[cfg(feature = "python")]
#[pyfunction]
fn read_file_chunk(path: String, start_line: usize, num_lines: usize) -> PyResult<String> {
    core_read_lines(&path, start_line, num_lines).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e))
}

#[cfg(feature = "python")]
#[pymodule]
fn rusty_pad(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate, m)?)?;
    m.add_function(wrap_pyfunction!(get_text_stats, m)?)?; // 復活
    m.add_function(wrap_pyfunction!(get_file_stats, m)?)?;
    m.add_function(wrap_pyfunction!(read_file_chunk, m)?)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Module: WebAssembly Interface (wasm-bindgen)
// -----------------------------------------------------------------------------
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct RustySession {
    ctx: fend_core::Context,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl RustySession {
    pub fn new() -> Self {
        RustySession { ctx: fend_core::Context::new() }
    }
    pub fn evaluate(&mut self, expression: &str) -> String {
        if expression.trim().is_empty() { return String::new(); }
        match fend_core::evaluate(expression, &mut self.ctx) {
            Ok(res) => res.get_main_result().to_string(),
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct TextStats {
    pub chars: usize,
    pub words: usize,
    pub lines: usize,
}

// WASM用にも core_text_stats を利用する
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_text_stats(text: &str) -> TextStats {
    let (c, w, l) = core_text_stats(text);
    TextStats { chars: c, words: w, lines: l }
}