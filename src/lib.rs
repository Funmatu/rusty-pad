// -----------------------------------------------------------------------------
// Core Logic (Pure Rust)
// -----------------------------------------------------------------------------

/// 数式文字列を受け取り、計算結果を文字列で返す
/// fend-core は単位や通貨も扱えるため、結果は f64 ではなく String となる
fn core_calculate(expression: &str) -> String {
    let mut context = fend_core::Context::new();
    // fend_core::evaluate は Result を返すが、Err の場合もエラーメッセージを含んだ Ok に近い挙動をする場合があるため
    // 単純に main_result を取得する
    match fend_core::evaluate(expression, &mut context) {
        Ok(res) => res.get_main_result().to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

/// テキストの統計情報を計算する (変更なし)
fn core_text_stats(text: &str) -> (usize, usize, usize) {
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    let lines = text.lines().count();
    (chars, words, lines)
}

// -----------------------------------------------------------------------------
// Module: Python Interface (PyO3)
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
// fend-core の仕様に合わせて戻り値を String に変更
fn calculate(expression: String) -> PyResult<String> {
    Ok(core_calculate(&expression))
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

// -----------------------------------------------------------------------------
// Module: WebAssembly Interface (wasm-bindgen)
// -----------------------------------------------------------------------------
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_calculate(expression: &str) -> String {
    core_calculate(expression)
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