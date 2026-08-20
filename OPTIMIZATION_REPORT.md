# 📊 گزارش تحلیل فنی و نقشه راه بهینه‌سازی — Win11 Clipboard History for Linux

**پروژه:** Tauri 2 + React 19 + TypeScript + Rust (نسخه 0.7.1)
**تاریخ تحلیل:** ۲۰۲۶-۰۸-۲۰
**حوزه تحلیل:** بک‌اند Rust (~۷,۸۶۵ خط در ۱۵ ماژول)، فرانت‌اند React، CI/CD و بسته‌بندی لینوکس

---

## 📋 ۱. خلاصه وضعیت و پتانسیل‌های ارتقاء

پروژه از نظر ساختار، یک اپلیکیشن دسکتاپ Tauri خوش‌فرم با جداسازی منطقی ماژول‌ها (clipboard_manager، input_simulator، paste_sync و…) است و لایه‌ی ورود کلید (uinput/XTest) با دقت بالایی طراحی شده. اما **لایه‌ی ذخیره‌سازی و انتقال داده (Data Layer) نقطه‌ی بحرانی معماری** است: کل تاریخچه به‌صورت یک `Vec` در حافظه و یک فایل JSON غول‌پیکر (`history.json`) نگهداری می‌شود که **تصاویر را به‌صورت base64 داخل خود JSON ذخیره می‌کند** — با ۵۰ تصویر، فایل به‌راحتی می‌تواند چند صد مگابایت شود و هر بار کپی جدید، کل فایل را دوباره سریالایز و روی دیسک می‌نویسد. مهم‌ترین فرصت‌های بهینه‌سازی:

- **🔴 جایگزینی ذخیره‌سازی JSON تمام‌در-حافظه با SQLite + فایل‌های تصویر مجزا** — بزرگ‌ترین برد معماری ممکن (حذف O(n²)، حذف base64 از IPC، حذف رایت‌های سنکرون کل فایل).
- **🔴 Watcher هر ۵۰۰ms چندین اتصال جدید X11 Clipboard می‌سازد** (`Clipboard::new()` در `get_current_text`، `get_current_html` و `get_current_image`) — هر تیک چرخه‌ی نظارت، ۲ تا ۳ بار اتصال X11 جدید برقرار می‌کند و قفل سراسری را هم هنگام I/O نگه می‌دارد.
- **🟡 امنیت پیکربندی:** `"csp": null`، `withGlobalTauri: true`، کلید API تنور هاردکد در باندل فرانت‌اند و دانلود GIF بدون سقف حجم (SSRF/پر کردن دیسک).
- **🟡 فرانت‌اند:** لیست تاریخچه بدون ویرچوالایزیشن رندر می‌شود (تا ۱۰۰,۰۰۰ آیتم!) و در هر رویداد `clipboard-changed` کل تاریخچه از بک‌اند دوباره fetch می‌شود.

---

## 🚀 ۲. نقشه راه بهینه‌سازی عملی (Actionable Optimization Blueprint)

### الف) معماری و ساختار کد (Architecture & Refactoring)

---

#### ⚠️ الف-۱: ذخیره‌سازی تمام-در-حافظه/JSON با تصاویر base64 (بحرانی‌ترین مشکل پروژه)

**مشکل:**
در `src-tauri/src/clipboard_manager.rs` کل تاریخچه یک `Vec<ClipboardItem>` است که:
- در `save_history()` با `serde_json::to_string_pretty(&self.history)` روی **هر تغییر** (افزودن، حذف، پین، جابه‌جایی) دوباره سریالایز و با `fs::write` روی دیسک نوشته می‌شود.
- تصاویر در `ClipboardContent::Image { base64, .. }` به‌صورت **base64 کامل‌رزولوشن PNG** داخل JSON ذخیره می‌شوند (`convert_image_to_base64`). یک اسکرین‌شات ۴K تبدیل‌شده به PNG می‌تواند ۵–۲۰MB و معادل base64 آن ۷–۲۷MB باشد؛ با سقف پیش‌فرض ۵۰ آیتم، فایل تاریخچه به‌آسانی چند صد مگابایت می‌شود.
- در `get_history()` کل Vec با `self.history.clone()` کپی می‌شود (کپی رشته‌های base64 سنگین) و **کل آن هر بار از طریق IPC به فرانت‌اند ارسال می‌شود** (در هر رویداد `history-sync`/`clipboard-changed`).

**راهکار عملی (گام‌به‌گام):**
1. **انتقال به SQLite** (کریت `rusqlite` با `bundled` feature — بدون وابستگی سیستمی): یک جدول `items` با ایندکس روی `timestamp`، `pinned` و `hash`.
2. **تصاویر را فایل کن** (در `~/.local/share/win11-clipboard-history/images/{id}.png`) و فقط `path` + `width` + `height` را در دیتابیس نگه دار؛ **یک نسخه‌ی تامبنیل (مثلاً حداکثر ۵۱۲px) برای نمایش در UI** بساز.
3. **رایت‌ها را debounce/بچ کن**: هر تغییر، فقط به حافظه/SQLite اعمال شود و flush دوره‌ای یا در لحظه‌ی خروج انجام گیرد.

**نمونه کد (طرح اسکیمای SQLite + ذخیره‌ی تصویر به‌جای base64):**

```sql
-- migration.sql (اجرا با rusqlite::Connection::execute_batch)
CREATE TABLE IF NOT EXISTS items (
    id          TEXT PRIMARY KEY,           -- UUID
    kind        TEXT NOT NULL,              -- 'text' | 'richtext' | 'image'
    text        TEXT,                       -- متن یا plain متن RichText
    html        TEXT,                       -- فقط برای RichText
    image_path  TEXT,                       -- فقط برای image: مسیر فایل PNG
    image_hash  INTEGER,                    -- هش پایدار FNV برای حذف تکراری
    width       INTEGER,
    height      INTEGER,
    preview     TEXT NOT NULL,
    pinned      INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL            -- unix millis (قابل ایندکس)
);
CREATE INDEX IF NOT EXISTS idx_items_created ON items(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_items_pinned ON items(pinned);
```

