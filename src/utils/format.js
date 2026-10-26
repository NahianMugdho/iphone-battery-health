export const pct = (v) => (v == null ? "—" : `${v.toFixed(2)}%`);

export const hm = (h) => {
  if (h == null) return "—";
  const m = Math.round(h * 60);
  return `${Math.floor(m / 60)} h ${m % 60} min`;
};

export const friendlyError = (e) =>
  String(e).includes("PairRecordData")
    ? "ফোনে Trust দেওয়া নেই। ফোন আনলক করে 'Trust This Computer' চাপুন।"
    : String(e);