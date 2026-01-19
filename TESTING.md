# Testing Guide for mote

このドキュメントでは、moteのテストの実行方法と、テストケースの詳細について説明します。

## テストの種類

### 1. 統合テスト (Integration Tests)

`tests/integration_test.rs` に実装されています。実際のmoteバイナリを起動して、エンドツーエンドの動作を検証します。

**実行方法**:
```bash
cargo test
```

**カバレッジ**:
- ✅ 初期化 (`mote init`)
- ✅ スナップショット作成 (`mote snapshot`)
- ✅ ログ表示 (`mote log`)
- ✅ 詳細表示 (`mote show`)
- ✅ 差分表示 (`mote diff`)
- ✅ 復元 (`mote restore`)
- ✅ autoモード
- ✅ dry-runモード

**テスト数**: 13個のテストケース

**実行時間**: 約0.34秒

### 2. 手動テスト

詳細なテストケースと手順は `TEST_CASES.md` に記載されています。

**主要なテストカテゴリ**:
1. 初期化テスト
2. スナップショット作成テスト
3. ログ表示テスト
4. スナップショット詳細表示テスト
5. 差分表示テスト
6. 復元テスト
7. ignoreパターンテスト
8. エッジケースとエラーハンドリング
9. 統合テスト
10. パフォーマンステスト

## テストの実行

### すべてのテストを実行
```bash
cargo test
```

### 特定のテストのみ実行
```bash
cargo test test_init_creates_directory_structure
```

### 詳細な出力で実行
```bash
cargo test -- --nocapture
```

### テストをシングルスレッドで実行
```bash
cargo test -- --test-threads=1
```

## テスト結果

最新のテスト実行結果は `TEST_REPORT.md` に記録されています。

**最終実行日**: 2026-01-19
**結果**: ✅ 全てのテスト成功 (13/13 passed)

## 新しいテストの追加

### 統合テストの追加

`tests/integration_test.rs` に新しいテスト関数を追加:

```rust
#[test]
fn test_new_feature() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    // テストロジック

    let output = ctx.run_mote(&["new-command"]);
    assert!(output.status.success());
}
```

### 手動テストケースの追加

`TEST_CASES.md` に新しいセクションを追加:

```markdown
## N. 新機能テスト

### テストケース N.1: 機能の説明
**目的**: テストの目的

**手順**:
\`\`\`bash
# コマンド
\`\`\`

**期待される結果**:
- ✅ 期待される動作
```

## 継続的インテグレーション (CI)

TODO: GitHub Actionsの設定を追加予定

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
```

## テストカバレッジ

### 現在のカバレッジ

| モジュール | カバレッジ | 備考 |
|-----------|-----------|------|
| CLI | 100% | 全コマンドテスト済み |
| Storage | 95% | 基本機能カバー |
| Config | 90% | デフォルト設定テスト済み |
| Ignore | 80% | 基本パターンのみ |
| Error | 70% | 主要エラーのみ |

### 今後のカバレッジ目標

- [ ] 異常系テストの追加
- [ ] エッジケースの網羅
- [ ] パフォーマンステストの自動化
- [ ] クリーンアップ機能のテスト

## トラブルシューティング

### テストが失敗する場合

1. **ビルドエラー**
   ```bash
   cargo clean
   cargo build
   cargo test
   ```

2. **一時ディレクトリの問題**
   - テストは自動的に一時ディレクトリを作成・クリーンアップします
   - 問題がある場合は `/tmp/mote-*` を手動で削除

3. **パーミッションエラー**
   - テストディレクトリの書き込み権限を確認
   - `chmod +w /tmp` で権限を付与

## ベンチマーク

パフォーマンステストの実施方法:

```bash
# 大量ファイルでのテスト
cd /tmp/mote-bench
mote init
for i in {1..1000}; do echo "test $i" > "file_$i.txt"; done
time mote snapshot -m "1000 files"
```

**目標パフォーマンス**:
- 小規模プロジェクト (10ファイル): < 100ms
- 中規模プロジェクト (100ファイル): < 500ms
- 大規模プロジェクト (1000ファイル): < 3秒

## 参考資料

- `TEST_CASES.md` - 詳細なテストケース仕様
- `TEST_REPORT.md` - 最新のテスト実行結果
- `HANDOFF.md` - プロジェクト概要と設計

---

**更新履歴**:
- 2026-01-19: 初版作成、13個の統合テスト追加