```rust
// در clipboard_manager.rs — جایگزین convert_image_to_base64
const MAX_STORED_IMAGE_DIM: u32 = 512; // تامبنیل برای UI؛ فایل اصلی لازم نیست
const IMAGES_DIR: &str = "images";

fn store_image(&self, id: &str, image: &DynamicImage) -> Result<String, String> {
    let dir = self.persistence_path.parent().unwrap().join(IMAGES_DIR);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // 1. Downscale به جای نگه‌داشتن تصویر کامل در JSON
    let thumb = image.thumbnail(MAX_STORED_IMAGE_DIM, MAX_STORED_IMAGE_DIM);
    let path = dir.join(format!("{}.png", id));

    // 2. رایت اتمیک: بنویس در فایل موقت بعد rename کن (جلوگیری از خرابی در crash)
    let tmp = path.with_extension("png.tmp");
    thumb.save_with_format(&tmp, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}
```

> **استدلال فنی:** حذف base64 از JSON هم حجم دیسک، هم پهنای‌باند IPC (که روی هر رویداد همگام‌سازی کل تاریخچه را می‌فرستد)، هم حافظه‌ی watcher (که تصویر کامل را برای هش هر ۵۰۰ms می‌خواند) را به‌طور ریشه‌ای کاهش می‌دهد؛ SQLite با WAL رایت‌های جزئی و تراکنشی را جایگزین بازنویسی کل فایل می‌کند.

---

#### ⚠️ الف-۲: عملیات‌های O(n) و O(n²) در ساختار Vec

**مشکل:** در `clipboard_manager.rs`:
- `remove_duplicate_text_from_history` و `is_duplicate_text` هر بار کل لیست را خطی اسکن می‌کنند (در هر افزودن).
- `enforce_history_limit` برای هر آیتم اضافه، `self.history.remove(pos)` صدا می‌زند که هر بار O(n) جابه‌جایی حافظه است → در حالت بد (۱۰۰k آیتم) **O(n²)**.
- `insert_item` با `Vec::insert` در میانه‌ی لیست، بقیه‌ی عناصر را جابه‌جا می‌کند.

**راهکار عملی:**
1. یک `HashSet<u64>` از هش‌های متن (پایدار FNV) را **افزایشی** نگه دار تا تشخیص تکراری O(1) شود.
2. چون invariant «پین‌ها همیشه ابتدای لیست‌اند» برقرار است، `enforce_history_limit` را با `truncate` به‌جای حلقه‌ی remove بازنویسی کن.

**نمونه کد:**

```rust
pub struct ClipboardManager {
    history: Vec<ClipboardItem>,
    text_hashes: std::collections::HashSet<u64>, // شاخص افزایشی
    // ...
}

fn enforce_history_limit(&mut self) -> bool {
    let before = self.history.len();
    if self.history.len() <= self.max_history_size {
        return false;
    }
    // invariant: پین‌ها ابتدای لیست‌اند → truncate کافی است و O(n) است.
    // اگر همه پین شده‌اند، چیزی را حذف نکن (مطابق رفتار فعلی).
    if self.history.iter().any(|i| !i.pinned) {
        self.history.truncate(self.max_history_size);
        self.rebuild_hash_index(); // یک‌بار بازسازی، نه هر بار
    }
    self.history.len() != before
}

fn rebuild_hash_index(&mut self) {
    self.text_hashes.clear();
    for item in &self.history {
        if let Some(text) = item.plain_text() {
            self.text_hashes.insert(calculate_hash(text));
        }
    }
}
```

> **استدلال فنی:** تشخیص تکراری از O(n) به O(1) و trim از O(n²) به O(n) می‌رسد؛ در سقف ۱۰۰,۰۰۰ آیتم این تفاوت یعنی میلی‌ثانیه در برابر چند ثانیه‌ی انجماد UI (چون watcher قفل را حین این عملیات در دست دارد).

---

#### ⚠️ الف-۳: God Object در `main.rs` (۱,۱۱۵ خط) و کد مرده

**مشکل:** `main.rs` همزمان مسئول راه‌اندازی، کنترل پنجره (WindowController)، کنترل پنجره‌ی تنظیمات (SettingsController)، watcher، منوی tray و مدیریت رویدادهاست. علاوه بر آن `handle_window_moved_for_wayland` عملاً **کد مرده** است (بدنه‌ی آن کامنت شده) و در `config_manager.rs::resolve_window_position` کل منطق بازیابی موقعیت (بخش بزرگ) کامنت شده و در عمل همیشه bottom-center می‌گذارد.

**راهکار عملی:**
- `WindowController` و `SettingsController` را به ماژول‌های جدا (`window_controller.rs`) منتقل کن.
- کد مرده را حذف کن یا با feature-flag برگردان — کامنت‌های بزرگ باعث گمراهی نگهدارنده‌ها می‌شوند.
- `handle_window_moved_for_wayland` یا واقعاً موقعیت را ذخیره کند (بدنه‌ی کامنت‌شده را فعال کن) یا حذف شود.

**نمونه کد (حذف/فعال‌سازی کد مرده):**

```rust
// config_manager.rs — فعال‌سازی بازیابی موقعیت ذخیره‌شده (جایگزین بدنه‌ی کامنت‌شده)
pub fn resolve_window_position(
    state: &WindowState,
    available_monitors: &[Monitor],
    window_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    if let Some(saved_monitor) = &state.monitor_name {
        if let Some(monitor) = available_monitors.iter().find(|m| {
            m.name().is_some_and(|n| n.as_str() == saved_monitor)
        }) {
            if is_position_valid(state.x, state.y, monitor, window_size) {
                return PhysicalPosition::new(state.x, state.y);
            }
        }
    }
    // fallback به bottom-center (کد فعلی)
    let target = available_monitors.iter().find(|m| m.scale_factor() > 0.0)
        .unwrap_or(&available_monitors[0]);
    calculate_bottom_center(target, window_size)
}
```

---

#### ⚠️ الف-۴: تکرار کد تنظیم کلیپ‌بورد در سه ماژول (DRY)

**مشکل:** منطق «تنظیم کلیپ‌بورد با fallback خارجی (xclip/wl-copy) → arboard» سه بار پیاده شده: `set_text_robust`/`set_html_robust`/`set_image_robust` در `clipboard_manager.rs` و `ClipboardHandler` در `gif_manager.rs`. این سه نسخه در رفتار (مثلاً تایم‌اوت و verification) با هم اختلاف دارند.

**راهکار عملی:** یک ماژول `clipboard_io.rs` بساز که تنها نقطه‌ی تماس با سیستم کلیپ‌بورد باشد و هر سه نوع داده (text/html/image/uri-list) را با یک استراتژی مشترک (xclip → wl-copy → arboard) مدیریت کند.

**نمونه کد (ساختار ماژول واحد):**

