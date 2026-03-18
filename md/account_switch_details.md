# Account Switch - Chi tiết System Integration (Desktop Mode)

> **File nguồn chính**: `src-tauri/src/modules/integration.rs` → `DesktopIntegration::on_account_switch()`

Khi switch account ở chế độ Desktop, 4 action hệ thống được thực hiện tuần tự:

---

## 1. Get Storage Path — Lấy đường dẫn `storage.json`

> **File**: `src-tauri/src/modules/device.rs` → `get_storage_path()`

### Mục đích
Tìm file `storage.json` — nơi lưu trữ **device fingerprint** (telemetry) của Antigravity. File này quyết định "danh tính thiết bị" mà server nhìn thấy.

### Thuật toán tìm kiếm (theo thứ tự ưu tiên)

| # | Chiến lược | Đường dẫn |
|---|-----------|-----------|
| 1 | **`--user-data-dir` flag** | `{user-data-dir}/User/globalStorage/storage.json` |
| 2 | **Portable mode** | `{exe_parent}/data/user-data/User/globalStorage/storage.json` |
| 3 | **Standard install** | *(tùy OS, xem bên dưới)* |

#### Đường dẫn Standard theo OS

| OS | Đường dẫn |
|----|-----------|
| **macOS** | `~/Library/Application Support/Antigravity/User/globalStorage/storage.json` |
| **Windows** | `%APPDATA%\Antigravity\User\globalStorage\storage.json` |
| **Linux** | `~/.config/Antigravity/User/globalStorage/storage.json` |

### Chi tiết xác định `--user-data-dir`

Hàm `get_user_data_dir_from_process()` trong `process.rs`:
1. **Ưu tiên config**: Đọc `antigravity_args` từ `AppConfig` → tìm flag `--user-data-dir` hoặc `--user-data-dir=<path>`
2. **Fallback process args**: Nếu config không có → lấy command-line arguments từ process Antigravity đang chạy

### Chi tiết xác định Portable mode

Hàm `get_antigravity_executable_path()` trong `process.rs`:
1. Lấy path từ process đang chạy (ưu tiên cao nhất)
2. Nếu không → kiểm tra các vị trí cài đặt chuẩn:
   - **macOS**: `/Applications/Antigravity.app`
   - **Windows**: `%LOCALAPPDATA%\Programs\Antigravity\Antigravity.exe` → `%ProgramFiles%\...` → `%ProgramFiles(x86)%\...`
   - **Linux**: `/usr/bin/antigravity` → `/opt/Antigravity/antigravity` → `~/.local/bin/antigravity`

### Lỗi có thể xảy ra
- `"storage_json_not_found"` — Không tìm thấy `storage.json` ở bất kỳ vị trí nào → switch account sẽ **thất bại**

### Code

```rust
// device.rs:24-82
pub fn get_storage_path() -> Result<PathBuf, String> {
    // 1) --user-data-dir flag
    if let Some(user_data_dir) = process::get_user_data_dir_from_process() {
        let path = user_data_dir.join("User").join("globalStorage").join("storage.json");
        if path.exists() { return Ok(path); }
    }

    // 2) Portable mode
    if let Some(exe_path) = process::get_antigravity_executable_path() {
        if let Some(parent) = exe_path.parent() {
            let portable = parent.join("data").join("user-data")
                .join("User").join("globalStorage").join("storage.json");
            if portable.exists() { return Ok(portable); }
        }
    }

    // 3) Standard location (per-OS)
    // macOS: ~/Library/Application Support/Antigravity/User/globalStorage/storage.json
    // Windows: %APPDATA%\Antigravity\User\globalStorage\storage.json
    // Linux: ~/.config/Antigravity/User/globalStorage/storage.json

    Err("storage_json_not_found".to_string())
}
```

---

## 2. Kill Process — Đóng Antigravity process (timeout 20s)

> **File**: `src-tauri/src/modules/process.rs` → `close_antigravity(20)`

### Mục đích
Đóng **hoàn toàn** tất cả process của Antigravity trước khi ghi file, tránh xung đột file lock và đảm bảo dữ liệu mới được load khi khởi động lại.

### Điều kiện
Chỉ thực hiện khi `is_antigravity_running()` trả về `true`.

### Cách nhận diện process Antigravity

Hàm `get_antigravity_pids()` dùng `sysinfo` để quét tất cả process, với logic:

