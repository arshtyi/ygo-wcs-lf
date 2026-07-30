# ygo wcs lf

游戏王世界赛 ocg / tcg 禁卡表

## 缓存

构建过程会在 `.cache/ygo-wcs-lf` 中持久化卡名查询结果和卡图。重复执行时会优先复用缓存，避免对卡片 API 和图片服务器发起重复请求。

需要强制刷新时，可以删除其中的 `card-names.json` 或 `card-images`；后续构建会按需重新获取。
