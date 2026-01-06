use fend_core;

// -----------------------------------------------------------------------------
// Core Logic (Pure Rust)
// -----------------------------------------------------------------------------

// 以前のステートレスな関数は削除し、構造体に移行しても良いですが、
// Python用に簡易関数は残しておきます。
fn core_calculate_oneshot(expression: &str) -> String {
    let mut context = fend_core::Context::new();
    match fend_core::evaluate(expression, &mut context) {
        Ok(res) => res.get_main_result().to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

fn core_text_stats(text: &str) -> (usize, usize, usize) {
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    let lines = text.lines().count();
    (chars, words, lines)
}

// -----------------------------------------------------------------------------
// Module: WebAssembly Interface (wasm-bindgen)
// -----------------------------------------------------------------------------
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// 【変更点】WASM側で状態（変数定義など）を保持するための構造体を作成
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct RustySession {
    ctx: fend_core::Context,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl RustySession {
    // コンストラクタ
    pub fn new() -> Self {
        RustySession {
            ctx: fend_core::Context::new(),
        }
    }

    // 状態を維持しながら計算する
    pub fn evaluate(&mut self, expression: &str) -> String {
        // expression が空なら何もしない
        if expression.trim().is_empty() {
            return String::new();
        }

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

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_text_stats(text: &str) -> TextStats {
    let (c, w, l) = core_text_stats(text);
    TextStats { chars: c, words: w, lines: l }
}

// -----------------------------------------------------------------------------
// Module: Python Interface (PyO3)
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
fn calculate(expression: String) -> PyResult<String> {
    Ok(core_calculate_oneshot(&expression))
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_text_stats(text: String) -> PyResult<(usize, usize, usize)> {
    Ok(core_text_stats(&text))
}

#[cfg(feature = "python")]
#[pymodule]
fn rusty_pad(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate, m)?)?;
    m.add_function(wrap_pyfunction!(get_text_stats, m)?)?;
    Ok(())
}