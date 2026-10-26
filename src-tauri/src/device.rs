use idevice::{
    lockdown::LockdownClient,
    provider::UsbmuxdProvider,
    usbmuxd::{UsbmuxdAddr, UsbmuxdConnection, UsbmuxdDevice},
    IdeviceService,
};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub udid: String,
    pub name: String,
    pub product_type: String,
    pub ios_version: String,
}

fn err<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}

/// প্রতিবার নতুন usbmuxd কানেকশন খোলা হয়, সহজ ও নিরাপদ।
pub async fn connect_mux() -> Result<UsbmuxdConnection, String> {
    UsbmuxdConnection::default().await.map_err(err)
}

/// to_provider-এর আর্গুমেন্ট কোনো ভার্সনে বদলালে শুধু এখানে ঠিক করবেন।
pub fn make_provider(dev: &UsbmuxdDevice) -> Result<UsbmuxdProvider, String> {
    let addr = UsbmuxdAddr::from_env_var().map_err(err)?;
    Ok(dev.to_provider(addr, "iphone-battery"))
}

pub async fn find_device(udid: &str) -> Result<UsbmuxdDevice, String> {
    let mut mux = connect_mux().await?;
    let devs = mux.get_devices().await.map_err(err)?;
    devs.into_iter()
        .find(|d| d.udid == udid)
        .ok_or_else(|| format!("ডিভাইস পাওয়া যায়নি: {udid}"))
}

async fn lockdown_string(lockdown: &mut LockdownClient, key: &str) -> String {
match lockdown.get_value(Some(key), None).await {
        Ok(v) => v.as_string().unwrap_or_default().to_string(),
        Err(_) => String::new(),
    }
}

async fn read_info(dev: &UsbmuxdDevice) -> Result<DeviceInfo, String> {
    let provider = make_provider(dev)?;
    let mut lockdown = LockdownClient::connect(&provider).await.map_err(err)?;
    Ok(DeviceInfo {
        udid: dev.udid.clone(),
        name: lockdown_string(&mut lockdown, "DeviceName").await,
        product_type: lockdown_string(&mut lockdown, "ProductType").await,
        ios_version: lockdown_string(&mut lockdown, "ProductVersion").await,
    })
}

/// Trust না দেওয়া ডিভাইস বাদ যায়, পুরো লিস্ট ফেল করে না।
pub async fn list_all() -> Result<Vec<DeviceInfo>, String> {
    let mut mux = connect_mux().await?;
    let devs = mux.get_devices().await.map_err(err)?;
    let mut out = Vec::new();
    for d in &devs {
        if let Ok(info) = read_info(d).await {
            out.push(info);
        }
    }
    Ok(out)
}



pub async fn list_udids() -> Result<Vec<String>, String> {
    let mut mux = connect_mux().await?;
    let devs = mux.get_devices().await.map_err(err)?;
    let mut ids: Vec<String> = devs.iter().map(|d| d.udid.clone()).collect();
    ids.sort();
    Ok(ids)
}


#[tauri::command]
pub async fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    list_all().await
}