```rust
// src-tauri/src/clipboard_io.rs
pub enum Payload<'a> {
    Text(&'a str),
    Html { html: &'a str, plain: &'a str },
    Bytes { mime: &'a str, data: &'a [u8] },
}

pub fn set_clipboard(payload: Payload<'_>) -> Result<(), ClipboardIoError> {
    // 1) تلاش با ابزار خارجی (Wayland → wl-copy، X11 → xclip)
    // 2) fallback به arboard با verification
    // 3) خطاهای یکسان با این enum
}
```

> **استدلال فنی:** تکرار منطق حساس (تایم‌اوت، verification، مدیریت فرایندهای child) در چند جا یعنی هر باگ فقط در یک مسیر رفع می‌شود؛ تک‌نقطه‌ای‌سازی (Single Responsibility) احتمال رگرسیون را کم می‌کند.

---

### ب) کارایی و سرعت (Performance Tuning)

---

#### ⚠️ ب-۱: Watcher هر ۵۰۰ms اتصال‌های X11 جدید می‌سازد و قفل را حین I/O نگه می‌دارد

**مشکل:** در `main.rs::start_clipboard_watcher`، هر تیک چرخه (۵۰۰ms):
- `manager.get_current_text()` → `Clipboard::new()?.get_text()` (خط ۳۰۴ `clipboard_manager.rs`) — در X11 یعنی یک اتصال X جدید + یک thread پس‌زمینه‌ی arboard.
- سپس `manager.get_current_html()` → `Clipboard::new()` دوباره (خط ۳۱۶).
- سپس `manager.get_current_image()` → `Clipboard::new()` بار سوم.

یعنی **هر ثانیه ۴ تا ۶ اتصال X11 جدید** و علاوه بر آن، کل این مدت قفل `parking_lot::Mutex` روی `ClipboardManager` گرفته شده — پس `paste_item` که به همان قفل نیاز دارد، تا پایان خواندن/هش/انکود تصویر (که می‌تواند ده‌ها میلی‌ثانیه طول بکشد) بلوک می‌شود.

**راهکار عملی:**
1. یک instance از `arboard::Clipboard` را در watcher **بازیافت** کن (مستندات arboard: ساخت Clipboard گران است؛ اتصال X دوباره استفاده شود).
2. قفل را فقط برای عملیات‌های درون‌حافظه‌ای نگه دار؛ خواندن کلیپ‌بورد و انکود تصویر را خارج از قفل انجام بده (مثلاً snapshot کردن state قبل از lock).
3. روی X11 به‌جای polling ثابت، از رویداد `PropertyNotify` روی `CLIPBOARD` selection استفاده کن تا فقط هنگام تغییر واقعی بیدار شوی (هرچند arboard این را نمی‌دهد، می‌توان با x11rb مستقیم در یک thread جدا polling سبک‌تر کرد؛ یا حداقل interval را وقتی تغییری نبوده adaptive کن).

**نمونه کد (بازیافت Clipboard در watcher):**

```rust
fn start_clipboard_watcher(app: AppHandle, clipboard_manager: Arc<Mutex<ClipboardManager>>) {
    std::thread::spawn(move || {
        // یک instance سراسری — نه یکی در هر تیک!
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => { eprintln!("[Watcher] Clipboard init failed: {e}"); return; }
        };
        let mut last_text_hash: Option<u64> = None;
        let mut last_image_hash: Option<u64> = None;
        let mut cleanup_counter = 0u32;

        loop {
            std::thread::sleep(Duration::from_millis(500));
            cleanup_counter += 1;

            // ۱) خواندن و هش — خارج از قفل manager
            let (text, html) = (|| -> Result<(String, Option<String>), arboard::Error> {
                let text = clipboard.get_text()?;
                let html = clipboard.get().html().ok();
                Ok((text, html))
            })();
            let image = (|| -> Result<Option<arboard::ImageData<'static>>, arboard::Error> {
                clipboard.get_image().map(|img| {
                    arboard::ImageData {
                        width: img.width, height: img.height,
                        bytes: img.bytes.into_owned().into(),
                    }
                })
            })();

            // ۲) فقط اینجا قفل کوتاه بگیر
            let mut manager = clipboard_manager.lock();
            if let Ok((text, html)) = text { /* ... add_text ... */ }
            if let Ok(Some(img)) = image { /* ... add_image ... */ }
            drop(manager);
        }
    });
}
```

> **استدلال فنی:** حذف ساخت/تخریب اتصال X11 در هر تیک، هزینه‌ی ثابت هر پالس نظارت را از چند میلی‌ثانیه (اتصال + handshake) به زیر میکروثانیه می‌رساند و کوتاه‌کردن پنجره‌ی قفل، تأخیر paste را که مستقیماً حس کاربر است کاهش می‌دهد.

---

#### ⚠️ ب-۲: `save_history()` سنکرون، روی هر تغییر، با `fs::write` غیراتمی

**مشکل:** `save_history()` از `insert_item`، `remove_item`، `toggle_pin`، `move_item_to_top`، `clear` و… صدا زده می‌شود؛ هر بار کل تاریخچه سریالایز و **سنکرون روی همان thread** نوشته می‌شود (watcher حین نگه‌داشتن قفل). با تاریخچه‌ی چند ده‌مگابایتی، هر کپی → میلی‌ثانیه‌ها توقف.

**راهکار عملی:**
- رایت را به یک thread پس‌زمینه بده (یا debounce با `tokio::time::interval`).
- رایت اتمیک: بنویس در `history.json.tmp` سپس `fs::rename` — اگر برنامه وسط رایت crash کند فایل اصلی خراب نمی‌شود.

**نمونه کد (رایت اتمیک + دیباونس):**

```rust
fn save_history_atomic(&self) {
    let path = &self.persistence_path;
    let tmp = path.with_extension("json.tmp");
    let content = match serde_json::to_string_pretty(&self.history) { /* ... */ };
    let result = (|| -> std::io::Result<()> {
        fs::write(&tmp, content)?;
        fs::rename(&tmp, path) // atomic on same filesystem (POSIX)
    })();
    if let Err(e) = result { eprintln!("Failed to save history: {e}"); }
}
```

```rust
// در main.rs — ذخیره‌سازی ناهمگام خارج از قفل
fn spawn_debounced_saver(state: Arc<Mutex<ClipboardManager>>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            let manager = state.lock();
            if manager.is_dirty() {           // پرچم dirty فقط در صورت تغییر
                manager.save_history_atomic(); // هنوز سنکرون، ولی دیباونس‌شده
            }
        }
    });
}
```

