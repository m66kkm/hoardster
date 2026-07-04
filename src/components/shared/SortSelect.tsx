interface SortOption {
  value: string;
  label: string;
}

interface SortSelectProps {
  value: string;
  onChange: (val: string) => void;
  options: SortOption[];
  defaultLabel?: string;
}

export default function SortSelect({
  value,
  onChange,
  options,
  defaultLabel,
}: SortSelectProps) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="filter-select"
    >
      <option value="">{defaultLabel || "默认排序"}</option>
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}
