# PDF Compare

跨文件夹对比 PDF 图纸，找出漏发的文件并复制到指定输出目录。Windows / macOS 通用桌面应用，黑白极简界面。

## 用法

1. 选择「全部文件夹」（左侧文件源）
2. 选择「已发送文件夹」（右侧文件源）
3. 选择「输出文件夹」（缺失文件将被复制到此处）
4. 点击「开始对比」查看缺失清单与异常清单
5. 点击「复制缺失文件到输出」一键拷贝

## 对比规则

- 文件名形如 `40-XX-YYY--xxx.PDF`，对比 key = `40-XX-YYY`
- **缺失**：左侧存在、右侧不存在的 key
- **异常**：左侧存在但右侧连 `40-XX` 前缀都没有的（提示可能选错文件夹）
- 同 key 下多张 `_Sht_N`，右侧任一存在即视为已发送

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 测试

```bash
cd src-tauri/algo && cargo test
```

## 构建

```bash
pnpm tauri build
```

## 发布

打 tag 触发 GitHub Actions 自动构建并创建 Release：

```bash
git tag v0.1.0
git push --tags
```

Release 草稿出现在 GitHub → Releases，确认无误后点 Publish 即可。

## 技术栈

Tauri v2 · React 18 · TypeScript · Vite · Tailwind CSS · Rust（walkdir + regex）