> **استدلال فنی:** دیباونس، تعداد رایت‌های دیسک را در کپی‌های سریع (مثلاً کپی ۱۰ متن در ۲ ثانیه) از ۱۰ به ۱–۲ کاهش می‌دهد و rename اتمیک، احتمال خرابی `history.json` (که الان با `fs::write` ممکن است وسط‌رایت با crash خراب شود) را تقریباً صفر می‌کند.

---

#### ⚠️ ب-۳: GIF دوباره دانلود می‌شود و کش از هش ناپایدار استفاده می‌کند

**مشکل:** در `gif_manager.rs`:
- `download_gif_to_file` هر بار فایل را از نو دانلود می‌کند (کامنت داخل کد هم تأیید می‌کند: «maintain overwrite»).
- نام فایل کش با `DefaultHasher` ساخته می‌شود (خط `url.hash(&mut hasher)`) — در حالی که `clipboard_manager.rs` دقیقاً برای «پایداری بین restartها» هش FNV را دستی پیاده کرده. `DefaultHasher` بین نسخه‌های Rust/اجراها ثابت نیست → کش در هر آپدیت بی‌اثر می‌شود.

**راهکار عملی:**
- از `calculate_hash` پایدار (همان FNV موجود در `clipboard_manager`) برای نام فایل کش استفاده کن.
- دانلود را با بررسی `Last-Modified`/TTL (مثلاً ۲۴ ساعت) و «اگر فایل موجود است و کوچک‌تر از آستانه، استفاده کن» بهینه کن.

**نمونه کد:**

```rust
// gif_manager.rs — جایگزینی DefaultHasher با FNV پایدار
fn get_path_for_url(url: &str) -> Result<PathBuf, String> {
    // از همان تابع پایدار clipboard_manager استفاده کن (یا به یک util مشترک منتقل کن)
    let hash = crate::clipboard_manager::calculate_hash(&url);
    Ok(Self::get_dir()?.join(format!("{hash:016x}.gif")))
}

// دانلود مشروط: فقط اگر کش قدیمی‌تر از TTL باشد
fn download(url: &str, destination: &Path) -> Result<(), String> {
    const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
    if let Ok(meta) = fs::metadata(destination) {
        if meta.len() > 0
            && meta.modified().ok().is_some_and(|m| m.elapsed().unwrap_or_default() < CACHE_TTL)
        {
            return Ok(()); // cache hit
        }
    }
    // ... دانلود واقعی ...
}
```

> **استدلال فنی:** هش پایدار یعنی همان URL همیشه به همان فایل نگاشت می‌شود (هدف اصلی کش) و TTL جلوی ارائه‌ی GIFهای منقضی‌شده را می‌گیرد — این ترکیب هم تعداد درخواست‌های شبکه را کم می‌کند هم صحت محتوا را حفظ می‌کند.

---

### ج) امنیت و پایداری (Security & Reliability)

---

#### ⚠️ ج-۱: `"csp": null` و `withGlobalTauri: true`

**مشکل:** در `src-tauri/tauri.conf.json` مقدار `security.csp` برابر `null` است یعنی **هیچ Content-Security-Policy نداریم**؛ و `app.withGlobalTauri: true` کل API تائوری را به `window.__TAURI__` در معرض هر اسکریپت تزریق‌شده قرار می‌دهد. اگر روزی متنی از کلیپ‌بورد (که کاملاً کنترل کاربر است) به‌صورت HTML رندر شود یا XSS در فرانت‌اند رخ دهد، مهاجم به `invoke` دسترسی کامل دارد (حذف تاریخچه، پین کردن، باز کردن URL دلخواه با shell و…).

**راهکار عملی:**
- یک CSP سخت‌گیرانه تنظیم کن (`default-src 'self'; connect-src 'self' ipc: http://ipc.localhost https://g.tenor.com; img-src 'self' data: blob: https://g.tenor.com; style-src 'self' 'unsafe-inline'`).
- `withGlobalTauri` را `false` کن؛ فرانت‌اند از ماژول‌های npm (`@tauri-apps/api`) استفاده می‌کند که نیاز به global ندارد.

**نمونه پیکربندی:**

```json
{
  "app": {
    "withGlobalTauri": false,
    "security": {
      "csp": "default-src 'self'; connect-src 'self' ipc: http://ipc.localhost https://g.tenor.com; img-src 'self' data: blob: https://g.tenor.com; style-src 'self' 'unsafe-inline'; script-src 'self'; font-src 'self' data:"
    }
  }
}
```

> **استدلال فنی:** CSP اولین سد دفاعی در برابر XSS است؛ «'self' فقط برای script» یعنی هیچ اسکریپت اینلاین/دیتای خارجی اجرا نمی‌شود. حذف global API هم سطح حمله را از «هر اسکریپت در هر جایی» به «فقط کد باندل خودمان» کاهش می‌دهد.

---

#### ⚠️ ج-۲: دانلود GIF بدون اعتبارسنجی URL و بدون سقف حجم (SSRF + پر کردن دیسک)

**مشکل:** `paste_gif_from_url` در `main.rs` و `Downloader::download` در `gif_manager.rs`:
- هیچ بررسی scheme/دامنه‌ای نمی‌کنند — `reqwest` می‌تواند به هر `http(s)://` داخلی (مثل `http://127.0.0.1:8080/admin`) درخواست بدهد (SSRF).
- هیچ سقف حجمی ندارند — `response.bytes()` کل پاسخ را در حافظه می‌خواند و بعد به دیسک می‌نویسد؛ یک URL مخرب می‌تواند دیسک کاربر را پر کند.

**راهکار عملی:**
- فقط `https://` را بپذیر و آی‌پی‌های خصوصی/loopback را مسدود کن (یا حداقل فقط دامنه‌های مجاز Tenor/Giphy را بپذیر).
- با `Content-Length` و خواندن stream با `take(max_bytes)` سقف بگذار.

**نمونه کد:**

