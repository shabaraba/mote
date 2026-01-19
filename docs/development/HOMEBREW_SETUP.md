# Homebrew配信セットアップガイド (release-please版)

moteをrelease-please + Homebrewで配信するための完全なセットアップ手順です。

## 📋 概要

このプロジェクトは以下の自動化フローを採用しています:

```
┌─────────────────┐
│ Conventional    │
│ Commits         │ (feat:, fix:, etc.)
└────────┬────────┘
         │ push to main
         ▼
┌─────────────────┐
│ release-please  │ PRを自動作成
│ GitHub Actions  │ (CHANGELOG, version更新)
└────────┬────────┘
         │ PRをマージ
         ▼
┌─────────────────┐
│ GitHub Release  │ 自動作成
│ が作成される    │
└────────┬────────┘
         │ トリガー
         ▼
┌─────────────────┐
│ Build & Release │ 4プラットフォーム向け
│ GitHub Actions  │ バイナリビルド
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Homebrew Tap    │ Formula自動更新
│ 更新            │
└─────────────────┘
```

## 🚀 初回セットアップ（一度だけ実行）

### 1. homebrew-tapリポジトリの作成

```bash
# GitHubでtapリポジトリを作成
gh repo create shabaraba/homebrew-tap --public --description "Homebrew tap for shabaraba's projects"

# ローカルにクローン
cd ~
git clone git@github.com:shabaraba/homebrew-tap.git
cd homebrew-tap

# Formulaディレクトリを作成
mkdir -p Formula
cat > README.md << 'EOF'
# Homebrew Tap for shabaraba's projects

## Installation

```bash
brew tap shabaraba/tap
brew install mote
```
EOF

git add Formula README.md
git commit -m "chore: initialize tap repository"
git push
```

### 2. Personal Access Token (PAT)の作成

1. GitHub Settings → Developer settings → Personal access tokens → Tokens (classic) にアクセス
2. "Generate new token (classic)" をクリック
3. トークン名: `mote-homebrew-release`
4. 有効期限: 無期限 or 1年
5. 以下のスコープを選択:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `workflow` (Update GitHub Action workflows)
6. "Generate token" をクリック
7. **トークンをコピーして安全な場所に保存**

### 3. GitHub Secretの設定

```bash
# moteリポジトリに移動
cd ~/workspace/mote

# コマンドラインでSecretを設定（推奨）
gh secret set TAP_GITHUB_TOKEN
# プロンプトが表示されたら、コピーしたトークンを貼り付け
```

**または**、GitHubのWeb UIで設定:
1. https://github.com/shabaraba/mote/settings/secrets/actions にアクセス
2. "New repository secret" をクリック
3. Name: `TAP_GITHUB_TOKEN`
4. Secret: コピーしたトークンを貼り付け
5. "Add secret" をクリック

### 4. 設定の確認

```bash
# Secretが正しく設定されているか確認
gh secret list

# 出力例:
# TAP_GITHUB_TOKEN  Updated 2026-01-19
```

## 📦 リリース手順（通常時）

初回セットアップが完了したら、以下の手順でリリースできます。

### ステップ1: Conventional Commitsでコミット

コミットメッセージを規約に従って書きます:

```bash
# 新機能（MINOR version up）
git commit -m "feat: add new snapshot feature"

# バグ修正（PATCH version up）
git commit -m "fix: resolve permission error"

# パフォーマンス改善（PATCH version up）
git commit -m "perf: optimize hash calculation"

# ドキュメント（バージョンup なし、CHANGELOGには記載）
git commit -m "docs: update README"

# その他（バージョンup なし）
git commit -m "chore: update dependencies"
```

### ステップ2: mainブランチにプッシュ

```bash
git push origin main
```

### ステップ3: release-pleaseが自動でPRを作成

数分後、GitHub Actionsが自動的に:
- CHANGELOG.mdを生成/更新
- バージョン番号を更新（Cargo.toml, .release-please-manifest.json）
- Release PRを作成

**確認方法**:
```bash
# PRリストを確認
gh pr list

# 出力例:
# #123  chore(main): release 0.2.0  release-please[bot]
```

または:
```bash
# ブラウザでPRを確認
open https://github.com/shabaraba/mote/pulls
```

### ステップ4: Release PRをレビュー＆マージ

```bash
# PRの内容を確認
gh pr view 123

# 内容を確認したらマージ
gh pr merge 123 --squash
```

### ステップ5: 自動で全て完了！

PRマージ後、以下が自動的に実行されます（約10分）:

1. ✅ **GitHub Releaseが作成される**
   - タグ: v0.2.0
   - リリースノート: CHANGELOGから自動生成

2. ✅ **バイナリがビルドされる**（4プラットフォーム）
   - macOS arm64 (Apple Silicon)
   - macOS x86_64 (Intel)
   - Linux arm64
   - Linux x86_64

3. ✅ **バイナリとSHA256がReleaseにアップロード**

4. ✅ **Homebrew Formulaが自動更新**
   - バージョン番号
   - SHA256ハッシュ
   - ダウンロードURL

5. ✅ **homebrew-tapリポジトリに自動プッシュ**

### ステップ6: インストールテスト

```bash
# Homebrewでインストールテスト
brew update
brew upgrade mote

# バージョン確認
mote --version
# 出力: mote 0.2.0
```

## 🎯 実際の使い方例

### シナリオ: 新機能を追加してリリース

