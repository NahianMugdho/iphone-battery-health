# iPhone Battery Health

USB-তে লাগানো iPhone-এর ব্যাটারি হেলথ দেখানোর একটি ডেস্কটপ অ্যাপ। **Windows ও Linux** দুই জায়গাতেই চলে।

- **Frontend:** React + JavaScript (Vite)
- **Backend:** Rust + [Tauri v2](https://tauri.app)
- **iPhone যোগাযোগ:** [`idevice`](https://crates.io/crates/idevice) crate (pure Rust, libimobiledevice-এর মতো কাজ করে)। আলাদা C লাইব্রেরি বা DLL বান্ডেল করতে হয় না।

## ফিচার

- ফোন লাগালে/খুললে ডিভাইস নিজে নিজে শনাক্ত হয় (অটো-আপডেট)
- ডিভাইসের নাম, মডেল (`ProductType`), iOS ভার্সন
- ব্যাটারি হেলথ, সাইকেল ব্যবহার, রানটাইম, চার্জ হতে সময়, অ্যাডাপ্টার পাওয়ার
- প্রতি ৫ সেকেন্ডে রিফ্রেশ
- ফোনে Trust দেওয়া না থাকলে বাংলায় সহজ বার্তা

## ব্যাটারি হিসাবের সূত্র

ডেটা আসে `diagnostics_relay` সার্ভিসের `AppleSmartBattery` এন্ট্রি থেকে (`idevicediagnostics ioregentry AppleSmartBattery`-এর সমতুল্য)।

| মান | সূত্র |
|---|---|
| Battery Health (Nominal) | `NominalChargeCapacity / DesignCapacity × 100` |
| Health Estimate (Full Charge) | `FullChargeCapacity / DesignCapacity × 100` |
| Capacity Difference | `FullChargeCapacity − NominalChargeCapacity` (mAh) |
| Cycle Usage | `CycleCount / Rated cycles × 100` |
| Runtime (ঘণ্টা) | `RemainingCapacity / |Amperage|` (ডিসচার্জের সময়) |
| Time to Full (ঘণ্টা) | `(NominalChargeCapacity − RemainingCapacity) / Amperage` (চার্জের সময়) |
| Adapter Power | `AdapterVoltage(mV) × Current(mA) / 10^6` (W) |

**সীমাবদ্ধতা**

- iPhone-এর Settings-এ দেখানো "Maximum Capacity" Apple-এর নিজস্ব হিসাব। এই অ্যাপের ফলাফলে কয়েক শতাংশ ফারাক হতে পারে।
- সব মডেলে সব key থাকে না। না পেলে `—` দেখায়, ক্র্যাশ করে না।
- `RemainingCapacity` ১০০ বা তার কম এলে সেটা শতাংশ ধরা হয়, তাই Runtime ও Time to Full দেখানো হয় না।
- **Rated cycles** মডেল দেখে অনুমান করা হয় (iPhone15,4 ও পরের মডেলে ১০০০, আগেরগুলোতে ৫০০)। Apple-এর অফিসিয়াল তথ্যের সাথে মিলিয়ে `src-tauri/src/battery.rs`-এর `rated_cycles()` আপডেট করুন।

## চালানোর পূর্বশর্ত

`idevice` নিজে usbmuxd নয়, usbmuxd-এর ক্লায়েন্ট। তাই:

| OS | যা লাগবে |
|---|---|
| **Windows** | Apple Mobile Device Service (iTunes বা Apple Devices অ্যাপ ইনস্টল থাকলে থাকে) |
| **Linux** | `usbmuxd` প্যাকেজ ইনস্টল ও চালু |

এছাড়া **ফোন আনলক করে USB-তে লাগিয়ে "Trust This Computer" দিতে হবে।** Trust না দিলে ব্যাটারি ডেটা আসে না (`missing PairRecordData` এরর)।

## ইনস্টল করে ব্যবহার

### Windows

1. `.msi` বা `-setup.exe` ইনস্টলার চালান। (কোড সাইনিং নেই, তাই SmartScreen সতর্কবার্তা এলে **More info → Run anyway**।)
2. iTunes বা Apple Devices অ্যাপ ইনস্টল আছে কিনা নিশ্চিত করুন।
3. ফোন লাগান, আনলক করুন, **Trust** দিন।
4. অ্যাপ খুলুন।

### Linux (Debian/Ubuntu)

Debian 12 বা নতুন, Ubuntu 22.04 বা নতুন ধরে বানানো। পুরনো ডিস্ট্রোতে না-ও চলতে পারে।

```bash
sudo apt update
sudo apt install ./iphone-battery_*.deb
sudo apt install usbmuxd
sudo systemctl enable --now usbmuxd
```

অ্যাপ মেনু থেকে "iPhone Battery Health" চালান, অথবা টার্মিনালে:

```bash
iphone-battery
```

বাইনারির নাম না মিললে `dpkg -L iphone-battery | grep /bin/` দিয়ে দেখুন।

AppImage হলে:

```bash
chmod +x *.AppImage
./*.AppImage
```

## ডেভেলপমেন্ট

### প্রয়োজনীয় টুল

- Node.js (LTS) ও npm
- Rust ([rustup.rs](https://rustup.rs))
- **Windows:** Visual Studio C++ Build Tools, WebView2
- **Linux (Debian/Ubuntu):**
  ```bash
  sudo apt install build-essential curl wget file libwebkit2gtk-4.1-dev \
    libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev usbmuxd
  ```

### চালানো

```bash
npm install
npm run tauri dev
```

**গুরুত্বপূর্ণ:** `npm run tauri dev` যে নেটিভ উইন্ডো খোলে সেটাতেই অ্যাপ চলে। ব্রাউজারে `localhost:1420` খুললে Rust কমান্ড পৌঁছায় না (`__TAURI_INTERNALS__ is undefined` এরর আসবে)।

### Rust অংশ আলাদা চেক

```bash
cd src-tauri
cargo check
```

### বিল্ড

প্রতিটা OS-এর ইনস্টলার সেই OS-এই বানাতে হয়।

```bash
npm run tauri build
```

আউটপুট `src-tauri/target/release/bundle/`-এ:

| OS | ফাইল |
|---|---|
| Windows | `msi/*.msi`, `nsis/*-setup.exe` |
| Linux | `deb/*.deb`, `appimage/*.AppImage` |

### GitHub Actions দিয়ে দুই OS একসাথে

`.github/workflows/build.yml` Windows (`windows-latest`) ও Linux (`ubuntu-22.04`) দুটোই বানায়।

1. repo-তে **Actions → Build installers → Run workflow** চাপুন (অথবা `git tag v0.1.0 && git push --tags`)।
2. রান শেষ হলে রানের **Summary** পেজের নিচে **Artifacts** থেকে ইনস্টলার ডাউনলোড করুন।

## প্রজেক্ট স্ট্রাকচার

```
iphone-battery/
├── package.json
├── index.html
├── vite.config.js
├── .github/workflows/build.yml     # Windows + Linux বিল্ড
├── src/                            # React frontend
│   ├── main.jsx
│   ├── App.jsx
│   ├── App.css
│   ├── components/
│   │   └── BatteryCard.jsx
│   ├── hooks/
│   │   ├── useDevices.js           # ডিভাইস তালিকা + devices-changed ইভেন্ট
│   │   └── useBattery.js           # প্রতি ৫ সেকেন্ডে get_battery
│   └── utils/
│       └── format.js
└── src-tauri/                      # Rust backend
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/default.json
    └── src/
        ├── main.rs
        ├── lib.rs                  # কমান্ড রেজিস্টার + ডিভাইস পর্যবেক্ষণ থ্রেড
        ├── device.rs               # usbmuxd + lockdown (ডিভাইস শনাক্ত)
        └── battery.rs              # diagnostics_relay + ব্যাটারি সূত্র
```

## React ↔ Rust যোগাযোগ

| দিক | Rust | React |
|---|---|---|
| React → Rust | `#[tauri::command] get_battery(udid, product_type)` | `invoke("get_battery", { udid, productType })` |
| React → Rust | `#[tauri::command] list_devices()` | `invoke("list_devices")` |
| Rust → React | `app.emit("devices-changed", devices)` | `listen("devices-changed", cb)` |

- Rust-এর `snake_case` প্যারামিটার JS-এ `camelCase` হয় (`product_type` → `productType`)।
- Rust struct-এ `#[serde(rename_all = "camelCase")]` থাকায় JSON-এ `healthNominal`, `timeToFullH` ইত্যাদি আসে।

## `idevice` crate নিয়ে সতর্কতা

এই crate এখনও 0.1.x, প্রতি point release-এ API ভাঙে। তাই `Cargo.toml`-এ ভার্সন পিন করা আছে:

```toml
idevice = { version = "=0.1.60", default-features = false, features = ["rustls", "ring", "usbmuxd", "diagnostics_relay"] }
```

- `rustls` ফিচারের সাথে একটা crypto backend (`ring` বা `aws-lc`) লাগে, না হলে `No crypto backend was selected` এরর আসে।
- `to_provider`, `get_value` ইত্যাদির আর্গুমেন্ট ভার্সনে বদলায়। কম্পাইল এরর এলে `device.rs`-এর `make_provider` ও `lockdown_string` ফাংশন দেখুন।

## সমস্যা সমাধান

| সমস্যা | সমাধান |
|---|---|
| ডিভাইস দেখা যায় না | ফোন আনলক ও Trust দেওয়া আছে কিনা দেখুন। Windows-এ Apple Mobile Device Service চালু (`services.msc`), Linux-এ `systemctl status usbmuxd` |
| `missing PairRecordData in pair record response` | ফোনে Trust দেওয়া হয়নি। ক্যাবল খুলে আবার লাগিয়ে **Trust** চাপুন |
| Trust প্রম্পট আসে না | iPhone: Settings → General → Transfer or Reset iPhone → Reset → **Reset Location & Privacy** |
| `Cannot read properties of undefined (reading 'invoke')` | ব্রাউজারে না খুলে `npm run tauri dev`-এর নেটিভ উইন্ডোতে দেখুন |
| `No crypto backend was selected` | `idevice` ফিচারে `ring` (বা `aws-lc`) যোগ করুন |
| Linux-এ অ্যাপ খুলে সাদা উইন্ডো | টার্মিনাল থেকে চালিয়ে এরর দেখুন |
| Runtime / Time to Full `—` দেখায় | `RemainingCapacity` শতাংশ হিসেবে আসছে, অথবা ফোন চার্জ/ডিসচার্জের অবস্থার সাথে মিলছে না |

## লাইসেন্স

নিজের পছন্দমতো লাইসেন্স এখানে যোগ করুন।

## ক্রেডিট

- [Tauri](https://tauri.app)
- [`idevice`](https://github.com/jkcoxson/idevice) by jkcoxson
- [libimobiledevice](https://libimobiledevice.org) ও এর `idevicediagnostics` টুল (ডেটা কোন এন্ট্রি থেকে আসে তার রেফারেন্স হিসেবে)
