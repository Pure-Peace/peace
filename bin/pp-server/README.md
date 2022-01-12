## A Fast osu! pp calculator web api written in Rust

It is also the PP server of **[Peace](https://github.com/Pure-Peace/peace)**.

**Pure-Rust** pp calculator based on [peace-performance](https://github.com/Pure-Peace/peace-performance) and [rosu-pp](https://github.com/MaxOhn/rosu-pp).

### Features

- **Common**:
  - **request with you like: beatmap md5, beatmap id, beatmapset id + file name**
  - **beatmap cache**
  - **preload beatmaps** (WARING: May cause insufficient memory, if the number of maps is large enough)
  - **calculate beatmap MD5**
  - **auto request, download beatmap from osu!api**
  - **raw pp info: aim, spd, acc, str.**
  - **acc list: 95, 98, 99, 100 (request with &acc_list=1)**
  - **Oppai? Or a custom algorithm**
  - **auto-pp-recalculate (peace)**
    - If pp calculation fails (such as restarting pp-server), just save task to redis in the format of "`calc:{table(mode)}:{score_id}:{player_id}`":"`md5=xxx&mods=xx&mode=xx&n300=xx`". pp-server will auto recalculate these tasks, and notify peace to update the stats of these players.
.
  - **pp calculate**:
    - osu!Standard
    - Catch the beat
    - Taiko
    - Mainia
- (**Default Enabled**) feature **with_peace**:
  - beatmap database (needs setup with [Peace](https://github.com/Pure-Peace/Peace/tree/main/sql))
  - auto pp recalculate
  - How to **enable**?
    - `Cargo.toml` features Set `default = ["with_peace"]`
    - **Will run as a pp server of Peace**
  - How to **disable**?
    - Set features `default = ["peace-objects/no_database"]`
    - **Not use peace database, run pp-server independently.**

## Examples

**with md5**

*Common*

```
/api/calc?md5=ccb1f31b5eeaf26d40f8c905293efc03
```

```json
{
  "acc_list": null,
  "message": "done",
  "mode": 0,
  "mods": 0,
  "pp": 522.0230712890625,
  "raw": {
    "acc": 128.93072509765625,
    "aim": 255.26805114746094,
    "spd": 127.87066650390625,
    "str": 0,
    "total": 522.0230712890625
  },
  "stars": 7.084656715393066,
  "status": 1
}
```

*Simple*

```
/api/calc?md5=ccb1f31b5eeaf26d40f8c905293efc03&simple=1
```

```json
{
  "acc_list": null,
  "message": "done",
  "mode": 0,
  "mods": 0,
  "pp": 522.0230712890625,
  "stars": 7.084656715393066,
  "status": 1
}
```

*Acc list*

```
/api/calc?md5=ccb1f31b5eeaf26d40f8c905293efc03&acc_list=1
```

```json
{
  "acc_list": {
    "95": 311.07989501953125,
    "98": 389.848388671875,
    "99": 452.67974853515625,
    "100": 522.0230712890625
  },
  "message": "done",
  "mode": 0,
  "mods": 0,
  "pp": 522.0230712890625,
  "raw": {
    "acc": 128.93072509765625,
    "aim": 255.26805114746094,
    "spd": 127.87066650390625,
    "str": 0,
    "total": 522.0230712890625
  },
  "stars": 7.084656715393066,
  "status": 1
}
```

**with bid (Can use without add osu!api keys)**

```
/api/calc?bid=2848898
```

```json
{
  "acc_list": null,
  "message": "done",
  "mode": 0,
  "mods": 0,
  "pp": 366.8739013671875,
  "raw": {
    "acc": 118.15778350830078,
    "aim": 122.00947570800781,
    "spd": 121.79961395263672,
    "str": 0,
    "total": 366.8739013671875
  },
  "stars": 5.856814384460449,
  "status": 1
}
```

**with sid + file name (need osu!api keys)**

```
/api/calc?sid=1378720&file_name=Tanchiky%20-%20Bridge%20(NyarkoO)%20[Extension].osu
```

This method is currently cannot use cache.

```json
{
  "acc_list": null,
  "message": "done",
  "mode": 0,
  "mods": 0,
  "pp": 366.8739013671875,
  "raw": {
    "acc": 118.15778350830078,
    "aim": 122.00947570800781,
    "spd": 121.79961395263672,
    "str": 0,
    "total": 366.8739013671875
  },
  "stars": 5.856814384460449,
  "status": 1
}
```

### Best performance (Fastest, but lower accuracy)

Set Cargo.toml

```rust
peace-performance = { ... }
```

to

```rust
peace-performance = { ..., feature = "no_sliders_no_leniency" }
```

## Note

**This pp-server requires all `.osu` files use file MD5 as the name.**

- **Rename .osu files to file md5:**

  - If you want **pp server** auto recalculate all `.osu` files MD5 name before started, set `recalculate_osu_file_md5 = true` in `config/pp-server/default.toml`
  - Or manual run this python script `rename_osu_files.py` in project (python3.8+).
  - If its Debug compile, python will more faster than Rust.

- **Effect**
  - Calculating
  - ![p](screenshot/ef1.png)
  - After
  - ![p](screenshot/ef2.png)

### Setup

1. Set your `.osu` files dir path in `config/pp-server/default.toml`
2. Will let the `.osu` files name be the `md5` of the file
3. Set your osu!api keys in *.toml (if enabled feature `peace`, set it on your database)

### Debug

```
cargo run
```

### Release

```
cargo run --release
```

**Cross compile (Win to Linux)**

```
cargo cross_linux_x86
```

## MIT
