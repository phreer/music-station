# 音乐库检查

`music_audit.py` 递归读取音频标签，输出疑似重复歌曲和缺少 title、artist、album 的文件。音乐文件以只读方式打开；报告必须输出到音乐库之外。

## 使用

在项目根目录运行（Python 3.10+）：

```bash
python3 -m venv .venv-music-audit
.venv-music-audit/bin/python -m pip install -r scripts/music-audit-requirements.txt
.venv-music-audit/bin/python scripts/music_audit.py /Volumes/data/music
```

默认报告保存在当前目录的 `music-audit-reports/`，每次创建独立子目录。也可以指定位置，或要求专辑相同：

```bash
.venv-music-audit/bin/python scripts/music_audit.py /Volumes/data/music \
  --output ./music-audit-reports --match-album
```

## 匹配规则

- 默认按 **title + artist** 匹配，跨专辑收录也会列入候选。`--match-album` 则要求三个字段均存在且匹配。
- 比较时使用 OpenCC `t2s` 繁转简、Unicode NFKC 全半角规范化、忽略大小写、去除首尾空白并合并连续空白。报告保留原始简繁体和大小写，标签值去除首尾空白。
- 保留标点及 Live、Remix 等版本文字；内部空格不会全部删除。多值标签忽略值的顺序，不猜测单个字符串里的歌手分隔符。
- 字段不存在、空字符串或仅有空白均算缺失；不使用文件名、album artist 或目录名补齐。`Unknown`、`未知` 等非空占位值不算缺失。
- 缺少 title 或 artist 的文件不参与重复分组，但仍进入缺失标签报告。缺少 album 的文件仍可参与默认分组。
- **候选依据是标签，不是音频指纹或内容哈希**。同名不同录音可能同组，标签不同的同一录音可能漏报；时长和大小供人工比较，不参与分组，也不能据此推断可回收空间。

支持 FLAC、MP3、OGG/OGA、Opus、M4A/MP4、WAV、AIFF、APE、WavPack。跳过符号链接、macOS `._` 文件以及 `.music-station` 目录；其他扩展名不检查。

## 报告

| 文件 | 内容 |
| --- | --- |
| `duplicates.csv` | 每个候选文件一行，使用 group_id 分组，附标签、时长、大小和完整路径 |
| `missing_metadata.csv` | 缺失标签的文件，missing_fields 指明缺少哪些字段 |
| `errors.csv` | 损坏音频、无法识别格式、目录遍历失败等错误 |
| `report.json` | 匹配配置、统计及上述全部结果；多值标签保留为数组 |

CSV 使用 UTF-8 BOM，便于电子表格软件读取中文，多值用 ` | ` 展示。可能被表格软件当成公式的文本加单引号；JSON 保留这些文本原值。

读取失败单独记录，不判定为缺失标签。程序遇到单个读取错误会继续扫描；退出码 `0` 表示完整扫描无读取错误，`2` 表示报告已生成但存在读取错误，`1` 表示依赖、路径、报告写入等运行错误（参数错误为 `2`）。发现重复或缺失标签本身不算运行错误。

依赖文档：[Mutagen](https://mutagen.readthedocs.io/en/latest/api/base.html)、[OpenCC Python](https://github.com/yichen0831/opencc-python)。
