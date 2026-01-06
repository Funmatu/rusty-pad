use std::io::{self, BufRead, Write};
use std::fs::File;
use fend_core;
use regex::Regex;
use chrono::{NaiveDate, Datelike}; 

// -----------------------------------------------------------------------------
// Core Logic: Pure Rust (Stats, Search, I/O)
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

fn core_save_range(src_path: &str, dest_path: &str, start_line: usize, end_line: usize) -> Result<usize, String> {
    let src_file = File::open(src_path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(src_file);
    
    let dest_file = File::create(dest_path).map_err(|e| e.to_string())?;
    let mut writer = io::BufWriter::new(dest_file);
    
    let mut lines_written = 0;

    for (index, line) in reader.lines().enumerate() {
        if index >= end_line { break; }
        
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
// Helper: Smart Date Logic (Chrono)
// -----------------------------------------------------------------------------

/// 正確なカレンダー計算を行うヘルパー
/// Chronoを使って「月」や「年」を加算し、存在しない日（2/30など）は月末に丸める
fn add_date_parts(base: NaiveDate, val: i32, unit: &str) -> Option<NaiveDate> {
    let mut y = base.year();
    let mut m = base.month();
    let mut d = base.day();

    match unit {
        "year" | "years" => {
            y += val;
        },
        "month" | "months" => {
            // 月の計算（年またぎ対応）
            let total_months = y * 12 + (m as i32 - 1) + val;
            y = total_months / 12;
            m = (total_months % 12 + 1) as u32;
        },
        _ => return None,
    }

    // 存在しない日付の調整 (例: 1/31 + 1 month -> 2/28 or 2/29)
    // chronoの with_ymd_opt は無効な日付だとNoneを返すので、有効になるまで日を減らす
    let mut new_date = NaiveDate::from_ymd_opt(y, m, d);
    while new_date.is_none() && d > 28 {
        d -= 1;
        new_date = NaiveDate::from_ymd_opt(y, m, d);
    }
    new_date
}

fn apply_input_patches(expression: &str, date_str: &str) -> String {
    // 1. 基本置換 (today, now, date -> @YYYY-MM-DD)
    let mut expr = expression
        .replace("date", date_str)
        .replace("today", date_str)
        .replace("now", date_str); // 時刻非対応のため、nowも日付のみに置換

    // 2. Week置換 (1 week = 7 days は定義として不変なので単純置換でOK)
    let re_week = Regex::new(r"(\d+)\s*weeks?").unwrap();
    expr = re_week.replace_all(&expr, "$1 * 7 days").to_string();

    // 3. 正確な Year/Month 計算 (Regex + Chrono)
    // パターン: @YYYY-MM-DD +/- N years/months
    // 連続計算 (today + 1 year + 1 month) に対応するため、マッチしなくなるまでループする
    let re_calc = Regex::new(r"(@\d{4}-\d{2}-\d{2})\s*([+\-])\s*(\d+)\s*(years?|months?)").unwrap();
    
    // ループ制限（無限ループ防止）
    for _ in 0..10 {
        if !re_calc.is_match(&expr) {
            break;
        }
        
        // replace_all で一括置換
        expr = re_calc.replace_all(&expr, |caps: &regex::Captures| {
            let base_str = &caps[1][1..]; // @を除く
            let op = &caps[2];
            let val_str = &caps[3];
            let unit = &caps[4];

            if let (Ok(base_date), Ok(val)) = (
                NaiveDate::parse_from_str(base_str, "%Y-%m-%d"),
                val_str.parse::<i32>()
            ) {
                // 引き算なら値を負にする
                let signed_val = if op == "-" { -val } else { val };
                
                if let Some(new_date) = add_date_parts(base_date, signed_val, unit) {
                    // 計算結果を新しいリテラルに置換: @YYYY-MM-DD
                    return format!("@{}", new_date.format("%Y-%m-%d"));
                }
            }
            // パース失敗などは元の文字列を維持
            caps[0].to_string()
        }).to_string();
    }

    // 4. 日付同士の引き算 (@YYYY-MM-DD - @YYYY-MM-DD)
    let re_sub = Regex::new(r"(@\d{4}-\d{2}-\d{2})\s*-\s*(@\d{4}-\d{2}-\d{2})").unwrap();
    if re_sub.is_match(&expr) {
        expr = re_sub.replace_all(&expr, |caps: &regex::Captures| {
            let d1_str = &caps[1][1..]; 
            let d2_str = &caps[2][1..]; 
            if let (Ok(d1), Ok(d2)) = (
                NaiveDate::parse_from_str(d1_str, "%Y-%m-%d"),
                NaiveDate::parse_from_str(d2_str, "%Y-%m-%d")
            ) {
                let diff = d1.signed_duration_since(d2);
                format!("{} days", diff.num_days())
            } else {
                caps[0].to_string()
            }
        }).to_string();
    }

    expr
}

// -----------------------------------------------------------------------------
// Python Interface (PyO3)
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
fn calculate(expression: String) -> PyResult<String> {
    use chrono::Local;
    
    let now = Local::now();
    let date_str = now.format("@%Y-%m-%d").to_string();
    
    let expr_processed = apply_input_patches(&expression, &date_str);

    let mut context = fend_core::Context::new();
    match fend_core::evaluate(&expr_processed, &mut context) {
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

        let date = js_sys::Date::new_0();
        let y = date.get_full_year();
        let m = date.get_month() + 1; 
        let d = date.get_date();
        let date_str = format!("@{}-{:02}-{:02}", y, m as u8, d as u8);
        
        let expr_processed = apply_input_patches(expression, &date_str);

        match fend_core::evaluate(&expr_processed, &mut self.ctx) {
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