# mote テストケース仕様書

## テスト環境

- **実行環境**: macOS (テスト実施済み)
- **Rustバージョン**: 2021 edition
- **テストディレクトリ**: `/tmp/mote-test` (推奨)

## 1. 初期化テスト (init)

### テストケース 1.1: 基本的な初期化
**目的**: moteの初期化が正常に完了すること

**手順**:
```bash
mkdir -p /tmp/mote-test
cd /tmp/mote-test
mote init
```

**期待される結果**:
- ✅ `.mote` ディレクトリが作成される
- ✅ `.moteignore` ファイルが作成される
- ✅ "✓ Initialized mote in ..." メッセージが表示される
- ✅ `.mote/objects/` ディレクトリが存在する
- ✅ `.mote/snapshots/` ディレクトリが存在する

**確認方法**:
```bash
ls -la .mote/
ls -la .moteignore
```

### テストケース 1.2: 重複初期化
**目的**: 既に初期化されたディレクトリでの再初期化が適切に処理されること

**手順**:
```bash
mote init
mote init
```

**期待される結果**:
- ✅ エラーなく完了するか、適切な警告メッセージが表示される
- ✅ 既存のスナップショットデータが保持される

---

## 2. スナップショット作成テスト (snapshot)

### テストケース 2.1: 初回スナップショット作成
**目的**: ファイルのスナップショットが正常に作成されること

**手順**:
```bash
echo "Hello World" > test1.txt
echo "Test content" > test2.txt
mote snapshot -m "Initial snapshot"
```

**期待される結果**:
- ✅ "✓ Created snapshot [id] (N files)" メッセージが表示される
- ✅ スナップショットIDが表示される
- ✅ ファイル数が正しい (test1.txt, test2.txt, .moteignore = 3ファイル)
- ✅ メッセージが "Initial snapshot" として記録される

**確認方法**:
```bash
mote log
ls .mote/snapshots/
```

### テストケース 2.2: メッセージなしスナップショット
**目的**: メッセージオプションなしでもスナップショットが作成されること

**手順**:
```bash
echo "New content" > test3.txt
mote snapshot
```

**期待される結果**:
- ✅ スナップショットが正常に作成される
- ✅ メッセージは表示されない（またはnull/空）

### テストケース 2.3: triggerオプション
**目的**: トリガー情報が記録されること

**手順**:
```bash
echo "Modified" > test1.txt
mote snapshot -m "After edit" -t "claude-code-hook"
```

**期待される結果**:
- ✅ スナップショットが作成される
- ✅ トリガー情報が記録される

**確認方法**:
```bash
mote log
mote show [snapshot-id]
```

### テストケース 2.4: autoモード - 変更なし
**目的**: 変更がない場合、autoモードでスナップショットを作成しないこと

**手順**:
```bash
mote snapshot --auto
mote snapshot --auto
```

**期待される結果**:
- ✅ 2回目のコマンドで新しいスナップショットが作成されない
- ✅ 出力がない（quietモード）

**確認方法**:
```bash
mote log --limit 5
```

### テストケース 2.5: autoモード - 変更あり
**目的**: 変更がある場合、autoモードでスナップショットを作成すること

**手順**:
```bash
echo "Auto change" >> test2.txt
mote snapshot --auto
```

**期待される結果**:
- ✅ 新しいスナップショットが作成される
- ✅ 出力がない（quietモード）

### テストケース 2.6: 空のプロジェクト
**目的**: 追跡対象ファイルがない場合の処理

**手順**:
```bash
rm test*.txt
mote snapshot
```

**期待される結果**:
- ✅ "! No files to snapshot" メッセージが表示される
- ✅ スナップショットは作成されない

---

## 3. ログ表示テスト (log)

### テストケース 3.1: 標準ログ表示
**目的**: スナップショット履歴が正しく表示されること

**手順**:
```bash
mote log
```

**期待される結果**:
- ✅ 最新のスナップショットが最初に表示される
- ✅ 各エントリに以下が含まれる:
  - snapshot ID (短縮形)
  - Date (タイムスタンプ)
  - Message (あれば)
  - Trigger (あれば)
  - Files (ファイル数)

