import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Game, DuplicateGroup, FranchiseGroup } from "../types";

interface UseGamesParams {
  searchVal: string;
  driveVal: string;
  typeVal: string;
  ratingVal: string;
  sortVal: string;
}

export function useGames({ searchVal, driveVal, typeVal, ratingVal, sortVal }: UseGamesParams) {
  const [gamesList, setGamesList] = useState<Game[]>([]);
  const [exactDuplicates, setExactDuplicates] = useState<DuplicateGroup[]>([]);
  const [versionDuplicates, setVersionDuplicates] = useState<DuplicateGroup[]>([]);
  const [franchises, setFranchises] = useState<FranchiseGroup[]>([]);

  // Load games list
  const loadGames = useCallback(async (onlyRepresentatives: boolean, onlyInstalled: boolean) => {
    try {
      const list = await invoke<Game[]>("get_games_list_command", {
        search: searchVal,
        drive: driveVal,
        type: typeVal,
        rating: ratingVal,
        sort: sortVal,
        onlyRepresentatives,
        onlyInstalled
      });
      setGamesList(list);
    } catch (e) {
      console.error("获取游戏列表失败:", e);
    }
  }, [searchVal, driveVal, typeVal, ratingVal, sortVal]);

  // Load duplicate groups
  const loadDuplicates = useCallback(async () => {
    try {
      const exactDups = await invoke<DuplicateGroup[]>("get_duplicates_command", { dupType: "exact" });
      const versionDups = await invoke<DuplicateGroup[]>("get_duplicates_command", { dupType: "version" });

      const searchLower = searchVal.toLowerCase();
      
      const filterGroup = (dups: DuplicateGroup[]) => {
        return dups.map(group => {
          const filteredGames = group.games.filter(game => {
            const nameMatch = game.original_name.toLowerCase().includes(searchLower) ||
                              game.full_path.toLowerCase().includes(searchLower);
            const driveMatch = driveVal ? game.source_path === driveVal : true;
            const typeMatch = typeVal ? game.genres?.includes(typeVal) : true;
            return nameMatch && driveMatch && typeMatch;
          });
          return { ...group, games: filteredGames };
        }).filter(group => group.games.length > 1);
      };

      setExactDuplicates(filterGroup(exactDups));
      
      // Filter out version duplicates that are purely exact duplicates (all games have the same clean_name)
      const filteredVersion = filterGroup(versionDups).filter(group => {
        const uniqueCleanNames = new Set(group.games.map(g => g.clean_name));
        return uniqueCleanNames.size > 1; 
      });
      setVersionDuplicates(filteredVersion);
    } catch (e) {
      console.error("获取重复项失败:", e);
    }
  }, [searchVal, driveVal, typeVal]);

  // Load franchises
  const loadFranchises = useCallback(async () => {
    try {
      const frs = await invoke<FranchiseGroup[]>("get_franchises_command");
      const searchLower = searchVal.toLowerCase();
      const filteredFrs = frs.map(group => {
        const filteredGames = group.games.filter(game => {
          const nameMatch = game.original_name.toLowerCase().includes(searchLower) ||
                            game.full_path.toLowerCase().includes(searchLower);
          const driveMatch = driveVal ? game.source_path === driveVal : true;
          const typeMatch = typeVal ? game.genres?.includes(typeVal) : true;
          return nameMatch && driveMatch && typeMatch;
        });
        return { ...group, games: filteredGames };
      }).filter(group => group.games.length > 0);

      setFranchises(filteredFrs);
    } catch (e) {
      console.error("获取系列关系失败:", e);
    }
  }, [searchVal, driveVal, typeVal]);

  return {
    gamesList,
    exactDuplicates,
    versionDuplicates,
    franchises,
    loadGames,
    loadDuplicates,
    loadFranchises
  };
}
