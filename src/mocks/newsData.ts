export interface NewsItem {
  id: number;
  category: "情报" | "特惠" | "更新" | "Repack" | "限免";
  title: string;
  source: string;
  time: string;
  readTime: string;
  summary: string;
  details: string;
}

export const NEWS_DATA: NewsItem[] = [
  {
    id: 1,
    category: "情报",
    title: "Rockstar 确认：《GTA 6》PC版将延期发售，优先主机平台",
    source: "IGN China",
    time: "2小时前",
    readTime: "3 分钟阅读",
    summary: "Rockstar Games 在最新的财报分析会议中确认，《侠盗猎车手 6》（GTA VI）的 PC 版本开发正在稳步推进，但其发售时间窗口确定将落后于 PS5 和 Xbox Series X/S 版本。",
    details: "财报会议透露，首发主机平台是为了确保首发表现与主机机能调优的最大化。PC版将在主机版发售后的数个月内推出，并会搭载更先进的光线追踪与画质选项。开发团队承诺PC版不会粗制滥造，将有极致的画质体验。"
  },
  {
    id: 2,
    category: "特惠",
    title: "Steam 2026 夏季特卖正式开启：数千款游戏低至2折",
    source: "Steam Store",
    time: "5小时前",
    readTime: "5 分钟阅读",
    summary: "Steam 年度重磅盛典\"夏季特卖\"今日凌晨拉开帷幕，本年度特卖包含数千款史低折扣与各种集卡活动。《幻兽帕鲁》、《黑神话：悟空》迎来新史低折扣。",
    details: "本次夏促从7月4日持续到7月18日。除了海量大作折扣外，Steam点数商店也同步更新了夏日主题的动态头像框、个人资料背景等。同时，每天浏览探索队列可免费获得一张夏促限定集换式卡牌。"
  },
  {
    id: 3,
    category: "Repack",
    title: "FitGirl Repacks 更新：多线程解压算法大幅优化，安装提速30%",
    source: "FitGirl Official",
    time: "1天前",
    readTime: "2 分钟阅读",
    summary: "著名游戏打包姬 FitGirl 发布官方通告，针对最新的高核心数 CPU（如 AMD Ryzen 9 9950X, Intel Core i9-14900K）更新了多线程打包解压核心，平均安装解压速度提升 30%。",
    details: "新的解压算法解决了在 16 核及以上处理器上部分解压引擎（如 QuickBMS 和 Razor1911 解压模块）CPU 占用率不均的问题。更新后的安装器会自动检测系统核心数并合理分摊 IO 负载，大幅降低固态硬盘与处理器的等待延迟。"
  },
  {
    id: 4,
    category: "更新",
    title: "《怪物猎人：荒野》发布 1.05 性能热修复补丁",
    source: "Capcom",
    time: "2天前",
    readTime: "4 分钟阅读",
    summary: "Capcom 针对《怪物猎人：荒野》（Monster Hunter Wilds）发布了紧急热修复 1.05 补丁，主要解决了多人联机状态下偶发性掉线、着色器编译卡顿，以及部分高端显卡上的帧率骤降问题。",
    details: "除了修复崩溃问题外，1.05 补丁还优化了荒野地图在大风暴天气下的粒子渲染开销，并对 Steam Deck 进行了兼容性微调。玩家在重新连接网络后，游戏会自动在后台下载约 1.2 GB 的更新数据。"
  },
  {
    id: 5,
    category: "限免",
    title: "Epic 游戏商城本周限免：《消逝的光芒 终极版》免费领",
    source: "Epic Games",
    time: "3天前",
    readTime: "2 分钟阅读",
    summary: "Epic Games Store 开启本周限时免费领取活动，玩家可以在7月11日前免费将经典丧尸跑酷动作大作《消逝的光芒 终极版》（Dying Light Definitive Edition）永久入库。",
    details: "《消逝的光芒 终极版》包含基础游戏、4个主打 DLC 和 17 个皮肤包，是体验该作的最佳合集。下周限免游戏预告为赛博朋克风动作砍杀游戏《幽灵行者》（Ghostrunner）。"
  }
];