### テストケース 3.2: oneline形式
**目的**: コンパクトな1行形式で表示されること

**手順**:
```bash
mote log --oneline
```

**期待される結果**:
- ✅ 各スナップショットが1行で表示される
- ✅ ID、日時、メッセージ、ファイル数が含まれる

### テストケース 3.3: limit指定
**目的**: 表示件数が制限されること

**手順**:
```bash
mote log --limit 5
```

**期待される結果**:
- ✅ 最大5件のスナップショットのみ表示される

### テストケース 3.4: スナップショットがない場合
**目的**: スナップショットがない場合の処理

**手順**:
```bash
# 新しいディレクトリで
mkdir /tmp/mote-test-empty
cd /tmp/mote-test-empty
mote init
mote log
```

**期待される結果**:
- ✅ "! No snapshots yet" メッセージが表示される

---

## 4. スナップショット詳細表示テスト (show)

### テストケース 4.1: 詳細情報表示
**目的**: スナップショットの詳細情報が表示されること

**手順**:
```bash
# snapshot IDを取得
SNAP_ID=$(mote log --oneline | head -1 | awk '{print $1}')
mote show $SNAP_ID
```

**期待される結果**:
- ✅ 完全なスナップショットIDが表示される
- ✅ タイムスタンプが表示される
- ✅ メッセージ（あれば）が表示される
- ✅ トリガー（あれば）が表示される
- ✅ 全ファイルのリストが表示される（パスとバイト数）

### テストケース 4.2: 短縮IDでの指定
**目的**: 短縮形のIDで詳細表示できること

**手順**:
```bash
mote show cdd78  # 最初の数文字のみ
```

**期待される結果**:
- ✅ 正しいスナップショットが表示される

### テストケース 4.3: 存在しないID
**目的**: 存在しないIDのエラーハンドリング

**手順**:
```bash
mote show nonexistent123
```

**期待される結果**:
- ✅ エラーメッセージが表示される
- ✅ exit code 1

---

## 5. 差分表示テスト (diff)

### テストケース 5.1: 作業ディレクトリとの比較
**目的**: スナップショットと現在の作業ディレクトリの差分が表示されること

**手順**:
```bash
SNAP_ID=$(mote log --oneline | tail -1 | awk '{print $1}')
echo "Modified content" > test1.txt
echo "New file" > test3.txt
mote diff $SNAP_ID
```

**期待される結果**:
- ✅ "Comparing [id] -> working directory" メッセージ
- ✅ Modified: test1.txt が表示される
- ✅ Added: test3.txt が表示される

### テストケース 5.2: スナップショット間の比較
**目的**: 2つのスナップショット間の差分が表示されること

**手順**:
```bash
SNAP1=$(mote log --oneline | tail -1 | awk '{print $1}')
SNAP2=$(mote log --oneline | head -1 | awk '{print $1}')
mote diff $SNAP1 $SNAP2
```

**期待される結果**:
- ✅ "Comparing [id1] -> [id2]" メッセージ
- ✅ 変更されたファイルが表示される
- ✅ 追加されたファイルが表示される
- ✅ 削除されたファイルが表示される

### テストケース 5.3: contentオプション付きdiff
**目的**: ファイル内容の差分が表示されること

**手順**:
```bash
mote diff $SNAP1 $SNAP2 --content
```

**期待される結果**:
- ✅ ファイルリストに加えて、行単位の差分が表示される
- ✅ 削除行が赤色で `-` プレフィックス付きで表示される
- ✅ 追加行が緑色で `+` プレフィックス付きで表示される
- ✅ 行番号が表示される

### テストケース 5.4: 変更がない場合
**目的**: 差分がない場合の処理

**手順**:
```bash
mote snapshot -m "Same state"
SNAP_ID=$(mote log --oneline | head -1 | awk '{print $1}')
mote diff $SNAP_ID
```

**期待される結果**:
- ✅ 何も表示されない、または "No changes" メッセージ

---

## 6. 復元テスト (restore)

### テストケース 6.1: 単一ファイルの復元
**目的**: 特定ファイルが正しく復元されること

