import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function useBattery(udid, productType, intervalMs = 5000) {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (!udid) return;
    let alive = true;

    const tick = async () => {
      try {
        const r = await invoke("get_battery", { udid, productType });
        if (alive) { setData(r); setError(null); }
      } catch (e) {
        if (alive) setError(e);
      }
    };

    tick();
    const id = setInterval(tick, intervalMs);
    return () => { alive = false; clearInterval(id); };
  }, [udid, productType, intervalMs]);

  return { data, error };
}