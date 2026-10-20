#!/usr/bin/env python3
"""Read-only music metadata audit; see MUSIC_AUDIT.md for matching rules."""

import argparse
import csv
import json
import os
from pathlib import Path
import sys
import tempfile
import unicodedata
from collections import defaultdict
from datetime import datetime, timezone


EXTENSIONS = {".flac", ".mp3", ".ogg", ".oga", ".opus", ".m4a", ".mp4",
              ".wav", ".aif", ".aiff", ".ape", ".wv"}
FIELDS = ("title", "artist", "album")
TRACK_COLUMNS = ["path", *FIELDS, "duration_seconds", "size_bytes", "missing_fields"]


def normalize(value, converter):
    value = unicodedata.normalize("NFKC", value)
    return " ".join(converter.convert(value).casefold().split())


def tag_values(tags, field, id3_type):
    if tags is None:
        return []
    if isinstance(tags, id3_type):
        # WAVE/AIFF expose native ID3 frames even with easy=True.
        key = {"title": "TIT2", "artist": "TPE1", "album": "TALB"}[field]
        values = [text for frame in tags.getall(key) for text in frame.text]
    else:
        values = tags.get(field, [])
    if isinstance(values, str):
        values = [values]
    return [str(value).strip() for value in values if str(value).strip()]


def read_track(path, mutagen, id3_type):
    # Explicitly open in read-only mode, including when a parser uses file objects.
    with path.open("rb") as source:
        audio = mutagen.File(source, easy=True)
        if audio is None:
            raise ValueError("无法识别音频格式")
        metadata = {field: tag_values(audio.tags, field, id3_type) for field in FIELDS}
        duration = getattr(audio.info, "length", None)
        return {
            "path": str(path),
            **metadata,
            "duration_seconds": round(duration, 3) if duration is not None else None,
            "size_bytes": os.fstat(source.fileno()).st_size,
            "missing_fields": [field for field in FIELDS if not metadata[field]],
        }


def scan(library, converter, match_fields, mutagen, id3_type):
    groups = defaultdict(list)
    missing = []
    errors = []
    counts = {"audio_files": 0, "readable_files": 0, "duplicate_eligible_files": 0,
              "ignored_files": 0, "skipped_symlinks": 0}

    def walk_error(error):
        errors.append({"path": str(error.filename or library), "error": str(error)})

    for directory, subdirs, filenames in os.walk(library, onerror=walk_error):
        kept = []
        for name in sorted(subdirs):
            if (Path(directory) / name).is_symlink():
                counts["skipped_symlinks"] += 1
            elif name != ".music-station":
                kept.append(name)
        subdirs[:] = kept
        for name in sorted(filenames):
            path = Path(directory) / name
            if path.is_symlink():
                counts["skipped_symlinks"] += 1
                continue
            if name.startswith("._") or path.suffix.lower() not in EXTENSIONS:
                counts["ignored_files"] += 1
                continue
            counts["audio_files"] += 1
            try:
                track = read_track(path, mutagen, id3_type)
            except Exception as error:
                # A broken file must not prevent the rest of the library being audited.
                errors.append({"path": str(path),
                               "error": f"{type(error).__name__}: {error}"})
            else:
                counts["readable_files"] += 1
                if track["missing_fields"]:
                    missing.append(track)
                key = tuple(tuple(sorted({normalize(value, converter)
                                          for value in track[field]}))
                            for field in match_fields)
                if all(key):
                    counts["duplicate_eligible_files"] += 1
                    groups[key].append(track)
            if counts["audio_files"] % 500 == 0:
                print(f"已扫描 {counts['audio_files']} 个音频文件…", file=sys.stderr)

    duplicates = [
        {"group_id": index, "normalized": dict(zip(match_fields, key)), "tracks": tracks}
        for index, (key, tracks) in enumerate(
            ((key, tracks) for key, tracks in sorted(groups.items()) if len(tracks) > 1), 1
        )
    ]
    counts.update({
        "duplicate_groups": len(duplicates),
        "files_in_duplicate_groups": sum(len(group["tracks"]) for group in duplicates),
        "missing_metadata_files": len(missing),
        "missing_by_field": {field: sum(field in track["missing_fields"] for track in missing)
                             for field in FIELDS},
        "errors": len(errors),
    })
    return counts, duplicates, missing, errors