**手順**:
```bash
SNAP_ID=$(mote log --oneline | tail -1 | awk '{print $1}')
echo "Current content" > test1.txt
mote restore $SNAP_ID --file test1.txt
cat test1.txt
```

**期待される結果**:
- ✅ "✓ Restored: test1.txt" メッセージが表示される
- ✅ ファイルがスナップショット時の内容に戻る

### テストケース 6.2: プロジェクト全体の復元
**目的**: 全ファイルが復元されること

**手順**:
```bash
SNAP_ID=$(mote log --oneline | tail -1 | awk '{print $1}')
echo "Changed" > test1.txt
echo "Changed" > test2.txt
mote restore $SNAP_ID
```

**期待される結果**:
- ✅ 自動的にバックアップスナップショットが作成される
- ✅ "✓ Created backup snapshot: [id]" メッセージ
- ✅ 全ファイルが復元される
- ✅ "✓ Restored N file(s)" メッセージ

### テストケース 6.3: dry-runモード
**目的**: 実際に復元せず、復元対象が表示されること

**手順**:
```bash
mote restore $SNAP_ID --dry-run
ls test*.txt  # ファイルが変更されていないことを確認
```

**期待される結果**:
- ✅ "dry-run Would restore: [file]" メッセージが各ファイルに表示される
- ✅ 実際のファイルは変更されない
- ✅ バックアップスナップショットは作成されない

### テストケース 6.4: forceオプション
**目的**: 変更されたファイルを強制的に上書きできること

**手順**:
```bash
echo "Local changes" > test1.txt
mote restore $SNAP_ID --force
```

**期待される結果**:
- ✅ 警告なしに全ファイルが復元される
- ✅ バックアップスナップショットが作成される

### テストケース 6.5: 変更されたファイルのスキップ
**目的**: forceなしで変更ファイルがスキップされること

**手順**:
```bash
echo "Local changes" > test1.txt
mote restore $SNAP_ID
```

**期待される結果**:
- ✅ "Skipped: test1.txt (use --force to overwrite)" メッセージ
- ✅ 変更されていないファイルは復元される
- ✅ "Skipped N modified file(s)" サマリーが表示される

### テストケース 6.6: 存在しないファイルの復元
**目的**: スナップショットに存在しないファイルのエラーハンドリング

**手順**:
```bash
mote restore $SNAP_ID --file nonexistent.txt
```

**期待される結果**:
- ✅ エラーメッセージが表示される
- ✅ exit code 1

---

## 7. ignoreパターンテスト

### テストケース 7.1: .moteignoreの尊重
**目的**: .moteignoreパターンに一致するファイルが除外されること

**手順**:
```bash
cat .moteignore  # デフォルトパターンを確認
mkdir node_modules
echo "test" > node_modules/test.js
mote snapshot -m "With ignored files"
mote show [snapshot-id]
```

**期待される結果**:
- ✅ node_modules/ 内のファイルがスナップショットに含まれない
- ✅ .git/ ディレクトリが除外される
- ✅ target/ ディレクトリが除外される

### テストケース 7.2: カスタムignoreパターン
**目的**: カスタムパターンが機能すること

**手順**:
```bash
echo "*.tmp" >> .moteignore
echo "test" > test.tmp
mote snapshot -m "Custom ignore"
mote show [snapshot-id]
```

**期待される結果**:
- ✅ test.tmp がスナップショットに含まれない

---

## 8. エッジケースとエラーハンドリング

### テストケース 8.1: 未初期化ディレクトリでのコマンド実行
**目的**: 適切なエラーメッセージが表示されること

**手順**:
```bash
mkdir /tmp/mote-test-uninit
cd /tmp/mote-test-uninit
mote snapshot
```

**期待される結果**:
- ✅ "error: ..." メッセージが表示される
- ✅ exit code 1

### テストケース 8.2: 破損したオブジェクトファイル
**目的**: 破損データの適切な処理

**手順**:
```bash
# objectファイルを意図的に破損
echo "corrupted" > .mote/objects/[hash-prefix]/[hash]
mote restore [snapshot-id]
```