**Luôn loại trừ:**
- Process của chính Manager (so sánh PID + executable path)
- Helper processes: kiểm tra theo tên (`helper`, `plugin`, `renderer`, `gpu`, `crashpad`, `utility`, `audio`, `sandbox`) và args (`--type=`)

**Nhận diện chính:**

| # | Nguồn | Logic |
|---|-------|-------|
| 1 | **Manual config path** | So khớp `antigravity_executable` từ AppConfig (macOS: cùng `.app` bundle) |
| 2 | **Executable path** | macOS: `exe_path` chứa `antigravity.app` |
| 3 | **Process name** | Windows: `antigravity.exe`; Linux: name chứa `antigravity` và không chứa `tools` |

### Chiến lược đóng (2 pha)

```
Phase 1: Graceful Shutdown (SIGTERM / taskkill)
    ├── macOS:  kill -15 <main_pid>     ← chỉ gửi cho main process
    ├── Linux:  kill -15 <main_pid>     ← tương tự
    └── Windows: taskkill /F /PID <pid> ← cho từng PID

    → Chờ tối đa 70% timeout (14s) cho process tự tắt
    → Kiểm tra mỗi 500ms

Phase 2: Force Kill (SIGKILL) — nếu vẫn còn chạy
    ├── macOS:  kill -9 <all_remaining_pids>
    ├── Linux:  kill -9 <all_remaining_pids>  
    └── Windows: (đã dùng /F trong Phase 1)

    → Chờ thêm 1s
    → Kiểm tra lần cuối
```

### Xác định Main Process (macOS/Linux)

Quan trọng vì gửi SIGTERM cho main process sẽ tự giải phóng helper processes mà không gây popup **"Window terminated unexpectedly"**.

Thuật toán:
1. **Ưu tiên manual path match**: So `antigravity_executable` config với executable path của process, đảm bảo cùng `.app` bundle (macOS) và không phải Helper
2. **Fallback feature analysis**: Process không có `--type=` args và tên không chứa keyword Helper

### Linux: Family Process Tree Exclusion

`get_self_family_pids()` — xây dựng danh sách PID "gia đình" để tránh kill chính mình:
- **Ancestors**: BFS ngược lên parent (max 10 level)
- **Descendants**: BFS xuống children
- Tất cả PID trong family tree đều được loại khỏi danh sách target

### Lỗi có thể xảy ra
- `"Unable to close Antigravity process, please close manually and retry"` — Cả 2 phase đều thất bại

### Code tóm tắt

```rust
// process.rs:356-701
pub fn close_antigravity(timeout_secs: u64) -> Result<(), String> {
    // Windows: taskkill /F /PID cho từng PID
    // macOS/Linux:
    //   1. Xác định main_pid (manual path match → feature analysis)
    //   2. Phase 1: kill -15 main_pid → chờ 70% timeout
    //   3. Phase 2: kill -9 all remaining → chờ 1s
    //   4. Final check: is_antigravity_running()
}
```

---

## 3. Write Device Profile — Ghi device fingerprint vào `storage.json`

> **File**: `src-tauri/src/modules/device.rs` → `write_profile()`

### Mục đích
Ghi **device fingerprint** (4 trường telemetry) của account vào `storage.json`, giúp mỗi account có danh tính thiết bị riêng biệt khi kết nối server.

### DeviceProfile struct

```rust
// models/account.rs:147-152
pub struct DeviceProfile {
    pub machine_id: String,      // "auth0|user_<random_hex_32>"
    pub mac_machine_id: String,  // UUID v4 format
    pub dev_device_id: String,   // UUID v4
    pub sqm_id: String,          // "{UUID_V4_UPPERCASE}"
}
```

### Quy trình ghi

```
1. Đọc nội dung storage.json hiện tại → parse JSON
2. Ghi vào nested object `telemetry`:
   ├── telemetry.machineId     = profile.machine_id
   ├── telemetry.macMachineId  = profile.mac_machine_id
   ├── telemetry.devDeviceId   = profile.dev_device_id
   └── telemetry.sqmId         = profile.sqm_id

3. Ghi thêm flat keys (tương thích format cũ):
   ├── "telemetry.machineId"     = profile.machine_id
   ├── "telemetry.macMachineId"  = profile.mac_machine_id
   ├── "telemetry.devDeviceId"   = profile.dev_device_id
   └── "telemetry.sqmId"         = profile.sqm_id

4. Đồng bộ serviceMachineId:
   └── "storage.serviceMachineId" = profile.dev_device_id

5. Ghi file JSON pretty-printed → log thành công

6. Đồng bộ vào state.vscdb (SQLite):
   └── INSERT OR REPLACE INTO ItemTable 
       (key='storage.serviceMachineId', value=dev_device_id)
```