```bash
# 1. 機能を実装
vim src/main.rs
git add src/main.rs
git commit -m "feat: add --quiet flag to snapshot command"
git push origin main

# 2. release-pleaseがPRを作成（自動）
# 待つ: 約1-2分

# 3. PRを確認してマージ
gh pr list  # PRが作成されているか確認
gh pr view <PR番号>  # 内容を確認
gh pr merge <PR番号> --squash

# 4. 完了を待つ
# 待つ: 約10分

# 5. リリース確認
gh release view v0.2.0
brew upgrade mote
mote --version
```

### シナリオ: 複数の変更を含むリリース

```bash
# 1. 複数のコミットを追加
git commit -m "feat: add diff --context option"
git commit -m "fix: resolve crash on empty directory"
git commit -m "docs: add usage examples"
git push origin main

# 2. release-pleaseが全てのコミットを集約してPRを作成（自動）
# CHANGELOG:
# - Features: add diff --context option
# - Bug Fixes: resolve crash on empty directory
# - Documentation: add usage examples

# 3. PRをマージ（以下同じ）
```

## 🔧 トラブルシューティング

### release-pleaseのPRが作成されない

**症状**: mainにプッシュしてもPRが作成されない

**原因と対処**:

1. **Conventional Commitsの形式が間違っている**
   ```bash
   # NG例
   git commit -m "add new feature"  # typeがない
   git commit -m "feat add feature"  # コロンがない

   # OK例
   git commit -m "feat: add new feature"
   ```

2. **コミットにバージョンを上げる内容がない**
   ```bash
   # これらはバージョンを上げないので、PRは作成されない
   git commit -m "docs: update README"
   git commit -m "chore: update dependencies"
   git commit -m "refactor: simplify code"
   ```

   対処: `feat:` や `fix:` のコミットを追加

3. **release-please設定ファイルが間違っている**
   ```bash
   # 設定ファイルを確認
   cat release-please-config.json
   cat .release-please-manifest.json
   ```

### バイナリビルドが失敗する

**症状**: GitHub Releaseは作成されるが、バイナリがアップロードされない

**対処**:

```bash
# Actions画面でログを確認
open https://github.com/shabaraba/mote/actions

# ローカルでクロスコンパイルをテスト
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
```

### Homebrew Formula更新が失敗する

**症状**: `TAP_GITHUB_TOKEN` エラー

**対処**:

```bash
# Secretを再設定
gh secret set TAP_GITHUB_TOKEN

# または、手動でFormulaを更新
cd ~/homebrew-tap
# mote.rbを手動で編集
git add Formula/mote.rb
git commit -m "fix: update formula"
git push
```

### Homebrewインストールが失敗する

**症状**: `brew install mote` でSHA256エラー

**対処**:

```bash
# 正しいSHA256を取得
VERSION=v0.2.0
TARGET=aarch64-apple-darwin
curl -sL https://github.com/shabaraba/mote/releases/download/$VERSION/mote-$VERSION-$TARGET.tar.gz | shasum -a 256

# homebrew-tapで修正
cd ~/homebrew-tap
vim Formula/mote.rb
# SHA256を正しい値に修正
git add Formula/mote.rb
git commit -m "fix: correct SHA256 hash for $VERSION"
git push
```

## 📊 Conventional Commits クイックリファレンス

| Prefix | バージョンへの影響 | 例 |
|--------|------------------|-----|
| `feat:` | MINOR up (0.1.0 → 0.2.0) | `feat: add new command` |
| `fix:` | PATCH up (0.1.0 → 0.1.1) | `fix: resolve crash` |
| `perf:` | PATCH up | `perf: optimize query` |
| `docs:` | なし* | `docs: update README` |
| `chore:` | なし* | `chore: update deps` |
| `refactor:` | なし* | `refactor: simplify code` |
| `test:` | なし* | `test: add unit tests` |
| `ci:` | なし* | `ci: update workflow` |

\* CHANGELOGには記載されるが、バージョンは上がらない

**Breaking Change（MAJOR up）**:
```bash
git commit -m "feat!: change storage format

BREAKING CHANGE: Old format is not compatible"
```

## ✅ セットアップ完了チェックリスト

### 初回セットアップ
- [ ] `shabaraba/homebrew-tap` リポジトリを作成
- [ ] Personal Access Tokenを作成
- [ ] `TAP_GITHUB_TOKEN` Secretを設定
- [ ] `gh secret list` で確認

### 動作確認
- [ ] Conventional Commitsでコミット
- [ ] mainにプッシュ
- [ ] release-pleaseのPRが作成される
- [ ] PRをマージ
- [ ] GitHub Releaseが作成される
- [ ] バイナリがアップロードされる
- [ ] Homebrew Formulaが更新される
- [ ] `brew install mote` でインストールできる

## 📚 関連ドキュメント

- **RELEASE.md** - 詳細なリリース手順とトラブルシューティング
- **README.md** - ユーザー向けインストール手順
- **.github/workflows/release-please.yml** - release-please設定
- **.github/workflows/release.yml** - バイナリビルド設定
- **homebrew-formula/mote.rb** - Homebrew Formula

## 🎉 まとめ

**必要な操作は3ステップだけ!**

1. Conventional Commitsでコミット
2. mainにプッシュ
3. release-pleaseが作成したPRをマージ

あとは全て自動で完了します！ 🚀

---

**次回以降のリリースは、「リリース手順（通常時）」セクションだけを参照すればOKです！**
