use std::io::{self, BufRead, Write};
use std::fs::File;
use fend_core;

// -----------------------------------------------------------------------------
// Core Logic: Pure Rust
// -----------------------------------------------------------------------------

fn core_text_stats(text: &str) -> (usize, usize, usize) {
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    let lines = text.lines().count();
    (chars, words, lines)
}

fn core_stream_stats(path: &str) -> Result<(usize, usize, usize), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    let mut chars = 0;
    let mut words = 0;
    let mut lines = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        lines += 1;
        chars += line.chars().count() + 1; 
        words += line.split_whitespace().count();
    }

    Ok((chars, words, lines))
}

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
// Search Features
// -----------------------------------------------------------------------------

fn core_search_next(path: &str, query: &str, start_line: usize) -> Result<Option<usize>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        if index < start_line { continue; }
        
        let line = line.map_err(|e| e.to_string())?;
        if line.contains(query) {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn core_search_prev(path: &str, query: &str, start_line: usize) -> Result<Option<usize>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    let mut last_match: Option<usize> = None;

    for (index, line) in reader.lines().enumerate() {
        if index >= start_line {
            break;
        }

        let line = line.map_err(|e| e.to_string())?;
        if line.contains(query) {
            last_match = Some(index);
        }
    }

    Ok(last_match)
}

// ✅ 修正: 開始行(start_line)と終了行(end_line)を直接受け取る仕様に変更
// end_lineは「そこまで含める」のではなく「ここまで来たら終了(Exclusive)」とするのが一般的だが、
// 呼び出し元で調整済みとする。ここでは `index >= end_line` でbreakする。
fn core_save_range(src_path: &str, dest_path: &str, start_line: usize, end_line: usize) -> Result<usize, String> {
    let src_file = File::open(src_path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(src_file);
    
    let dest_file = File::create(dest_path).map_err(|e| e.to_string())?;
    let mut writer = io::BufWriter::new(dest_file);
    
    let mut lines_written = 0;

    for (index, line) in reader.lines().enumerate() {
        // end_lineに達したら終了
        if index >= end_line {
            break;
        }
        
        if index >= start_line {
            let line = line.map_err(|e| e.to_string())?;
            writeln!(writer, "{}", line).map_err(|e| e.to_string())?;
            lines_written += 1;
        }
    }
    
    writer.flush().map_err(|e| e.to_string())?;
    Ok(lines_written)
}

// -----------------------------------------------------------------------------
// Python Interface (PyO3)
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

#[cfg(feature = "python")]
#[pyfunction]
fn get_text_stats(text: String) -> PyResult<(usize, usize, usize)> {
    Ok(core_text_stats(&text))
}

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
#[pyfunction]
fn search_next(path: String, query: String, start_line: usize) -> PyResult<Option<usize>> {
    core_search_next(&path, &query, start_line).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e))
}

#[cfg(feature = "python")]
#[pyfunction]
fn search_prev(path: String, query: String, start_line: usize) -> PyResult<Option<usize>> {
    core_search_prev(&path, &query, start_line).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e))
}

// ✅ 修正: 引数を start_line, end_line に変更
#[cfg(feature = "python")]
#[pyfunction]
fn save_range(src_path: String, dest_path: String, start_line: usize, end_line: usize) -> PyResult<usize> {
    core_save_range(&src_path, &dest_path, start_line, end_line).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e))
}

#[cfg(feature = "python")]
#[pymodule]
fn rusty_pad(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate, m)?)?;
    m.add_function(wrap_pyfunction!(get_text_stats, m)?)?;
    m.add_function(wrap_pyfunction!(get_file_stats, m)?)?;
    m.add_function(wrap_pyfunction!(read_file_chunk, m)?)?;
    m.add_function(wrap_pyfunction!(search_next, m)?)?;
    m.add_function(wrap_pyfunction!(search_prev, m)?)?;
    m.add_function(wrap_pyfunction!(save_range, m)?)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// WebAssembly Interface (wasm-bindgen)
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

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_text_stats(text: &str) -> TextStats {
    let (c, w, l) = core_text_stats(text);
    TextStats { chars: c, words: w, lines: l }
}