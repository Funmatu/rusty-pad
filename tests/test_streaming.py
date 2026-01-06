import os
import rusty_pad
import time


def create_large_dummy_file(filename, size_mb):
    """ダミーの巨大ファイルを生成する"""
    print(f"Generating {size_mb}MB dummy file: {filename} ...")
    with open(filename, "w", encoding="utf-8") as f:
        # 1行 約100バイト
        line = "This is a test line for rusty-pad streaming performance verification. 0123456789\n"
        for _ in range(size_mb * 10000):  # おおよそのサイズ
            f.write(line)
    print("Done.")


def test_streaming():
    filename = "large_test.txt"
    # 100MB程度のファイルでテスト（ディスク容量に応じて調整可）
    create_large_dummy_file(filename, 100)

    print("\n--- Testing Streaming Stats ---")
    start = time.time()
    # ここでRustが全行を舐めるが、メモリには一行しか乗らないはず
    stats = rusty_pad.get_file_stats(filename)
    end = time.time()

    print(f"Stats: {stats}")  # (chars, words, lines)
    print(f"Time: {end - start:.4f} sec")

    print("\n--- Testing Pagination Read (Line 5000-5010) ---")
    # 巨大ファイルの途中だけをピンポイントで読む
    chunk = rusty_pad.read_file_chunk(filename, 5000, 10)
    print(f"Chunk Output:\n{chunk}")

    # 後始末
    os.remove(filename)


if __name__ == "__main__":
    test_streaming()