```rust
fn download(url: &str, destination: &Path) -> Result<(), String> {
    const MAX_GIF_BYTES: u64 = 10 * 1024 * 1024; // 10MB سقف
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("Only https URLs are allowed".into());
    }
    // بلاک آی‌پی‌های خصوصی (SSRF): دامنه را resolve و چک کن
    if let Ok(ip) = parsed.host_str().and_then(|h| h.parse::<std::net::IpAddr>()) {
        if ip.is_loopback() || ip.is_private() {
            return Err("Private IPs are not allowed".into());
        }
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT))
        .build().map_err(|e| e.to_string())?;
    let mut resp = client.get(parsed).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() { return Err(format!("HTTP {}", resp.status())); }

    let mut file = fs::File::create(destination).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    for chunk in resp.bytes_stream() {          // استریم به‌جای bytes()
        let chunk = chunk.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        if downloaded > MAX_GIF_BYTES {
            let _ = fs::remove_file(destination); // پاک‌سازی فایل ناقص
            return Err(format!("GIF exceeds {MAX_GIF_BYTES} bytes limit"));
        }
        file.write_all(&chunk).map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

> **استدلال فنی:** SSRF به‌خصوص در اپلیکیشن‌های دسکتاپ جدی است چون می‌تواند به سرویس‌های محلی کاربر (مولفه‌ی شبکه، Docker، ربات‌های محلی) حمله کند؛ سقف حجم هم از OOM و پر شدن دیسک جلوگیری می‌کند.

---

#### ⚠️ ج-۳: مدیریت خطا با `Result<(), String>` و لاگ‌های پراکنده‌ی eprintln

**مشکل:** سراسر کد بک‌اند خطاها `String` هستند (مثلاً `Result<(), String>`) و لاگ‌ها فقط `eprintln!` به stderr — بدون سطوح لاگ، بدون timestamp، بدون فایل لاگ، بدون context. عیب‌یابی مشکلات میدانی (مثلاً «paste کار نمی‌کند») در عمل غیرممکن است.

**راهکار عملی:**
1. یک enum خطای واحد با `thiserror` تعریف کن و `String` را حذف کن.
2. `tracing` را اضافه کن (سبک، سازگار با tokio) و خروجی را به `~/.local/share/win11-clipboard-history/app.log` بنویس (با گردش حجم).
3. رویدادهای مهم (paste، کپی، خطا) را به‌صورت ساختاریافته لاگ کن.

**نمونه کد:**

```rust
// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),
    #[error("history persistence error: {0}")]
    Persistence(String),
    #[error("input simulation failed: {0}")]
    InputSimulation(String),
    #[error("item '{id}' not found")]
    NotFound { id: String },
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

```rust
// در main() یا setup — راه‌اندازی tracing با فایل لاگ
fn init_tracing() {
    let log_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
        .join("win11-clipboard-history");
    let _ = fs::create_dir_all(&log_dir);
    let appender = tracing_appender::rolling::daily(log_dir, "app.log");
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,win11_clipboard_history=debug".into()),
        )
        .with_writer(appender)
        .init();
}
```

> **استدلال فنی:** `thiserror` خطاهای قابل دسته‌بندی و `?` تمیز می‌دهد و `tracing` به‌جای eprintln پراکنده، لاگ ساختاریافته با span و سطح‌بندی فراهم می‌کند — بدون این، هر باگ میدانی فقط با «دوباره نصب کن» قابل حل است.

---

#### ⚠️ ج-۴: کپی هم‌زمان فایل‌های تنظیمات (race روی disk)

**مشکل:** `UserSettingsManager::save` و `ConfigManager::save_to_disk` همگی `fs::write` مستقیم دارند؛ و رویداد `app-settings-changed` + تایم‌های مختلف (hide کردن پنجره در Wayland → `sync_to_disk`) یعنی دو thread می‌توانند هم‌زمان بنویسند.

**راهکار عملی:** یک ماژول `fs_atomic.rs` واحد: `write_atomic(path, bytes)` = write به `.tmp` + `rename`، و همه‌ی رایت‌ها را از آن عبور بده.

**نمونه کد:**

```rust
// src-tauri/src/fs_atomic.rs
pub fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path) // atomic جایگزینی
}
```

> **استدلال فنی:** `rename` در همان فایل‌سیستم اتمیک است؛ دو thread که هم‌زمان می‌نویسند یا فایل کاملِ اولی یا کاملِ دومی را می‌بینند — نه ترکیبی خراب از هر دو.

---

#### ⚠️ ج-۵: تست و CI

**مشکل:** در `.github/workflows/ci.yml` فقط lint و build و audit اجرا می‌شود؛ **`cargo test` اصلاً در CI نیست** در حالی که پروژه تست‌های unit خوبی دارد (`input_simulator.rs`، `user_settings.rs`، `emoji_manager.rs`). فرانت‌اند هم هیچ تستی ندارد.

**راهکار عملی:** یک job تست اضافه کن؛ برای فرانت‌اند، Vitest + React Testing Library را برای حداقل بحرانی‌ترین منطق (فیلتر کردن تاریخچه، smart actions) اضافه کن.

**نمونه (بخش CI):**

```yaml
  test:
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      # ... نصب وابستگی‌های سیستمی مشترک ...
      - name: Run Rust unit tests
        run: cd src-tauri && cargo test --all-features
```

---

### د) فرانت‌اند و تجربه کاربری (Frontend Optimization)

---

#### ⚠️ د-۱: لیست تاریخچه بدون ویرچوالایزیشن رندر می‌شود

**مشکل:** در `src/components/ClipboardTab.tsx` همه‌ی آیتم‌ها با `filteredHistory.map(...)` رندر می‌شوند در حالی که `max_history_size` تا **۱۰۰,۰۰۰** قابل تنظیم است. با هر آیتم شامل div + آیکون + متن + دکمه‌ها + `animationDelay`، رندر ۱۰ هزار آیتم = انجماد چند ثانیه‌ای و مصرف حافظه‌ی سنگین. نکته‌ی جالب: `react-window` از قبل dependency پروژه است ولی فقط در EmojiPicker استفاده می‌شود.

**راهکار عملی:** لیست تاریخچه را با `FixedSizeList` (یا `VariableSizeList`) ویرچوالایز کن؛ فقط آیتم‌های قابل مشاهده DOM شوند. و `animationDelay` را روی ورود اولیه محدود کن (مثلاً فقط ۱۰ آیتم اول).

**نمونه کد:**

```tsx
import { FixedSizeList } from 'react-window'

// در ClipboardTab — به‌جای حلقه‌ی map
<FixedSizeList
  height={viewportHeight}
  width="100%"
  itemCount={filteredHistory.length}
  itemSize={isCompact ? 44 : 64}
  itemData={{ items: filteredHistory, ...handlers }}
>
  {({ index, style, data }) => (
    <div style={style}>
      <HistoryItem
        item={data.items[index]}
        index={index}
        // animationDelay فقط برای ۱۲ آیتم اول، بقیه 0
        // ...
      />
    </div>
  )}
</FixedSizeList>
```

> **استدلال فنی:** ویرچوالایزیشن هزینه‌ی رندر را از O(n) به O(viewport) می‌رساند؛ با ۱۰۰k آیتم، همیشه فقط ~۱۵-۲۰ آیتم در DOM هستند — جوابگویی کلیدهای جهت‌نما (که الان با آرایه‌ی refها کار می‌کند) هم باید با `scrollToIndex` بازنویسی شود.

