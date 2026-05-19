# x-cli-rs

简体中文 | [English](README.md)

`x-cli-rs` 是一组基于 Rust 的浏览器自动化命令行工具，灵感来自 [`better-world-ai/x-cli`](https://github.com/better-world-ai/x-cli)。

它围绕 `kimi-webbridge` 设计：通过本地 WebBridge 服务驱动用户真实 Chrome 会话，让 CLI 能够自动化已经登录的网站页面，而不需要 API Key、浏览器 Cookie 或额外服务 Token。

## 项目亮点

- **统一入口 `x`**：通过一个命令访问 ChatGPT Images、Google Search、Baidu Search 和 Gemini Nano Banana 图像生成。
- **兼容入口**：同时保留 `chatgpt-image-cli`、`google-cli`、`baidu-cli`、`nanobanana-cli` 等独立二进制。
- **稳定 JSON 输出**：标准输出面向 Agent、脚本和自动化流水线，便于机器解析。
- **可复用 Rust crate**：每个浏览器工作流都拆成独立库 crate，方便复用和测试。
- **真实浏览器自动化**：复用用户 Chrome Profile 中的登录态，适合需要网页登录态的场景。
- **跨平台发布**：Release 产物覆盖 macOS、Linux 和 Windows，并提供安装脚本。

## 适用场景

`x-cli-rs` 适合下列场景：

- 在脚本或 Agent 工作流里调用网页能力。
- 复用真实浏览器登录态，而不是维护 API 凭证。
- 将搜索、图片生成等网页操作封装成稳定 JSON CLI。
- 在 Rust 项目中复用底层浏览器自动化流程。
- 对原始 `x-cli` 示例做 Rust 化、类型化和可发布化实现。

## 前置要求

运行任何命令前，请确认：

1. 已启动 `kimi-webbridge` 兼容的本地 daemon，默认地址为 `http://127.0.0.1:10086`。
2. Chrome WebBridge 扩展已连接。
3. 目标网站已在该 Chrome Profile 中登录。
4. 命令运行期间保持 Chrome 打开。

默认桥接地址可通过环境变量覆盖：

```bash
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 x google search "rust cli"
```

常见问题判断：

- 如果返回 daemon unreachable，通常是本地 WebBridge 服务未启动或端口不一致。
- 如果返回 extension not connected，通常是 Chrome 扩展没有连接到 daemon。
- 如果页面停留在登录页，请先在 Chrome 中手动登录目标网站。

## 安装

### macOS / Linux

安装最新版本：

```bash
curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

没有 `curl` 时使用 `wget`：

```bash
wget -qO- https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

安装指定版本：

```bash
XCLI_RS_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

安装到自定义目录：

```bash
XCLI_RS_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

### Windows PowerShell

```powershell
iwr https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.ps1 -UseB | iex
```

安装脚本会下载当前平台对应的 Release zip，校验 `.sha256`，并安装以下二进制：

```text
x
chatgpt-image-cli
google-cli
baidu-cli
nanobanana-cli
```

## 快速开始

### ChatGPT Images 生成图片

```bash
x chatgpt-image generate "a cute panda riding a bicycle" -o ./images
```

也可以使用别名：

```bash
x image g "a cat in a space suit" --timeout 180
x img gen "夕阳下的富士山" -o ./images
```

### Google 搜索

```bash
x google search "rust cli" --limit 10 --hl en
```

### 百度搜索

```bash
x baidu search "大模型" --limit 10
x baidu search "天气 北京" -n 20 --all
```

`--all` 会尽量保留页面上解析到的全部类型结果；不加时保留默认过滤策略。

### Gemini Nano Banana 图片生成

```bash
x nanobanana gen "画一朵粉色月季花，微距特写" -o ./out
```

也可以使用短别名：

```bash
x nano gen "generate an image of a tiny robot in a garden" --thumb-width 320 --timeout 300
x banana gen "画一个赛博朋克风格的茶杯" -o ./out
```

## 统一入口与兼容入口

### 统一入口 `x`

| 功能 | 命令 |
| --- | --- |
| ChatGPT 图片生成 | `x chatgpt-image generate "prompt" -o ./images` |
| ChatGPT 图片别名 | `x image g "prompt"`、`x img gen "prompt"` |
| Google 搜索 | `x google search "rust cli" --limit 10 --hl en` |
| 百度搜索 | `x baidu search "大模型" --limit 10` |
| 百度搜索保留更多类型 | `x baidu search "天气 北京" -n 20 --all` |
| Gemini Nano Banana | `x nanobanana gen "prompt" -o ./out` |
| Nano Banana 别名 | `x nano gen "prompt"`、`x banana gen "prompt"` |

### 独立兼容入口

这些命令适合保持与原独立 CLI 类似的调用方式：

```bash
chatgpt-image-cli generate "a cute panda riding a bicycle" -o ./images
google-cli search "rust cli" --limit 10 --hl en
baidu-cli search "大模型" --limit 10
baidu-cli search "天气 北京" -n 20 --all
nanobanana-cli gen "画一朵粉色月季花，微距特写" -o ./out
```

统一入口和兼容入口调用的是同一套底层库流程。

## JSON 输出约定

所有成功输出都使用稳定 envelope：

```json
{
  "ok": true,
  "data": {}
}
```

所有错误输出也使用稳定 envelope，并返回非 0 退出码：

```json
{
  "ok": false,
  "error": {
    "code": "missing_args",
    "message": "..."
  }
}
```

这种约定适合被 Shell、Node.js、Python、Agent 框架等上层系统消费。

## 输出示例

### ChatGPT 图片生成

```json
{
  "ok": true,
  "data": {
    "prompt": "a cute panda riding a bicycle",
    "path": "/absolute/path/to/chatgpt-20260518-120000.png",
    "bytes": 2228437,
    "caption": "...",
    "conversation_url": "https://chatgpt.com/c/...",
    "elapsed_ms": 59970
  }
}
```

### Google 搜索

```json
{
  "ok": true,
  "data": [
    {
      "title": "...",
      "url": "https://example.com",
      "snippet": "..."
    }
  ]
}
```

### 百度搜索

```json
{
  "ok": true,
  "data": {
    "query": "大模型",
    "count": 1,
    "results": [
      {
        "rank": 1,
        "id": "...",
        "tpl": "www_index",
        "title": "...",
        "url": "https://example.com",
        "abstract": "...",
        "source": "..."
      }
    ]
  }
}
```

### Gemini Nano Banana

```json
{
  "ok": true,
  "data": {
    "prompt": "画一朵粉色月季花，微距特写",
    "full": "/abs/path/out/20260518-120000-full.png",
    "thumb": "/abs/path/out/20260518-120000-thumb.png",
    "width": 2816,
    "height": 1536,
    "thumb_width": 256,
    "elapsed_ms": 184230
  }
}
```

## 调试

使用 `--verbose` 将流程日志输出到 stderr，同时保持 stdout 为机器可读 JSON：

```bash
x --verbose chatgpt-image generate "hello" -o ./images
x --verbose google search "rust cli"
x --verbose baidu search "大模型"
x --verbose nanobanana gen "画一朵粉色月季花" -o ./out
```

兼容入口同样支持 `--verbose`：

```bash
chatgpt-image-cli --verbose generate "hello" -o ./images
google-cli --verbose search "rust cli"
baidu-cli --verbose search "大模型"
nanobanana-cli --verbose gen "画一朵粉色月季花" -o ./out
```

使用 `RUST_LOG` 控制日志级别：

```bash
RUST_LOG=debug x --verbose chatgpt-image generate "hello"
```

ChatGPT 图片生成的典型 verbose 流程：

```text
status -> navigate -> input -> submit -> wait_url -> wait_image -> read_image_meta -> download_image -> write_file
```

Google 搜索页面 DOM 和 consent 行为记录在 [Google Search DOM Archaeology](docs/google-archaeology.md)。

## 工作区结构

```text
crates/
  xcli/                顶层 `x` CLI 入口
  xcli-core/           共享错误、配置和通用工具
  xcli-output/         稳定 JSON 响应和错误输出
  xcli-webbridge/      kimi-webbridge 兼容 daemon 的 HTTP 客户端
  xcli-browser/        基于 bridge 的浏览器动作抽象
  xcli-chatgpt-image/  可复用 ChatGPT 图片生成流程
  xcli-google/         可复用 Google 搜索流程
  xcli-baidu/          可复用百度搜索流程
  xcli-nanobanana/     可复用 Gemini Nano Banana 图片流程
examples/
  chatgpt-image-cli/   兼容原始 CLI 形态的 ChatGPT 图片命令
  google-cli/          Google 搜索兼容命令
  baidu-cli/           百度搜索兼容命令
  nanobanana-cli/      Gemini Nano Banana 兼容命令
```

## 开发

常用本地流程建议使用 Makefile：

```bash
make lock
make check
make build
make verify
```

等价 Cargo 命令：

```bash
cargo generate-lockfile
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p xcli -p chatgpt-image-cli -p google-cli -p baidu-cli -p nanobanana-cli
```

真实 WebBridge 冒烟测试：

```bash
make run-image
make run-google
make run-baidu
make run-nanobanana
```

更多贡献要求、Cargo.lock 策略、PR 检查清单和发布预期见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 发布

发布前请完成 [release checklist](docs/release-checklist.md)。

Release workflow 会构建以下二进制：

```text
x
chatgpt-image-cli
google-cli
baidu-cli
nanobanana-cli
```

每个平台会生成一个 zip：

```text
x-cli-rs-x86_64-unknown-linux-gnu.zip
x-cli-rs-aarch64-apple-darwin.zip
x-cli-rs-x86_64-apple-darwin.zip
x-cli-rs-x86_64-pc-windows-msvc.zip
```

每个 zip 都有对应 SHA256 文件：

```text
x-cli-rs-x86_64-unknown-linux-gnu.zip.sha256
```

推送版本 tag 触发正式发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

也可以在 GitHub Actions 页面通过 `workflow_dispatch` 手动运行 Release workflow。

## 设计原则

- 命令参数稳定。
- stdout JSON 稳定。
- 错误码稳定。
- 退出码稳定。
- 浏览器流程库可复用。
- 常见桌面和服务器平台均提供 Release 产物。

## 状态

当前仓库处于快速迭代阶段，已经具备：

- 统一 `x` 入口。
- `chatgpt-image-cli`、`google-cli`、`baidu-cli`、`nanobanana-cli` 兼容入口。
- 共享 JSON 输出工具。
- `kimi-webbridge` 协议客户端。
- ChatGPT 图片、Google 搜索、百度搜索、Nano Banana 流程的 mock 测试。
- 用于真实浏览器调试的 verbose tracing。
- Release 打包和安装脚本。
