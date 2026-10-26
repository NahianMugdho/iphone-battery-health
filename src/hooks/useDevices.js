import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export default function useDevices() {
  const [devices, setDevices] = useState([]);

  useEffect(() => {
    invoke("list_devices").then(setDevices).catch(() => setDevices([]));
    const unlisten = listen("devices-changed", (e) => setDevices(e.payload));
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  return devices;
}