---

#### ⚠️ د-۲: در هر `clipboard-changed` کل تاریخچه دوباره fetch می‌شود + سه مکانیزم همگام‌سازی موازی

**مشکل:** در `src/hooks/useClipboardHistory.ts` رویداد `clipboard-changed` پیلود آیتم جدید را دارد اما handler آن باز `fetchHistory()` صدا می‌زند (کل IPC). سه رویداد موازی (`clipboard-changed`، `history-cleared`، `history-sync`) برای همگام‌سازی وجود دارد و رفتارها هم‌پوشانند.

**راهکار عملی:**
- `clipboard-changed` → prepend آیتم جدید + trim سمت کلاینت (بدون round-trip).
- `history-cleared` → فقط filter.
- `history-sync` را فقط برای مواردی نگه دار که ترتیب واقعاً تغییر کرده (مثلاً بعد از paste که آیتم به بالا منتقل می‌شود).

**نمونه کد:**

```ts
const uChanged = await listen<ClipboardItem>('clipboard-changed', (event) => {
  // به‌جای fetchHistory: prepend + trim با max از settings
  setHistory((prev) => {
    const next = [event.payload, ...prev.filter((i) => i.id !== event.payload.id)]
    return next.slice(0, maxHistorySizeRef.current)
  })
})
```

> **استدلال فنی:** حذف round-trip کامل تاریخچه در هر کپی، ترافیک IPC و رندر را از «کل لیست» به «یک آیتم» کاهش می‌دهد — در ترکیب با ب-۱ (حذف base64 از پیلودها) تأثیر چشمگیر است.

---

#### ⚠️ د-۳: باندل اولیه شامل همه‌ی تب‌ها و دیتاست‌های سنگین است

**مشکل:** `ClipboardApp.tsx` همه‌ی پیکرها (`EmojiPicker`، `KaomojiPicker`، `SymbolPicker`) را استاتیک import می‌کند؛ و `src/data/symbols.json` (۵۱۲KB) و `kaomojis.json` (۱۳۶KB) — اگر استاتیک import شوند، بخشی از باندل اصلی می‌شوند. `emojilib` هم کل دیکشنری (~۱,۸۰۰ ایموجی) را در باندل اصلی می‌آورد.

**راهکار عملی:**
- تب‌ها را `React.lazy` + `Suspense` کن تا هر تب فقط وقتی باز شد دانلود شود.
- `symbols.json` و `kaomojis.json` را با `import()` داینامیک (route-based code splitting) بارگذاری کن.

**نمونه کد:**

```tsx
import { lazy, Suspense } from 'react'

const EmojiPicker = lazy(() => import('./components/EmojiPicker')
  .then((m) => ({ default: m.EmojiPicker })))
const KaomojiPicker = lazy(() => import('./components/KaomojiPicker')
  .then((m) => ({ default: m.KaomojiPicker })))
const SymbolPicker = lazy(() => import('./components/SymbolPicker')
  .then((m) => ({ default: m.SymbolPicker })))

// در renderContent:
case 'emoji':
  return <Suspense fallback={<Spinner />}><EmojiPicker isDark={isDark} opacity={secondaryOpacity} /></Suspense>
```

> **استدلال فنی:** چون کاربر معمولاً فقط تب کلیپ‌بورد را باز می‌کند، بارگذاری ۶۰۰KB+ داده‌ی ایموجی/سمبل در استارت‌آپ هدر است؛ lazy loading زمان تا اولین نمایش (TTFP) را کاهش می‌دهد.

---

#### ⚠️ د-۴: تکرار `DEFAULT_SETTINGS` بین فرانت‌اند و بک‌اند

**مشکل:** `DEFAULT_SETTINGS` در `ClipboardApp.tsx` کپی‌ی دستی از `UserSettings::default()` در `user_settings.rs` است — هر بار که فیلدی به Rust اضافه شود، فرانت‌اند تا دریافت تنظیمات از بک‌اند (که async است) fallback ناقص دارد و risk of drift.

**راهکار عملی:** یک command جدید `get_default_settings()` در Rust اضافه کن و فرانت‌اند را با آن مقداردهی کن؛ یا حداقل تایپ `UserSettings` را single-source با validation مشترک (مثلاً zod schema هم‌سنگ با `validate()` در Rust) کن.

**نمونه کد:**

```rust
#[tauri::command]
fn get_default_settings() -> UserSettings {
    UserSettings::default()
}
```

```ts
// ClipboardApp.tsx
const DEFAULT_SETTINGS = await invoke<UserSettings>('get_default_settings')
```

> **استدلال فنی:** تک‌منبع‌سازی پیش‌فرض‌ها یعنی افزودن تنظیم جدید فقط در یک جا انجام می‌شود و UI هرگز با state ناسازگار رندر نمی‌شود.

---

### هـ) زیرساخت و DevOps

---

#### ⚠️ هـ-۱: `Cargo.lock` از گیت حذف شده — بیلدها غیرقابل بازتولید

**مشکل:** `.gitignore` شامل `src-tauri/Cargo.lock` است (و `git ls-files` تأیید می‌کند ترک نمی‌شود). برای اپلیکیشن‌ها (نه کتابخانه‌ها) طبق مستندات رسمی Cargo و الگوی Tauri، `Cargo.lock` **باید** commit شود؛ بدون آن، هر `cargo build` در CI یا روی ماشین کاربر ممکن است وابستگی‌های متفاوتی resolve کند و «روی ماشین من کار می‌کند» کلاسیک را تولید کند.

**راهکار عملی:** خط `src-tauri/Cargo.lock` را از `.gitignore` حذف و فایل را commit کن.

**نمونه:**

```gitignore
# حذف این خط از .gitignore:
# src-tauri/Cargo.lock
```

```bash
git add -f src-tauri/Cargo.lock && git commit -m "chore: commit Cargo.lock for reproducible builds"
```

> **استدلال فنی:** پین کردن دقیق نسخه‌ی ۴۰۰+ کریت، هم بازتولیدپذیری بیلد CI را تضمین می‌کند هم `cargo audit` را معنادار می‌کند (ادعای امنیتی روی مجموعه‌ی مشخصی از وابستگی‌ها).

---

#### ⚠️ هـ-۲: CI تکراری و بدون تست

