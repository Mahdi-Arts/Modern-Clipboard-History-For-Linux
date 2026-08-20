# 🔐 Security Policy / سیاست امنیتی

<div dir="rtl">

## امنیت برای ما اولویت است

ما امنیت این پروژه را جدی می‌گیریم. اگر آسیب‌پذیری امنیتی کشف کرده‌اید، لطفاً مراحل زیر را دنبال کنید.

## گزارش آسیب‌پذیری

**⚠️ لطفاً برای گزارش آسیب‌پذیری‌های امنیتی، issue عمومی باز نکنید.**

در عوض، از یکی از روش‌های زیر استفاده کنید:

1. **GitHub Security Advisory**: از [قابلیت گزارش خصوصی GitHub](https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/security/advisories/new) استفاده کنید
2. **ایمیل**: gustaavoribeeiro@hotmail.com

### اطلاعات مورد نیاز

- **شرح** آسیب‌پذیری
- **مراحل بازتولید** مسئله
- **تأثیر بالقوه** آسیب‌پذیری
- **راه حل پیشنهادی** (اگر دارید)

### زمان‌بندی پاسخ

| سطح | زمان پاسخ |
| --- | --- |
| پاسخ اولیه | تا ۴۸ ساعت |
| به‌روزرسانی وضعیت | تا ۱ هفته |
| رفع بحرانی | ۲۴-۷۲ ساعت |
| رفع بالا | ۱ هفته |
| رفع متوسط | ۲ هفته |
| رفع پایین | انتشار بعدی |

</div>

---

## We Take Security Seriously

If you discover a security vulnerability, please follow these steps:

**⚠️ Do NOT open a public issue for security vulnerabilities.**

### Private Disclosure Methods

1. **GitHub Security Advisory**: Use [private vulnerability reporting](https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/security/advisories/new)
2. **Email**: gustaavoribeeiro@hotmail.com

### What to Include

- **Description** of the vulnerability
- **Steps to reproduce**
- **Potential impact**
- **Suggested fix** (if any)

### Response Timeline

| Severity | Response Time |
| --- | --- |
| Initial Response | Within 48 hours |
| Status Update | Within 1 week |
| Critical Fix | 24-72 hours |
| High | 1 week |
| Medium | 2 weeks |
| Low | Next release |

---

## Security Best Practices / بهترین روش‌های امنیتی

<div dir="rtl">

### هنگام استفاده

1. **همیشه به‌روز باشید**: از آخرین نسخه استفاده کنید
2. **از منبع معتبر نصب کنید**: از مخازن رسمی استفاده کنید
3. **مشکلات را گزارش دهید**: به ما در شناسایی رفتارهای مشکوک کمک کنید

### حریم خصوصی داده‌ها

- تاریخچه کلیپ‌بورد **فقط به صورت محلی** ذخیره می‌شود
- هیچ داده‌ای از طریق شبکه ارسال نمی‌شود
- اطلاعات حساس که کپی می‌شوند در تاریخچه ذخیره می‌شوند

### دسترسی‌های مورد نیاز

- **کلید میانبر سراسری**: برای `Super+V` و `Ctrl+Alt+V`
- **System Tray**: برای اجرای پس‌زمینه
- **دسترسی به کلیپ‌بورد**: عملکرد اصلی برنامه

### امنیت Wayland

در Wayland، دسترسی به کلیپ‌بورد تابع مدل امنیتی کامپوزیتور است که ممکن است دسترسی برنامه‌های پس‌زمینه را محدود کند.

</div>

### App Security Features / ویژگی‌های امنیتی برنامه

✔️ **CSP (Content Security Policy)** فعال — محافظت در برابر XSS  
✔️ `withGlobalTauri: false` — API تائوری در معرض اسکریپت‌های خارجی نیست  
✔️ **SSRF Protection** — دانلود GIF فقط به HTTPS و آی‌پی‌های عمومی محدود شده  
✔️ **سقف حجم** — دانلودها به ۱۰MB محدود شده‌اند  
✔️ **رایت اتمیک** — فایل‌های داده در صورت Crash خراب نمی‌شوند  
✔️ **ذخیره‌سازی محلی** — هیچ داده‌ای به سرور ارسال نمی‌شود  
✏️ <sub>_منبع باز: امنیت با شفافیت کد_</sub>

---

**Thank you for helping keep this project secure! 🔐**  
**از کمک شما برای امنیت این پروژه سپاسگزاریم! 🔐**