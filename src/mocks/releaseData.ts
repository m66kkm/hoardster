export interface ReleaseItem {
  id: number;
  releaseName: string;
  group: "RUNE" | "TENOKE" | "FLT" | "FitGirl" | "DODI";
  type: "Scene Crack" | "Repack" | "Update";
  size: string;
  date: string;
  nfoContent: string;
}

export const RUNE_ASCII = `
 ██████╗ ██╗   ██╗███╗   ██╗███████╗
 ██╔══██╗██║   ██║████╗  ██║██╔════╝
 ██████╔╝██║   ██║██╔██╗ ██║█████╗  
 ██╔══██╗██║   ██║██║╚██╗██║██╔══╝  
 ██║  ██║╚██████╔╝██║ ╚████║███████╗
 ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝
      - S  C  E  N  E     R  U  L  E  S -
`;

export const TENOKE_ASCII = `
  ████████╗███████╗███╗   ██╗ ██████╗ ██╗  ██╗███████╗
  ╚══██╔══╝██╔════╝████╗  ██║██╔═══██╗██║ ██╔╝██╔════╝
     ██║   █████╗  ██╔██╗ ██║██║   ██║█████╔╝ █████╗  
     ██║   ██╔══╝  ██║╚██╗██║██║   ██║██╔═██╗ ██╔══╝  
     ██║   ███████╗██║ ╚████║╚██████╔╝██║  ██╗███████╗
     ╚═╝   ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
`;

export const FLT_ASCII = `
  ███████╗██╗     ████████╗
  ██╔════╝██║     ╚══██╔══╝
  █████╗  ██║        ██║   
  ██╔══╝  ██║        ██║   
  ██║     ███████╗   ██║   
  ╚═╝     ╚══════╝   ╚═╝   
       -  F A I R L I G H T  -
`;

export const RELEASES: ReleaseItem[] = [
  {
    id: 1,
    releaseName: "Black.Myth.Wukong.v1.0.8.14920-RUNE",
    group: "RUNE",
    type: "Scene Crack",
    size: "118.4 GB",
    date: "2024-08-20",
    nfoContent: `${RUNE_ASCII}
RELEASE NOTES:
- Game Version: v1.0.8.14920 (Steam)
- Protection: Steam + Denuvo (Bypassed)
- Language: English, Simplified Chinese, Traditional Chinese, etc.

INSTALLATION:
1. Extract release
2. Mount ISO
3. Install the game
4. Copy crack from the RUNE folder
5. Play!`
  },
  {
    id: 2,
    releaseName: "Elden.Ring.Shadow.of.the.Erdtree-RUNE",
    group: "RUNE",
    type: "Scene Crack",
    size: "82.5 GB",
    date: "2024-06-21",
    nfoContent: `${RUNE_ASCII}
RELEASE NOTES:
- Game Version: v1.12 (Shadow of the Erdtree DLC Included)
- Protection: Steam (Emulator Included)
- Multi-language support

INSTALLATION:
1. Burn or mount the image
2. Install the game
3. Copy crack
4. Enjoy!`
  },
  {
    id: 3,
    releaseName: "Hades.II.v0.29340-TENOKE",
    group: "TENOKE",
    type: "Scene Crack",
    size: "4.1 GB",
    date: "2024-05-15",
    nfoContent: `${TENOKE_ASCII}
RELEASE NOTES:
- Game Version: v0.29340 (Early Access)
- Protection: Steam
- Cracked by TENOKE team

INSTALLATION:
1. Unrar
2. Install game
3. Copy contents from TENOKE folder to game directory
4. Play game!`
  },
  {
    id: 4,
    releaseName: "Ghost.of.Tsushima.Directors.Cut-FLT",
    group: "FLT",
    type: "Scene Crack",
    size: "58.9 GB",
    date: "2024-05-16",
    nfoContent: `${FLT_ASCII}
RELEASE NOTES:
- Game Version: v1.0.4.0516 (Director's Cut)
- Protection: Steam + SDK
- Scene crack by FairLight

INSTALLATION:
1. Mount or burn
2. Run Setup.exe
3. Play the game!`
  },
  {
    id: 5,
    releaseName: "Civilization.VII-FitGirl.Repack",
    group: "FitGirl",
    type: "Repack",
    size: "24.1 GB",
    date: "2025-02-12",
    nfoContent: `==========================================
   F I T G I R L   R E P A C K S
==========================================
GAME: Civilization VII
REPACK SIZE: 24.1 GB (Original: 35.8 GB)
LOSSLESS: Yes, nothing ripped or re-encoded.

REPACK FEATURES:
- Based on Civilization.VII.v1.0.3-RUNE release
- Game version: v1.0.3.4902
- Language selection during setup
- Installation takes 10-25 mins depending on CPU`
  },
  {
    id: 6,
    releaseName: "Monster.Hunter.Wilds-DODI.Repack",
    group: "DODI",
    type: "Repack",
    size: "92.4 GB",
    date: "2025-03-01",
    nfoContent: `==========================================
   D O D I   R E P A C K S
==========================================
GAME: Monster Hunter Wilds
REPACK SIZE: 92.4 GB (Original: 140.1 GB)
CRACK: RUNE Emulator (Steam)

REPACK FEATURES:
- Based on Steam-Rip files + RUNE crack
- All updates pre-installed up to v1.05
- High resolution textures included
- Selective download support`
  }
];