**مشکل:** در `ci.yml` سه job (lint، build-linux، security) هر کدام checkout + setup-node + rust-toolchain + نصب وابستگی‌های سیستمی را تکرار می‌کنند؛ `cargo test` اجرا نمی‌شود؛ caching دستی `actions/cache` است (در حالی که `dtolnay/rust-toolchain` به‌صورت built-in `cache: true` دارد) و کلید کش شامل profile یا toolchain نیست.

**راهکار عملی:**
- یک **composite action** یا `Makefile` مشترک برای مراحل تکراری.
- job تست اضافه کن (ج-۵).
- از `dtolnay/rust-toolchain@stable` با `cache: true` و `actions/setup-node` با cache موجود استفاده کن (الان هست) و کلید کش را با شامل‌کردن `Cargo.lock` + profile دقیق کن.

**نمونه (dedupe با composite action):**

```yaml
# .github/actions/setup-env/action.yml
runs:
  using: "composite"
  steps:
    - uses: actions/checkout@v7
    - uses: actions/setup-node@v6
      with: { node-version: "20", cache: "npm" }
    - uses: dtolnay/rust-toolchain@stable
      with: { components: "rustfmt, clippy", cache: true }
    - run: |
        sudo apt-get update
        sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
          libssl-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev \
          libgtk-3-dev libglib2.0-dev
      shell: bash
    - run: npm ci
      shell: bash
```

```yaml
# ci.yml — استفاده مجدد
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
      - run: npm run lint
      - run: cd src-tauri && cargo clippy -- -D warnings
```

> **استدلال فنی:** یک‌بار تعریف، چندبار استفاده = نگهداری کمتر و اطمینان از اینکه lint و build و test دقیقاً روی همان محیط اجرا می‌شوند؛ `cache: true` داخلی dtolnay هم نگهداری دستی کش را حذف می‌کند.

---

#### ⚠️ هـ-۳: نبود Dockerfile برای بیلد/تست بازتولیدپذیر

**مشکل:** پروژه فایل `Dockerfile` ندارد. برای اپ دسکتاپ، Docker محصول نهایی نیست، اما یک **environment بیلد/CI پین‌شده** (با همه‌ی وابستگی‌های WebKitGTK) خطاهای «روی ماشین من بیلد می‌شود» را حذف می‌کند و برای تست خودکار در CI هم به‌کار می‌آید.

**راهکار عملی:** یک Dockerfile چندمرحله‌ای بر پایه‌ی image رسمی تائوری (`ghcr.io/tauri-apps/tauri:ubuntu-22.04`) ارائه کن — برای dev/test در CI و توسعه‌ی لوکال (نه برای پکیج نهایی deb/rpm).

**نمونه Dockerfile:**

```dockerfile
# syntax=docker/dockerfile:1
# بیلد/تست بازتولیدپذیر برای win11-clipboard-history
FROM ghcr.io/tauri-apps/tauri:ubuntu-22.04 AS build

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./src-tauri/
COPY src-tauri/src ./src-tauri/src
COPY src-tauri/capabilities ./src-tauri/capabilities
COPY src-tauri/tauri.conf.json src-tauri/build.rs ./src-tauri/
COPY src-tauri/icons ./src-tauri/icons
COPY src-tauri/bundle ./src-tauri/bundle
COPY index.html vite.config.ts tsconfig.json eslint.config.js postcss.config.js tailwind.config.js ./
COPY src ./src
COPY public ./public

# عکس CI (بدون رابط گرافیکی): فقط lint + test + build فرانت
RUN npm run lint \
 && cd src-tauri && cargo test --all-features \
 && cd /app && npm run build

# (اختیاری) بیلد کامل Tauri در صورت داشتن Xvfb
# RUN xvfb-run -a npm run tauri:build
```

> **استدلال فنی:** پین‌کردن toolchain و وابستگی‌های سیستمی در image، «روی CI درست می‌شود ولی لوکال نه» را حذف می‌کند؛ لایه‌بندی صحیح (کپی package.json قبل از کد) کش لایه‌ی npm را حفظ می‌کند و بیلدهای تکراری را سریع می‌کند.

---

#### ⚠️ هـ-۴: همگام‌سازی نسخه با sed و کلید API هاردکد

**مشکل:** در `release.yml` نسخه با سه `sed` جداگانه (package.json، tauri.conf.json، Cargo.toml) همگام می‌شود — منابع حقیقت پراکنده. و `TENOR_API_KEY = 'LIVDSRZULELA'` در `src/services/gifService.ts` داخل باندل کاربر نهایی است (هر کلیدی در کلاینت قابل استخراج است).

**راهکار عملی:**
- یک اسکریپت واحد `scripts/sync-version.sh` (یا `cargo set-version` + `npm version`) که فقط یک `VERSION` می‌گیرد و هر سه فایل را از آن به‌روز می‌کند؛ در CI فقط اسکریپت صدا زده شود.
- کلید Tenor را به یک **پروکسی بک‌اند** منتقل کن: یک command Rust `search_tenor(query)` که کلید را سمت سرور نگه می‌دارد (در env) و فرانت‌اند دیگر هرگز کلید نمی‌بیند (این با ج-۲ هم‌پوشانی دارد).

**نمونه:**

```rust
// command جدید: جستجوی GIF از بک‌اند (کلید از env، نه از باندل)
#[tauri::command]
async fn search_tenor(query: Option<String>, limit: Option<u32>) -> Result<Vec<Gif>, String> {
    let api_key = std::env::var("TENOR_API_KEY")
        .map_err(|_| "TENOR_API_KEY not set".to_string())?;
    let endpoint = if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
        format!("{TENOR_API_BASE}/search?q={}&key={}&limit={}", q, api_key, limit.unwrap_or(30))
    } else {
        format!("{TENOR_API_BASE}/trending?key={}&limit={}", api_key, limit.unwrap_or(30))
    };
    // fetch + transform سمت سرور ...
}
```

> **استدلال فنی:** قرار دادن کلید در بک‌اند یعنی key rotation بدون انتشار نسخه‌ی جدید ممکن است، محدودیت نرخ (rate limit) قابل اعمال است، و اعتبارسنجی URL (ج-۲) در یک نقطه انجام می‌شود؛ اسکریپت واحد نسخه هم خطای انسانی sed را حذف می‌کند.

---

## 📌 ۳. ماتریس اولویت‌بندی اقدام (Action Prioritization Matrix)

