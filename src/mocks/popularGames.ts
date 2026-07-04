export interface PopularGame {
  id: number;
  name: string;
  enName: string;
  genre: string;
  rating: string;
  reviews: string;
  release: string;
  desc: string;
  gradient: string;
  bgIcon: string;
}

export const POPULAR_GAMES: PopularGame[] = [
  {
    id: 1,
    name: "黑神话：悟空",
    enName: "Black Myth: Wukong",
    genre: "动作角色扮演, 魂动作",
    rating: "96%",
    reviews: "Overwhelmingly Positive",
    release: "2024-08-20",
    desc: "一款以中国神话为背景的动作角色扮演游戏。故事取材于中国古典小说《西游记》，你将扮演一位\"天命人\"，踏上充满危险与惊奇的西行之路。",
    gradient: "linear-gradient(135deg, #1e1b4b 0%, #311005 100%)",
    bgIcon: "🐒"
  },
  {
    id: 2,
    name: "艾尔登法环：黄金树幽影",
    enName: "Elden Ring: Shadow of the Erdtree",
    genre: "开放世界, 动作RPG",
    rating: "91%",
    reviews: "Very Positive",
    release: "2024-06-21",
    desc: "《艾尔登法环》的全新大型资料片。以\"幽影之地\"为舞台，伴随着多种新增要素，展开全新冒险，探寻梅瑟莫与米凯拉的足迹。",
    gradient: "linear-gradient(135deg, #1e1b4b 0%, #172554 100%)",
    bgIcon: "💍"
  },
  {
    id: 3,
    name: "怪物猎人：荒野",
    enName: "Monster Hunter Wilds",
    genre: "动作, 联机合作",
    rating: "89%",
    reviews: "Very Positive",
    release: "2025-02-28",
    desc: "《怪物猎人》系列最新作。在生态环境瞬息万变的荒野中，体验极致的狩猎快感。全新动作设计和无缝连接的世界等你探索。",
    gradient: "linear-gradient(135deg, #022c22 0%, #1e1b4b 100%)",
    bgIcon: "🦖"
  },
  {
    id: 4,
    name: "哈迪斯 2",
    enName: "Hades II",
    genre: "动作Roguelike, 砍杀",
    rating: "94%",
    reviews: "Very Positive",
    release: "2024-05-06",
    desc: "在冥界之外浴血奋战，驾驭古老的巫术，与时间之神克罗诺斯正面对决。超人气的首款续作，带来全新的Roguelike地牢挑战。",
    gradient: "linear-gradient(135deg, #2e1065 0%, #581c87 100%)",
    bgIcon: "💀"
  },
  {
    id: 5,
    name: "赛博朋克2077：往日之影",
    enName: "Cyberpunk 2077: Phantom Liberty",
    genre: "开放世界, 角色扮演",
    rating: "92%",
    reviews: "Very Positive",
    release: "2023-09-26",
    desc: "谍战悬疑风格的全新剧情资料片。扮演义体强化的雇民兵 V，深入狗镇，与联络人李德一起解救新美国总统，展开危险的谍报风云。",
    gradient: "linear-gradient(135deg, #4c0519 0%, #1e1b4b 100%)",
    bgIcon: "🌆"
  },
  {
    id: 6,
    name: "文明 7",
    enName: "Civilization VII",
    genre: "策略, 历史, 4X",
    rating: "85%",
    reviews: "Mostly Positive",
    release: "2025-02-11",
    desc: "屡获殊荣的传奇策略游戏系列最新作。执掌人类历史的轮盘，选择独特的领袖与文明，建设经得起时间考验的伟大帝国。",
    gradient: "linear-gradient(135deg, #3b0764 0%, #1e1b4b 100%)",
    bgIcon: "🏛"
  }
];
