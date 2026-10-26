import useBattery from "../hooks/useBattery";
import { pct, hm, friendlyError } from "../utils/format";

export default function BatteryCard({ device }) {
  const { data: b, error } = useBattery(device.udid, device.productType);

  if (error && !b) return <div className="card err">{friendlyError(error)}</div>;
  if (!b) return <div className="card">লোড হচ্ছে…</div>;

  const health = b.healthNominal ?? b.healthFull;
  const barClass = health == null ? "" : health >= 80 ? "good" : health >= 70 ? "warn" : "bad";

  const rows = [
    ["Battery Health (Nominal)", pct(b.healthNominal)],
    ["Health Estimate (Full Charge)", pct(b.healthFull)],
    ["Capacity Difference", b.capacityDiff == null ? "—" : `${b.capacityDiff} mAh`],
    ["Design / Nominal", `${b.designCapacity ?? "—"} / ${b.nominalCapacity ?? "—"} mAh`],
    ["Cycle Usage", `${b.cycleCount ?? "—"} / ${b.ratedCycles} (${pct(b.cycleUsage)})`],
    ["Runtime", hm(b.runtimeH)],
    ["Time to Full", hm(b.timeToFullH)],
    ["Adapter Power", b.adapterWatts == null ? "—" : `${b.adapterWatts.toFixed(1)} W`],
  ];

  return (
    <div className="card">
      <div className="head">
        <div>
          <h3>{device.name}</h3>
          <small>{device.productType} · iOS {device.iosVersion}</small>
        </div>
        <div className="charge">
          {b.chargePercent ?? "—"}% {b.isCharging ? "⚡" : ""}
        </div>
      </div>

      <div className="bar">
        <div className={`fill ${barClass}`} style={{ width: `${Math.min(health ?? 0, 100)}%` }} />
      </div>
      <p className="big">{pct(health)} <small>Battery Health</small></p>

      <table>
        <tbody>
          {rows.map(([k, v]) => (
            <tr key={k}>
              <td>{k}</td>
              <td><b>{v}</b></td>
            </tr>
          ))}
        </tbody>
      </table>

      {error && <p className="err">{friendlyError(error)}</p>}
    </div>
  );
}