| اولویت | حوزه | اقدام مورد نیاز | تاثیر (Impact) | هزینه/زمان (Effort) |
|--------|------|-----------------|----------------|---------------------|
| 🔴 فوری | کارایی/معماری | مهاجرت `history.json` به SQLite + ذخیره‌ی تصاویر به‌صورت فایل با تامبنیل (ب-۱/الف-۱) | **بالا** — حذف پایه‌ای bloat حافظه/دیسک/IPC و O(n²) | متوسط (۲–۳ هفته) |
| 🔴 فوری | کارایی | بازیافت `Clipboard` در watcher و کوتاه‌کردن پنجره‌ی قفل (ب-۱) | **بالا** — کاهش تأخیر paste و هزینه‌ی هر تیک نظارت | کم (نیم روز) |
| 🔴 فوری | امنیت | فعال‌سازی CSP + `withGlobalTauri: false` (ج-۱) | **بالا** — بستن سطح حمله‌ی XSS | کم (ساعت‌ها) |
| 🔴 فوری | کارایی | حذف fetch مجدد کل تاریخچه روی `clipboard-changed` (د-۲) | **متوسط/بالا** — کاهش ترافیک IPC و رندر | کم (ساعت‌ها) |
| 🔴 فوری | پایداری | رایت اتمیک (`tmp` + `rename`) برای history/settings (ب-۲/ج-۴) | **متوسط** — جلوگیری از خرابی فایل داده در crash | کم (ساعت‌ها) |
| 🟡 مهم | امنیت | سقف حجم و اعتبارسنجی URL در دانلود GIF + انتقال کلید Tenor به بک‌اند (ج-۲/هـ-۴) | **متوسط/بالا** — جلوگیری از SSRF و پر شدن دیسک | متوسط (۲–۳ روز) |
| 🟡 مهم | فرانت‌اند | ویرچوالایز کردن `ClipboardTab` با react-window (د-۱) | **متوسط/بالا** — جوابگویی با ۱۰k+ آیتم | متوسط (۲–۴ روز) |
| 🟡 مهم | معماری | جایگزینی `Result<(), String>` با `thiserror` + لاگ‌های `tracing` (ج-۳) | **متوسط** — عیب‌یابی میدانی ممکن می‌شود | متوسط (۲–۳ روز) |
| 🟡 مهم | DevOps | commit کردن `Cargo.lock` (هـ-۱) | **متوسط** — بیلد بازتولیدپذیر | کم (دقیقه‌ها) |
| 🟡 مهم | DevOps | افزودن `cargo test` به CI + dedupe مراحل با composite action (ج-۵/هـ-۲) | **متوسط** — جلوگیری از رگرسیون | کم (نیم روز) |
| 🟡 مهم | کارایی | هش پایدار FNV برای کش GIF + TTL کش (ب-۳) | **متوسط** — حذف دانلودهای تکراری | کم (ساعت‌ها) |
| 🟢 اختیاری | فرانت‌اند | Lazy loading تب‌ها و دیتاست‌های ایموجی/سمبل (د-۳) | متوسط | کم (نیم روز) |
| 🟢 اختیاری | معماری | شکستن `main.rs` به ماژول‌ها + حذف کد مرده (الف-۳) | متوسط (نگهداری) | متوسط |
| 🟢 اختیاری | معماری | ماژول واحد `clipboard_io.rs` برای حذف تکرار کد کلیپ‌بورد (الف-۴) | متوسط (نگهداری) | متوسط |
| 🟢 اختیاری | DevOps | Dockerfile بیلد/تست بازتولیدپذیر (هـ-۳) | متوسط (CI) | کم |
| 🟢 اختیاری | فرانت‌اند | تک‌منبع‌سازی DEFAULT_SETTINGS از بک‌اند (د-۴) | پایین | کم |
| 🟢 اختیاری | کارایی | شاخص `HashSet` برای تشخیص تکراری O(1) و `truncate` به‌جای حلقه‌ی remove (الف-۲) | متوسط | کم |

**ترتیب پیشنهادی اجرا:** ابتدا ستون 🔴 (هزینه‌ی کم، برد بالا: ب-۱، ج-۱، د-۲، ب-۲) را در یک یا دو اسپرینت ببندید؛ سپس مهاجرت به SQLite (الف-۱) را با دقت و با حفظ schema قدیمی به‌عنوان fallback انجام دهید؛ در موازات، آیتم‌های 🟡 و در نهایت 🟢.

---

## ❓ ۴. اطلاعات تکمیلی مورد نیاز

برای دقیق‌تر کردن برخی راهکارها (مخصوصاً طرح ایندکس/مایگریشن دیتابیس و مقیاس‌پذیری) لطفاً پاسخ دهید:

1. **الگوی استفاده‌ی واقعی:** میانگین حجم تاریخچه در دستگاه‌های کاربران چقدر است؟ (بیشتر کاربران زیر ۵۰ آیتم‌اند یا افرادی با `max_history_size` بالا مثل ۱۰٬۰۰۰+ وجود دارند؟) — این تعیین می‌کند مهاجرت SQLite اولویت اول باشد یا صرفاً خروج تصاویر از JSON کافی است.
2. **اهمیت RichText/HTML:** آیا نگه‌داری HTML کامل برای هر آیتم RichText حیاتی است یا می‌توان آن را به‌صورت فشرده/فقط-plain نگه داشت؟ (حجم `history.json` مستقیماً به این بستگی دارد.)
3. **تصاویر:** آیا نیاز به paste مجدد تصویر با کیفیت اصلی وجود دارد یا تامبنیل ۵۱۲px (و در صورت درخواست paste، خواندن فایل اصلی) کافی است؟ و آیا تا به حال حجم‌های بالای ۵۰–۱۰۰MB برای `history.json` گزارش شده؟
4. **مخاطره‌پذیری مهاجرت:** آیا کاربران نسخه‌های قدیمی با `history.json` موجود وجود دارند که مهاجرت داده باید backward-compatible باشد (فایل قدیمی خوانده و به SQLite منتقل شود) یا پروژه در مرحله‌ای است که reset داده قابل قبول است؟
5. **هدف توزیع:** آیا انتشار از طریق apt-repo (Cloudsmith) و AUR برای همه‌ی نسخه‌ها الزامی است (که روی انتخاب CI/Docker تأثیر می‌گذارد) یا GitHub Releases تنها کانال است؟
6. **محدودیت اتصال X11 در محیط‌های کاربر:** آیا گزارش‌هایی از «paste کند» یا «غیرفعال شدن کلیپ‌بورد پس از مدتی» در X11 وجود دارد؟ (برای تصمیم بین نگه‌داشتن polling فعلی vs رویدادمحور شدن.)

---

*گزارش بر اساس کد موجود در commit `107c8f6` (شاخه master) تهیه شده؛ تمام ارجاع‌های فایل/تابع با خط‌های واقعی کد تطبیق داده شده‌اند.*