### Dual-format compatibility

File `storage.json` hỗ trợ 2 format:
- **Nested**: `{ "telemetry": { "machineId": "..." } }`
- **Flat**: `{ "telemetry.machineId": "..." }`

Hàm `write_profile()` ghi **cả hai** để tương thích với mọi phiên bản Antigravity.

### Đồng bộ `state.vscdb`

Hàm `sync_state_service_machine_id_value()`:
- Mở file `state.vscdb` (SQLite, cùng thư mục với `storage.json`)
- Tạo bảng `ItemTable` nếu chưa tồn tại
- `INSERT OR REPLACE` key `storage.serviceMachineId` = `dev_device_id`
- Nếu `state.vscdb` không tồn tại → log warning, bỏ qua (không lỗi)

### Ví dụ storage.json sau khi ghi

```json
{
  "telemetry": {
    "machineId": "auth0|user_a1b2c3d4e5f6...",
    "macMachineId": "f8a1b2c3-d4e5-4f6a-8b9c-0d1e2f3a4b5c",
    "devDeviceId": "12345678-abcd-efgh-ijkl-123456789012",
    "sqmId": "{ABCDEF12-3456-7890-ABCD-EF1234567890}"
  },
  "telemetry.machineId": "auth0|user_a1b2c3d4e5f6...",
  "telemetry.macMachineId": "f8a1b2c3-d4e5-4f6a-8b9c-0d1e2f3a4b5c",
  "telemetry.devDeviceId": "12345678-abcd-efgh-ijkl-123456789012",
  "telemetry.sqmId": "{ABCDEF12-3456-7890-ABCD-EF1234567890}",
  "storage.serviceMachineId": "12345678-abcd-efgh-ijkl-123456789012"
}
```

### Auto-generate profile (nếu chưa có)

Trong `switch_account()` (account.rs:973-985), nếu `account.device_profile` là `None`:

```rust
// device.rs:391-398
pub fn generate_profile() -> DeviceProfile {
    DeviceProfile {
        machine_id: format!("auth0|user_{}", random_hex(32)),
        mac_machine_id: new_standard_machine_id(),   // UUID v4 format
        dev_device_id: Uuid::new_v4().to_string(),
        sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
    }
}
```

### Lỗi có thể xảy ra
- `"storage_json_missing"` — File không tồn tại
- `"json_top_level_not_object"` — JSON root không phải object
- `"telemetry_not_object"` — Trường `telemetry` tồn tại nhưng không phải object

---

## 4. Backup + Inject DB — Backup database và inject token

> **File**: `src-tauri/src/modules/db.rs` → `get_db_path()` + `inject_token()`

### 4.1 Get DB Path — Tìm `state.vscdb`

Tìm file database `state.vscdb` (SQLite) theo thứ tự ưu tiên:

| # | Chiến lược | Đường dẫn |
|---|-----------|-----------|
| 1 | `--user-data-dir` flag | `{user-data-dir}/User/globalStorage/state.vscdb` |
| 2 | Portable mode | `{exe_parent}/data/user-data/User/globalStorage/state.vscdb` |
| 3 | Standard install | *(tùy OS)* |

#### Đường dẫn Standard theo OS

| OS | Đường dẫn |
|----|-----------|
| **macOS** | `~/Library/Application Support/Antigravity/User/globalStorage/state.vscdb` |
| **Windows** | `%APPDATA%\Antigravity\User\globalStorage\state.vscdb` |
| **Linux** | `~/.config/Antigravity/User/globalStorage/state.vscdb` |

### 4.2 Backup Database

Trong `integration.rs` (dòng 40-43), trước khi inject:

```rust
let db_path = db::get_db_path()?;
if db_path.exists() {
    let backup_path = db_path.with_extension("vscdb.backup");
    let _ = fs::copy(&db_path, &backup_path);
}
```

- Backup file: `state.vscdb.backup` (cùng thư mục)
- Backup **luôn ghi đè** file cũ (không giữ history)
- Lỗi copy bị **bỏ qua** (`let _ =`) — không block switch flow

### 4.3 Inject Token — Ghi token vào database

#### Parameters

| Param | Giá trị | Nguồn |
|-------|---------|-------|
| `access_token` | OAuth access token | `account.token.access_token` |
| `refresh_token` | OAuth refresh token | `account.token.refresh_token` |
| `expiry` | Timestamp hết hạn | `account.token.expiry_timestamp` |
| `email` | Email account | `account.email` |

