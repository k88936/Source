**Asset（资产）**：
Cloudreve 中的一张照片或一段视频文件，是时间线与相册的最小单元。 _Avoid_: Photo（当也指视频时）、Item、Media

**Album（相册）**：
用户当作相册来浏览和整理的一个 Cloudreve 目录。 _Avoid_: Collection、Folder（当指相册时）

**Folder（目录）**：
Cloudreve 的容器原语。v1 中每个 Album 对应一个 Folder，但并非所有 Folder 都是 Album。 _Avoid_: Directory

**Library Root（图库根）**：
用户指定、被 App 视为照片库的那棵 Cloudreve 目录子树。 _Avoid_: Sync root、Base folder

**Timeline（时间线）**：
跨 Album、按拍摄时间倒序排列的 Asset 视图，由本地 Catalog 支撑。 _Avoid_: Photos feed、Gallery

**Catalog（目录）**：
App 本地维护的已知 Asset 与 Album 的派生清单，可随时从 Cloudreve 重建。 _Avoid_: Cache（Catalog
是读取模型，不是可丢弃缓存）、Index、Database

**Curation（策展）**：
用户附加在 Album 或 Asset 上的、文件本身不包含的选择，例如封面与收藏。 _Avoid_: Tag、Label、Favorite（当指整体概念时）

**Capture Time（拍摄时间）**：
Asset 被拍摄的时间；优先取 Cloudreve 的 `exif:taken_at`，缺失时由客户端解析或降级推断。 _Avoid_: Date、Created
time（那是创建/上传时间）

**Backup（备份）**：
把设备相机胶卷中的新媒体自动上传到 Library Root 的过程，与手动 Upload 区分。 _Avoid_: Sync（Sync 指目录与元数据的同步）、Import

**Source of Truth（真相源）**：
Cloudreve 中的文件树与文件元数据。Catalog 与缩略图缓存都只是它的派生视图。 _Avoid_: Server、Backend（那指实现，不是权威）