def csv_cell(value):
    if isinstance(value, (list, tuple)):
        value = " | ".join(value)
    # Keep spreadsheet applications from interpreting metadata as formulas.
    if isinstance(value, str) and value.lstrip().startswith(("=", "+", "-", "@")):
        return "'" + value
    return value


def write_csv(path, columns, rows):
    with path.open("w", encoding="utf-8-sig", newline="") as output:
        writer = csv.DictWriter(output, fieldnames=columns, extrasaction="ignore")
        writer.writeheader()
        for row in rows:
            writer.writerow({key: csv_cell(value) for key, value in row.items()})


def write_reports(output, report):
    output.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix=datetime.now().strftime("%Y%m%d-%H%M%S-"), dir=output))
    with (run / "report.json").open("w", encoding="utf-8") as destination:
        json.dump(report, destination, ensure_ascii=False, indent=2)
        destination.write("\n")
    write_csv(run / "duplicates.csv", ["group_id", *TRACK_COLUMNS],
              ({"group_id": group["group_id"], **track}
               for group in report["duplicates"] for track in group["tracks"]))
    write_csv(run / "missing_metadata.csv", TRACK_COLUMNS, report["missing_metadata"])
    write_csv(run / "errors.csv", ["path", "error"], report["errors"])
    return run


def main():
    parser = argparse.ArgumentParser(description="只读扫描音乐库：疑似重复歌曲及缺失标签")
    parser.add_argument("library", type=Path, help="音乐库目录，例如 /Volumes/data/music")
    parser.add_argument("--output", type=Path, default=Path("music-audit-reports"),
                        help="报告根目录（默认 ./music-audit-reports，每次创建独立子目录）")
    parser.add_argument("--match-album", action="store_true",
                        help="只有 title、artist、album 都相同才算疑似重复")
    args = parser.parse_args()
    library = args.library.expanduser().resolve()
    output = args.output.expanduser().resolve()
    if not library.is_dir():
        parser.error(f"音乐库目录不存在或无法访问：{library}")
    if output == library or library in output.parents:
        parser.error("请将报告输出到音乐库之外，保持音乐库只读")
    try:
        import mutagen
        from mutagen.id3 import ID3
        from opencc import OpenCC
    except ImportError:
        parser.exit(1, "缺少依赖。请运行：python -m pip install -r scripts/music-audit-requirements.txt\n")

    match_fields = FIELDS if args.match_album else FIELDS[:2]
    print(f"开始扫描：{library}", file=sys.stderr)
    counts, duplicates, missing, errors = scan(library, OpenCC("t2s"), match_fields, mutagen, ID3)
    report = {
        "library": str(library), "generated_at": datetime.now(timezone.utc).isoformat(),
        "matching": {"fields": match_fields, "chinese_conversion": "t2s",
                     "unicode_normalization": "NFKC", "case_insensitive": True,
                     "collapse_whitespace": True, "audio_fingerprinting": False},
        "summary": counts, "duplicates": duplicates, "missing_metadata": missing, "errors": errors,
    }
    run = write_reports(output, report)
    print(f"扫描音频：{counts['audio_files']}；成功读取：{counts['readable_files']}")
    print(f"疑似重复：{counts['duplicate_groups']} 组，涉及 {counts['files_in_duplicate_groups']} 个文件")
    print(f"缺少标签：{counts['missing_metadata_files']} 个文件；读取错误：{counts['errors']}")
    print(f"报告目录：{run}")
    return 2 if errors else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit("扫描已中断，未生成完整报告。")
    except OSError as error:
        sys.exit(f"扫描或写入报告失败：{error}")
