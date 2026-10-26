import useDevices from "./hooks/useDevices";
import BatteryCard from "./components/BatteryCard";
import "./App.css";

export default function App() {
  const devices = useDevices();

  return (
    <main>
      <h1>iPhone Battery Health</h1>
      {devices.length === 0 && (
        <p className="hint">
          কোনো ডিভাইস পাওয়া যায়নি। USB ক্যাবল লাগান, ফোন আনলক করে "Trust" দিন।
          Windows-এ Apple Mobile Device Service, Linux-এ usbmuxd চালু থাকতে হবে।
        </p>
      )}
      {devices.map((d) => (
        <BatteryCard key={d.udid} device={d} />
      ))}
    </main>
  );
}