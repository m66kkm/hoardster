import { useTranslation } from "react-i18next";
import { Search } from "lucide-react";

interface SearchBoxProps {
  value: string;
  onChange: (val: string) => void;
  placeholder?: string;
}

export default function SearchBox({
  value,
  onChange,
  placeholder,
}: SearchBoxProps) {
  const { t } = useTranslation();

  return (
    <div className="search-box">
      <Search className="search-icon" size={16} />
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="search-input"
        placeholder={placeholder ?? t("filterSearch")}
      />
    </div>
  );
}