**期待される結果**:
- ✅ エラーメッセージが表示される
- ✅ 他のファイルの復元は継続される（可能な限り）

### テストケース 8.3: ディスク容量不足シミュレーション
**目的**: I/Oエラーの適切な処理

**実施注意**: 実際のディスク容量不足を再現するのは危険なため、代わりにパーミッションエラーでテスト

**手順**:
```bash
chmod -w .mote/objects
mote snapshot -m "Readonly test"
chmod +w .mote/objects
```

**期待される結果**:
- ✅ 適切なエラーメッセージが表示される
- ✅ プログラムがクラッシュしない

### テストケース 8.4: 大量ファイルのスナップショット
**目的**: パフォーマンステスト

**手順**:
```bash
for i in {1..1000}; do echo "test $i" > "file_$i.txt"; done
time mote snapshot -m "1000 files"
```

**期待される結果**:
- ✅ 100ms〜数秒以内で完了（環境依存）
- ✅ メモリエラーが発生しない
- ✅ 全ファイルが正しく記録される

---

## 9. 統合テスト

### テストケース 9.1: 完全なワークフロー
**目的**: 実際の使用シナリオの動作確認

**手順**:
```bash
# 1. 初期化
mkdir /tmp/mote-integration-test
cd /tmp/mote-integration-test
mote init

# 2. 初期ファイル作成とスナップショット
echo "v1" > app.txt
mote snapshot -m "Initial version"
SNAP1=$(mote log --oneline | head -1 | awk '{print $1}')

# 3. 編集とスナップショット
echo "v2" > app.txt
mote snapshot -m "Second version"
SNAP2=$(mote log --oneline | head -1 | awk '{print $1}')

# 4. さらに編集
echo "v3" > app.txt

# 5. 差分確認
mote diff $SNAP1
mote diff $SNAP1 $SNAP2 --content

# 6. 復元
mote restore $SNAP1 --file app.txt

# 7. 確認
cat app.txt  # "v1" が表示されるべき
```

**期待される結果**:
- ✅ 全ステップがエラーなく完了する
- ✅ 最終的に app.txt の内容が "v1" に戻る

---

## 10. パフォーマンステスト

### テストケース 10.1: スナップショット作成速度
**目的**: 小規模プロジェクトで100ms以内

**手順**:
```bash
# 10ファイル程度のプロジェクト
for i in {1..10}; do echo "content $i" > "file_$i.txt"; done
time mote snapshot -m "Performance test"
```

**成功基準**:
- ✅ 100ms以内（小規模プロジェクト、SSDの場合）

### テストケース 10.2: ストレージ効率
**目的**: 重複ファイルが1度だけ保存されること

**手順**:
```bash
echo "same content" > file1.txt
echo "same content" > file2.txt
mote snapshot -m "Duplicate content"
ls -la .mote/objects/
```

**期待される結果**:
- ✅ objectsディレクトリに同じ内容のファイルが1つだけ保存される
- ✅ 両ファイルが同じハッシュを参照している

---

## テスト実施チェックリスト

- [ ] 全テストケースを実施
- [ ] macOS環境で動作確認
- [ ] Linux環境で動作確認（可能であれば）
- [ ] パフォーマンス基準を満たしている
- [ ] エラーメッセージが分かりやすい
- [ ] ヘルプメッセージが正確

---

## 自動テストの推奨

以下のテストケースは自動化が推奨されます:

1. **単体テスト**（`tests/` ディレクトリ内）
   - ObjectStore のハッシュ計算
   - SnapshotStore の保存・読み込み
   - IgnoreFilter のパターンマッチング

2. **統合テスト**
   - 各CLIコマンドのend-to-endテスト
   - 一時ディレクトリを使用した完全なワークフロー

3. **回帰テスト**
   - 既知のバグ修正の再発防止
   - 互換性の維持

---

## バグ報告時の情報

テストで問題を発見した場合、以下の情報を含めてください:

- OS とバージョン
- Rust のバージョン
- mote のバージョン
- 再現手順
- 期待される動作
- 実際の動作
- エラーメッセージ（あれば）
- 関連するログファイル

---

最終更新: 2026-01-19
