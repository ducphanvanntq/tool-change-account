# 🔄 Tool Change Account

CLI tool để quản lý và chuyển đổi tài khoản Antigravity.

## 🚀 One-Click Install

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/ducphanvanntq/tool-change-account/main/scripts/install.sh | sudo bash
```

### Windows (PowerShell Administrator)

```powershell
irm https://raw.githubusercontent.com/ducphanvanntq/tool-change-account/main/scripts/install.ps1 | iex
```

## 📦 Manual Install

Download binary từ [Releases](https://github.com/ducphanvanntq/tool-change-account/releases/latest):

| OS | File |
|----|------|
| macOS (Intel) | `tool-change-account-macos-x86_64.tar.gz` |
| macOS (Apple Silicon) | `tool-change-account-macos-aarch64.tar.gz` |
| Linux (x64) | `tool-change-account-linux-x86_64.tar.gz` |
| Windows (x64) | `tool-change-account-windows-x86_64.zip` |

## 🔧 Usage

```bash
# Xem thông tin account hiện tại
tool-change-account info

# Xem phiên bản
tool-change-account version

# Xem help
tool-change-account help
```

## ⚙️ Config

Tạo file `.env` cùng thư mục với binary:

```
CLIENT_ID=your_client_id
CLIENT_SECRET=your_client_secret
```

## 📄 License

MIT
