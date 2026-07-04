interface FilterSelectProps {
  value: string;
  onChange: (val: string) => void;
  options: string[];
  allLabel: string;
}

export default function FilterSelect({
  value,
  onChange,
  options,
  allLabel,
}: FilterSelectProps) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="filter-select"
    >
      <option value="">{allLabel}</option>
      {options.map((opt) => (
        <option key={opt} value={opt}>
          {opt}
        </option>
      ))}
    </select>
  );
}
