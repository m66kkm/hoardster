// Types corresponding to Rust structs
export interface Game {
  id?: number;
  original_name: string;
  clean_name: string;
  base_name: string;
  type: string;
  source_path: string;
  full_path: string;
  size: string;
  size_bytes: number;
  created: string;
  is_exact_dup: boolean;
  is_version_dup: boolean;
  is_representative: boolean;
  
  // Steam Cache
  appid?: number;
  name?: string;
  local_cover?: string;
  review_score_desc?: string;
  positive_percent?: number;
  total_reviews?: number;
  release_date?: string;
  genres?: string;
}

export interface DuplicateGroup {
  name: string;
  games: Game[];
}

export interface FranchiseGroup {
  prefix: string;
  games: Game[];
}

export interface StatsSummary {
  total_scan: number;
  unique_games: number;
  franchise_count: number;
  exact_dups: number;
  version_dups: number;
}

export interface GenreStat {
  name: string;
  count: number;
}

export interface RatingStat {
  name: string;
  count: number;
}

export interface ProgressPayload {
  step: string;
  message: string;
  current: number;
  total: number;
}

export interface Torrent1337x {
  id?: number;
  torrent_id: string;
  name: string;
  url: string;
  seeds: number;
  leeches: number;
  date: string;
  size: string;
  uploader: string;
  uploader_url: string;
}
