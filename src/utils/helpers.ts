// Helper for steam ratings color
export const getRatingColorClass = (desc?: number | string): string => {
  if (desc === undefined || desc === null || desc === 0 || desc === "0") return "rating-none";
  
  const score = typeof desc === 'string' ? parseInt(desc, 10) : desc;
  
  if (score === 8 || score === 9) {
    return "rating-high";
  }
  if (score === 6 || score === 7) {
    return "rating-good";
  }
  if (score === 5) {
    return "rating-mixed";
  }
  if (score >= 1 && score <= 4) {
    return "rating-bad";
  }
  return "rating-none";
};

export const getReviewScoreText = (t: any, desc?: number | string): string => {
  if (desc === undefined || desc === null) return t("reviewScore_0");
  if (typeof desc === "number") return t(`reviewScore_${desc}`);
  
  // Backward compatibility for old string data before migration
  const num = parseInt(desc as string, 10);
  if (!isNaN(num)) return t(`reviewScore_${num}`);
  return desc as string;
};

// Steam store URL resolver
export const getSteamStoreUrl = (appid?: number, baseName?: string, fallbackTitle?: string): string => {
  if (appid && appid > 0) {
    return `https://store.steampowered.com/app/${appid}/`;
  }
  const term = baseName || fallbackTitle || "";
  return `https://store.steampowered.com/search/?term=${encodeURIComponent(term)}`;
};

// Cover image resolver
export const getCoverUrl = (localCover?: string): string | null => {
  if (localCover) {
    // Use the custom cover protocol URL format for Tauri v2 on Windows
    const filename = localCover.replace("covers/", "");
    return `http://cover.localhost/${filename}`;
  }
  return null;
};

export const getTypeBadgeClass = (type?: string): string => {
  if (type === "Installed" || type === "Directory") return "badge-dir";
  if (type === "Archive") return "badge-ver";
  return "badge-iso";
};

// Fallback gradient generator
export const getGradientsForName = (name: string): string => {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const grads = [
    "linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%)", // Dark Indigo
    "linear-gradient(135deg, #1e1b4b 0%, #311042 100%)", // Purple/Dark
    "linear-gradient(135deg, #311042 0%, #4c0519 100%)", // Rose/Maroon
    "linear-gradient(135deg, #022c22 0%, #064e3b 100%)", // Emerald Deep
    "linear-gradient(135deg, #1c1917 0%, #292524 100%)", // Stone
    "linear-gradient(135deg, #083344 0%, #164e63 100%)"  // Cyan Deep
  ];
  const index = Math.abs(hash) % grads.length;
  return grads[index];
};
