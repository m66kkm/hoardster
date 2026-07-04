import { LayoutGrid, List } from "lucide-react";

interface ViewToggleProps {
  value: "tile" | "detail";
  onChange: (mode: "tile" | "detail") => void;
}

export default function ViewToggle({ value, onChange }: ViewToggleProps) {
  return (
    <div className="view-toggle-group">
      <button
        className={`view-btn ${value === "tile" ? "active" : ""}`}
        onClick={() => onChange("tile")}
      >
        <LayoutGrid size={14} />
        平铺
      </button>
      <button
        className={`view-btn ${value === "detail" ? "active" : ""}`}
        onClick={() => onChange("detail")}
      >
        <List size={14} />
        详细
      </button>
    </div>
  );
}
