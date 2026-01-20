# Release Guide for mote

このドキュメントでは、moteをHomebrew経由で配布するためのリリース手順を説明します。

## 📋 リリースフロー概要

moteは**release-please**を使用した自動リリースフローを採用しています。

```
Conventional Commits → mainにマージ → release-pleaseが自動でPR作成
→ PRをマージ → GitHub Release作成 → バイナリビルド → Homebrew Formula更新
```

## 🎯 通常のリリース手順（推奨）

### 1. Conventional Commitsでコミット

コミットメッセージを以下の形式で書きます:

```bash
# 新機能
git commit -m "feat: add new snapshot feature"

# バグ修正
git commit -m "fix: resolve file permission issue"

# パフォーマンス改善
git commit -m "perf: optimize hash calculation"

# リファクタリング
git commit -m "refactor: simplify storage module"

# ドキュメント
git commit -m "docs: update README with examples"

# その他
git commit -m "chore: update dependencies"
```

**バージョンへの影響**:
- `feat:` → MINOR version up (0.1.0 → 0.2.0)
- `fix:`, `perf:` → PATCH version up (0.1.0 → 0.1.1)
- `BREAKING CHANGE:` (フッター) → MAJOR version up (0.1.0 → 1.0.0)

### 2. mainブランチにマージ

```bash
# プルリクエストを作成してマージ
gh pr create --title "feat: add new feature" --body "..."
gh pr merge
```

または直接プッシュ:
```bash
git push origin main
```

### 3. release-pleaseが自動でPRを作成

GitHub Actionsが自動的に:
- CHANGELOG.mdを更新
- バージョン番号を更新
- Release PRを作成

**確認**:
```bash
# PRリストを確認
gh pr list

# 出力例:
# #123  chore(main): release 0.2.0  release-please
```

### 4. Release PRをマージ

```bash
# PRの内容を確認
gh pr view 123

# 問題なければマージ
gh pr merge 123 --squash
```

### 5. 自動で完了するもの

PRマージ後、自動的に以下が実行されます:

1. ✅ GitHub Releaseが作成される
2. ✅ 4プラットフォーム向けにバイナリがビルドされる
   - macOS (arm64/x86_64)
   - Linux (arm64/x86_64)
3. ✅ バイナリとSHA256ハッシュがReleaseにアップロードされる
4. ✅ Homebrew Formulaが自動更新される
5. ✅ `shabaraba/homebrew-tap`にプッシュされる

### 6. リリースの確認

```bash
# GitHub Releaseを確認
gh release view v0.2.0

# Homebrewでインストールテスト
brew upgrade mote
mote --version
```

## 🔧 前提条件

### 初回のみ: GitHub Secretの設定

Homebrew Formulaを自動更新するために、Personal Access Tokenが必要です。

#### 1. Personal Access Tokenを作成

1. GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. "Generate new token (classic)" をクリック
3. トークン名: `mote-homebrew-release`
4. スコープを選択:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `workflow` (Update GitHub Action workflows)
5. "Generate token" をクリック
6. トークンをコピー

#### 2. Secretを設定

```bash
cd ~/workspace/mote
gh secret set TAP_GITHUB_TOKEN
# プロンプトでトークンを貼り付け
```

または、GitHub Web UIで:
1. https://github.com/shabaraba/mote/settings/secrets/actions
2. "New repository secret"
3. Name: `TAP_GITHUB_TOKEN`
4. Secret: トークンを貼り付け

#### 3. homebrew-tapリポジトリの作成

```bash
# tapリポジトリを作成
gh repo create shabaraba/homebrew-tap --public

# クローン
cd ~
git clone git@github.com:shabaraba/homebrew-tap.git
cd homebrew-tap

# 初期化
mkdir -p Formula
echo "# Homebrew Tap for shabaraba's projects" > README.md
git add Formula README.md
git commit -m "chore: initialize tap repository"
git push
```

## 📝 Conventional Commits リファレンス

### 基本フォーマット

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### type一覧

| type | 説明 | バージョンへの影響 |
|------|------|------------------|
| `feat` | 新機能 | MINOR (0.1.0 → 0.2.0) |
| `fix` | バグ修正 | PATCH (0.1.0 → 0.1.1) |
| `perf` | パフォーマンス改善 | PATCH |
| `refactor` | リファクタリング | なし* |
| `docs` | ドキュメント | なし* |
| `chore` | その他 | なし* |
| `test` | テスト追加 | なし* |
| `ci` | CI設定 | なし* |

\* CHANGELOGには記載されるが、バージョンは上がらない

### Breaking Changeの指定

MAJOR version upさせる場合:

```bash
git commit -m "feat!: change snapshot format

BREAKING CHANGE: Old snapshots are not compatible"
```

または:

```bash
git commit -m "feat: change snapshot format" --trailer "BREAKING CHANGE: Old snapshots are not compatible"
```

### 例