#### Version Detection

Hàm `inject_token()` tự động phát hiện phiên bản Antigravity và chọn strategy phù hợp:

```
Detect Antigravity version
    ├── >= 1.16.5 → New format ONLY
    ├── < 1.16.5  → Old format ONLY
    └── Detection failed → Try BOTH formats (fallback)
```

#### New Format (>= 1.16.5)

**Database key**: `antigravityUnifiedStateSync.oauthToken`

**Cấu trúc dữ liệu** (Protobuf → Base64):

```
OuterMessage (base64 encoded)
└── field 1 (len_delim): InnerMessage
    ├── field 1 (string): "oauthTokenInfoSentinelKey"
    └── field 2 (len_delim): InnerMessage2
        └── field 1 (string): base64(OAuthTokenInfo)
                                └── Protobuf binary chứa:
                                    - access_token
                                    - refresh_token
                                    - expiry timestamp
```

**SQL thực thi:**
```sql
INSERT OR REPLACE INTO ItemTable (key, value) 
VALUES ('antigravityUnifiedStateSync.oauthToken', '<base64_protobuf_data>');

INSERT OR REPLACE INTO ItemTable (key, value) 
VALUES ('antigravityOnboarding', 'true');
```

#### Old Format (< 1.16.5)

**Database key**: `jetskiStateSync.agentManagerInitState`

**Quy trình:**
```
1. SELECT value FROM ItemTable WHERE key = 'jetskiStateSync.agentManagerInitState'
2. Base64 decode → protobuf binary
3. Xóa các field cũ:
   ├── Field 1: UserID
   ├── Field 2: Email
   └── Field 6: OAuthTokenInfo
4. Tạo field mới:
   ├── Email field (protobuf) ← account email
   └── OAuth field (protobuf) ← access_token + refresh_token + expiry
5. Merge: clean_data + new_email + new_oauth
6. Base64 encode → UPDATE database
```

> **Lưu ý**: Field 1 (UserID) **không** được inject lại — buộc client phải re-authenticate session với token mới.

**SQL thực thi:**
```sql
UPDATE ItemTable SET value = '<base64_protobuf_data>' 
WHERE key = 'jetskiStateSync.agentManagerInitState';

INSERT OR REPLACE INTO ItemTable (key, value) 
VALUES ('antigravityOnboarding', 'true');
```

#### Onboarding Flag

Cả hai format đều inject thêm:
```sql
INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('antigravityOnboarding', 'true');
```
Đảm bảo Antigravity không hiện onboarding screen sau khi switch account.

### Lỗi có thể xảy ra

| Lỗi | Nguyên nhân |
|-----|-------------|
| `"Failed to open database"` | File `state.vscdb` bị lock hoặc corrupt |
| `"Old format key does not exist"` | Version mới nhưng dùng old format strategy |
| `"Base64 decoding failed"` | Data trong DB bị corrupt |
| `"Both formats failed"` | Không detect được version và cả 2 format đều lỗi |

---

## Tổng kết Flow trong `DesktopIntegration::on_account_switch()`

```rust
// integration.rs:22-59
async fn on_account_switch(&self, account: &Account) -> Result<(), String> {
    // 1. Get storage path
    let storage_path = device::get_storage_path()?;

    // 2. Kill Antigravity process
    if process::is_antigravity_running() {
        process::close_antigravity(20)?;
    }

    // 3. Write Device Profile → storage.json
    if let Some(ref profile) = account.device_profile {
        device::write_profile(&storage_path, profile)?;
    }

    // 4. Backup + Inject DB
    let db_path = db::get_db_path()?;
    if db_path.exists() {
        let backup_path = db_path.with_extension("vscdb.backup");
        let _ = fs::copy(&db_path, &backup_path);
    }
    db::inject_token(
        &db_path,
        &account.token.access_token,
        &account.token.refresh_token,
        account.token.expiry_timestamp,
        &account.email,
    )?;

    // 5. Restart Antigravity
    process::start_antigravity()?;

    // 6. Update system tray
    let _ = tray::update_tray_menus(&self.app_handle);

    Ok(())
}
```

> **Ghi chú**: Nếu bất kỳ bước nào từ 1-4 fail (trả về `Err`), toàn bộ switch account sẽ **dừng lại** và không thực hiện các bước còn lại (bước 5, 6). Antigravity sẽ **không** được khởi động lại.
