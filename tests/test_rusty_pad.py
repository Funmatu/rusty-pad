import pytest
import rusty_pad


# --- Helper Functions ---
def parse_fend_value(res_str):
    """
    fend-coreの出力文字列から数値を抽出するヘルパー。
    '1000 m' (単位付き) や 'approx. 3.1415...' (近似値) などの形式に堅牢に対応。
    """
    # 1. 'approx.' という単語が含まれていれば除去する
    #    (例: "approx. 3.14159..." -> " 3.14159...")
    clean_str = res_str.replace("approx.", "").strip()

    # 2. 空白で分割し、数値として解釈できる最初のトークンを探す
    #    (例: "3.14159..." -> ["3.14159..."] -> 3.14159)
    #    (例: "100 km" -> ["100", "km"] -> 100)
    for token in clean_str.split():
        try:
            return float(token)
        except ValueError:
            continue

    # 数値が見つからなかった場合
    return float("nan")


# --- Test Classes ---


class TestCalculator:
    """電卓機能（Rust: fend-core）のテスト"""

    @pytest.mark.parametrize(
        "expr, expected_str, expected_val",
        [
            ("1 + 2", "3", 3.0),
            ("10 / 2 * 3", "15", 15.0),
            ("sqrt(16) + 2", "6", 6.0),
            ("(2 + 3) * 4", "20", 20.0),
            ("1 km to m", "1000 m", 1000.0),  # 単位変換
            ("0xFF to decimal", "255", 255.0),  # 16進数
        ],
    )
    def test_arithmetic_and_units(self, expr, expected_str, expected_val):
        """基本演算、単位変換、基数変換のテスト"""
        result = rusty_pad.calculate(expr)

        # 1. まずは理想的な文字列表現との完全一致を試みる
        if result == expected_str:
            return

        # 2. 表記ゆれ（例: 3.0 vs 3, approx.付き）がある場合は、数値として近似比較を行う
        val = parse_fend_value(result)
        assert val == pytest.approx(expected_val, abs=1e-6), (
            f"Value mismatch for '{expr}'. Got string: '{result}'"
        )

    def test_trigonometry(self):
        """三角関数などの科学計算テスト"""
        # sin(0) は厳密に 0
        assert rusty_pad.calculate("sin(0)") == "0"

        # 円周率は無理数なので近似値で比較
        # fendは "approx. 3.1415926536" のように返す場合がある
        res_pi = rusty_pad.calculate("pi")
        pi_val = parse_fend_value(res_pi)

        # デバッグ用に失敗時に生の値を出力させる
        assert pi_val == pytest.approx(3.14159265, abs=1e-8), (
            f"Failed to parse PI value. Raw output was: '{res_pi}'"
        )

    def test_error_handling(self):
        """不正な入力に対するエラーハンドリング"""
        # 無効な構文
        res = rusty_pad.calculate("invalid syntax")
        assert res.startswith("Error") or "expected" in res.lower()

        # ゼロ除算などは fend-core の仕様に従う（通常は infinity や Error）
        res_div_zero = rusty_pad.calculate("1 / 0")
        # fendは "infinity" を返す場合がある
        assert "error" in res_div_zero.lower() or "infinity" in res_div_zero.lower()


class TestNotepadStats:
    """メモ帳の統計情報機能のテスト"""

    def test_basic_english_text(self):
        """基本的な英文の統計"""
        text = "Hello Rust World\nThis is a test."
        chars, words, lines = rusty_pad.get_text_stats(text)

        # 検証内訳:
        # Chars: "Hello Rust World" (16) + "\n" (1) + "This is a test." (15) = 32
        # Words: Hello, Rust, World, This, is, a, test. = 7
        # Lines: 2行
        assert chars == 32
        assert words == 7
        assert lines == 2

    def test_multibyte_japanese_text(self):
        """日本語（マルチバイト文字）の処理確認"""
        text = "こんにちは\nRustの世界"
        chars, words, lines = rusty_pad.get_text_stats(text)

        # Chars: "こんにちは" (5) + "\n" (1) + "Rustの世界" (7) = 13
        # Words: Rustの split_whitespace は空白区切りのみを行う仕様。
        assert chars == 13
        assert words == 2
        assert lines == 2

    def test_empty_input(self):
        """空文字のエッジケース"""
        chars, words, lines = rusty_pad.get_text_stats("")
        assert chars == 0
        assert words == 0
        assert lines == 0

    def test_whitespace_only(self):
        """空白のみの入力"""
        text = "   \n  "
        chars, words, lines = rusty_pad.get_text_stats(text)

        assert chars == 6
        assert words == 0
        assert lines == 2
