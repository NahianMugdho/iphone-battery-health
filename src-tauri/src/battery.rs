use crate::device::{find_device, make_provider};
use idevice::{services::diagnostics_relay::DiagnosticsRelayClient, IdeviceService};
use plist::{Dictionary, Value};
use serde::Serialize;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatteryReport {
    pub design_capacity: Option<i64>,
    pub nominal_capacity: Option<i64>,
    pub full_charge_capacity: Option<i64>,
    pub remaining_capacity: Option<i64>,
    pub charge_percent: Option<i64>,
    pub cycle_count: Option<i64>,
    pub rated_cycles: u32,
    pub amperage: Option<i64>,
    pub is_charging: bool,
    pub health_nominal: Option<f64>, // H_N
    pub health_full: Option<f64>,    // H_F
    pub capacity_diff: Option<i64>,  // ΔC
    pub cycle_usage: Option<f64>,    // CU%
    pub runtime_h: Option<f64>,
    pub time_to_full_h: Option<f64>,
    pub adapter_watts: Option<f64>,
}

// ধারণা-ভিত্তিক: iPhone15,4 ও তার পরের মডেলে ১০০০, আগেরগুলোতে ৫০০।
// Apple-এর অফিসিয়াল পেজ থেকে মিলিয়ে নেবেন।
fn rated_cycles(product_type: &str) -> u32 {
    let nums: Vec<u32> = product_type
        .trim_start_matches("iPhone")
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();
    match nums.as_slice() {
        [major, minor] if *major >= 16 || (*major == 15 && *minor >= 4) => 1000,
        _ => 500,
    }
}

// plist যত গভীরেই থাকুক, key খুঁজে বের করে
fn find<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Dictionary(d) => d.get(key).or_else(|| d.values().find_map(|x| find(x, key))),
        Value::Array(a) => a.iter().find_map(|x| find(x, key)),
        _ => None,
    }
}

fn int(v: &Value, key: &str) -> Option<i64> {
    match find(v, key)? {
        Value::Integer(i) => i.as_signed().or_else(|| i.as_unsigned().map(|u| u as i64)),
        Value::Real(r) => Some(*r as i64),
        _ => None,
    }
}

fn flag(v: &Value, key: &str) -> bool {
    matches!(find(v, key), Some(Value::Boolean(true)))
}

/// `idevicediagnostics ioregentry AppleSmartBattery`-এর সমতুল্য।
/// CLI আর্গুমেন্টটা class না name হিসেবে পাঠায় তা নিশ্চিত নই, তাই দুটোই চেষ্টা করা হয়।
async fn read_battery_dict(client: &mut DiagnosticsRelayClient) -> Result<Dictionary, String> {
    if let Ok(Some(d)) = client.ioregistry(None, None, Some("AppleSmartBattery")).await {
        return Ok(d);
    }
    match client.ioregistry(None, Some("AppleSmartBattery"), None).await {
        Ok(Some(d)) => Ok(d),
        Ok(None) => Err("AppleSmartBattery এন্ট্রি খালি এসেছে".into()),
        Err(e) => Err(format!("{e:?}")),
    }
}

#[tauri::command]
pub async fn get_battery(udid: String, product_type: String) -> Result<BatteryReport, String> {
    let dev = find_device(&udid).await?;
    let provider = make_provider(&dev)?;
    let mut client = DiagnosticsRelayClient::connect(&provider)
        .await
        .map_err(|e| format!("{e:?}"))?;

    let root = Value::Dictionary(read_battery_dict(&mut client).await?);

    let design = int(&root, "DesignCapacity");
    let nominal = int(&root, "NominalChargeCapacity");
    let full = int(&root, "FullChargeCapacity");
    // কিছু মডেলে RemainingCapacity শতাংশ আসে; ১০০ বা কমকে mAh ধরা হয় না
    let remaining = int(&root, "RemainingCapacity").filter(|r| *r > 100);
    let cycles = int(&root, "CycleCount");
    let amp = int(&root, "Amperage");
    let is_charging = flag(&root, "IsCharging");
    let rated = rated_cycles(&product_type);

    // H_N = Nominal / Design × 100
    let health_nominal = nominal
        .zip(design)
        .filter(|(_, d)| *d > 0)
        .map(|(n, d)| n as f64 / d as f64 * 100.0);
    // H_F = FullCharge / Design × 100
    let health_full = full
        .zip(design)
        .filter(|(_, d)| *d > 0)
        .map(|(f, d)| f as f64 / d as f64 * 100.0);
    // ΔC = FullCharge − Nominal
    let capacity_diff = full.zip(nominal).map(|(f, n)| f - n);
    // CU% = Cycle / Rated × 100
    let cycle_usage = cycles.map(|c| c as f64 / rated as f64 * 100.0);

    // Runtime = Remaining / |I| (শুধু ডিসচার্জের সময়)
    let runtime_h = match (remaining, amp) {
        (Some(r), Some(i)) if i < 0 => Some(r as f64 / i.abs() as f64),
        _ => None,
    };
    // T_full = (N − R) / I_charge (শুধু চার্জের সময়)
    let time_to_full_h = match (nominal, remaining, amp) {
        (Some(n), Some(r), Some(i)) if is_charging && i > 0 && n > r => {
            Some((n - r) as f64 / i as f64)
        }
        _ => None,
    };
    // P (W) = AdapterVoltage(mV) × Current(mA) / 10^6
    let adapter = find(&root, "AdapterDetails").unwrap_or(&root);
    let adapter_watts = int(adapter, "AdapterVoltage")
        .zip(int(adapter, "Current"))
        .map(|(v, i)| v as f64 * i as f64 / 1_000_000.0);

    Ok(BatteryReport {
        design_capacity: design,
        nominal_capacity: nominal,
        full_charge_capacity: full,
        remaining_capacity: remaining,
        charge_percent: int(&root, "CurrentCapacity"),
        cycle_count: cycles,
        rated_cycles: rated,
        amperage: amp,
        is_charging,
        health_nominal,
        health_full,
        capacity_diff,
        cycle_usage,
        runtime_h,
        time_to_full_h,
        adapter_watts,
    })
}