```bash
# 新機能（MINOR up）
git commit -m "feat: add diff with --content flag"

# バグ修正（PATCH up）
git commit -m "fix: resolve permission error on restore"

# スコープ付き
git commit -m "feat(cli): add --dry-run option to restore command"

# 複数行
git commit -m "feat: add auto-cleanup feature

Automatically cleanup old snapshots based on:
- max_snapshots configuration
- max_age_days configuration

Closes #123"

# Breaking Change（MAJOR up）
git commit -m "feat!: change storage format

BREAKING CHANGE: Storage format changed to improve performance.
Old snapshots need to be migrated using 'mote migrate' command."
```

## 🚨 緊急リリース（手動）

release-pleaseを経由せずに緊急リリースする場合:

### 1. バージョン番号を手動で更新

```bash
# Cargo.tomlのバージョンを更新
vim Cargo.toml
# version = "0.1.1"

# manifestも更新
vim .release-please-manifest.json
# { ".": "0.1.1" }
```

### 2. CHANGELOG.mdを手動で更新

```bash
vim CHANGELOG.md
```

### 3. コミット＆タグ作成

```bash
git add Cargo.toml .release-please-manifest.json CHANGELOG.md
git commit -m "chore: release 0.1.1"
git push origin main

# タグを作成
git tag v0.1.1
git push origin v0.1.1
```

### 4. GitHub Releaseを手動で作成

```bash
gh release create v0.1.1 \
  --title "v0.1.1" \
  --notes "Emergency release for critical bug fix"
```

これでバイナリビルドとHomebrew Formula更新が自動的に開始されます。

## 🐛 トラブルシューティング

### release-pleaseのPRが作成されない

**原因**: Conventional Commitsの形式が間違っている

**対処**:
```bash
# 最近のコミットメッセージを確認
git log --oneline -10

# 形式が正しいか確認（feat:, fix:, などで始まっているか）
```

### バイナリビルドが失敗する

**原因**: クロスコンパイルの問題

**対処**:
```bash
# ローカルでテスト
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
```

### Homebrew Formula更新が失敗する

**原因**: `TAP_GITHUB_TOKEN` が設定されていない

**対処**:
```bash
# Secretを確認
gh secret list

# 設定されていなければ設定
gh secret set TAP_GITHUB_TOKEN
```

### SHA256ハッシュが一致しない

**原因**: バイナリが破損している

**対処**:
```bash
# Releaseページから手動でSHA256を取得
curl -sL https://github.com/shabaraba/mote/releases/download/v0.1.0/mote-v0.1.0-aarch64-apple-darwin.tar.gz | shasum -a 256

# homebrew-tapリポジトリで手動修正
cd ~/homebrew-tap
vim Formula/mote.rb
# SHA256を修正
git add Formula/mote.rb
git commit -m "fix: correct SHA256 hash"
git push
```

## 📊 バージョニング戦略

### Semantic Versioning

- **MAJOR** (x.0.0): 後方互換性のない変更
- **MINOR** (0.x.0): 後方互換性のある機能追加
- **PATCH** (0.0.x): 後方互換性のあるバグ修正

### pre-1.0.0の特例

- 0.x.y バージョンでは、`feat` → MINOR、`fix` → PATCH
- BREAKING CHANGEがあってもMAJORは上げない（1.0.0まで）

### 推奨される開発フロー

```
0.1.0 (初期リリース)
  ↓ feat: add feature A
0.2.0
  ↓ fix: bug fix
0.2.1
  ↓ feat: add feature B
0.3.0
  ↓ 安定版と判断
1.0.0 (安定版リリース)
  ↓ feat: add feature C
1.1.0
  ↓ BREAKING CHANGE
2.0.0
```

## ✅ リリースチェックリスト

### リリース前
- [ ] 全テストが成功 (`cargo test`)
- [ ] ドキュメントが更新されている
- [ ] Conventional Commitsで適切にコミットされている
- [ ] release-pleaseのPRが作成されている
- [ ] CHANGELOGが正しく生成されている

### リリース実行
- [ ] Release PRをマージ
- [ ] GitHub Actionsが成功している
- [ ] GitHub Releaseが作成されている
- [ ] 4つのバイナリがアップロードされている
- [ ] Homebrew Formulaが更新されている

### リリース後
- [ ] `brew install mote` でインストールできる
- [ ] `mote --version` で正しいバージョンが表示される
- [ ] 基本的な動作確認が完了している

## 📚 参考リンク

- [Conventional Commits](https://www.conventionalcommits.org/)
- [release-please documentation](https://github.com/googleapis/release-please)
- [Semantic Versioning](https://semver.org/)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

---

**TL;DR**:
1. Conventional Commitsでコミット (`feat:`, `fix:` など)
2. mainにマージ
3. release-pleaseが自動でPRを作成
4. PRをマージ
5. 自動でリリース完了！ 🎉
