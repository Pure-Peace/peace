# Peace (WIP)

## osu! server written in Rust.
[![Discord](https://discordapp.com/api/guilds/817149875635879997/widget.png?style=shield)](https://discord.gg/sgQwkNXpVe) 
#### Fully asynchronous, high concurrency, high performance, and high security.



[![Rust](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

#### If you want to see the performance comparison of different bancho servers：
- [Bancho benchmarks vs](https://github.com/Pure-Peace/Peace/blob/main/bancho_benchmark.md)

### XXX:
- **Rust 1.48** (stable)
- **PostgreSQL 11+**, with special database design...(maybe..)
- **Redis**
- **Prometheus** + **Grafana**: server monitoring and visualization (performance)
- **Sentry**: server monitoring (errors)
- **WebSocket** support (used to push user results, etc.?)
- **Web templates** support... (html will be compiled into the binary file)
- **Geo-ip** Local geo-ip service (or api), fast and available for your other applications!

### Schedules WIP...
- ...


### Database design (WIP...)
![db](http://miya.ink/db_013.png)


### Why not python or javascript?

- Because I want to try out a high-performance, efficient compiled language. 
- If you want a bancho written in python, please use [gulag](https://github.com/cmyui/gulag), it's good.

### Why not C++?

- Because Rust is the best choice for near C++ performance, but more modern.
- [Benchmark: Rust **vs** C, C++, Go](https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/rust.html)
- [Benchmark: Web Framework](https://www.techempower.com/benchmarks/#section=data-r19&hw=ph&test=composite)

### Reasons to choose Rust:

1. **High performance and efficiency**

    No garbage collection, Rust's performance is very close to C++, and its memory footprint is low.

2. **Modernization**

    Rust is a modern compiled language with an elegant syntax.

3. **Package Management**

    Cargo, just like npm or pip, is convenient and has many packages available.
 
4. **Security**

    Memory Safety: No worry about memory leaks and segment errors; 
    
    Thread-safe: no data contention.

5. **Asynchronous Support**

    It is very easy and elegant to write asynchronous code in Rust, supporting very high concurrency.

6. **Cross-compilation**

    Rust is very easy to cross-compile. After a simple configuration, you can compile Rust programs for linux platforms on Windows.

7. **Documentation**

    Clear and complete language documentation

### Reference Links:
- https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/rust.html
  
- https://medium.com/paritytech/why-rust-846fd3320d3f

- https://logdna.com/blog/coding-for-performance-why-we-chose-rust/

- https://kornel.ski/rust-c-speed

- https://www.techempower.com/benchmarks/#section=data-r19&hw=ph&test=composite

- https://www.rust-lang.org/

### But you can still star :p

Recommended development tools: **Visual Studio Code** or **CLion**

![vscode](http://miya.ink/55.png)
![clion](http://miya.ink/44.png)
Soo good

![Chino](http://miya.ink/22.png)

**Vscode plugins:**

```
Rust
rust-analyzer
Rust Syntax
crates
Cargo
Even Better TOML
Better Comments
Code Runner

GitLens — Git supercharged
Git History
Git Graph
```

### Avatar Server?
- Use **nginx** (good performance; simple configuration)
- *This is different from many bancho implementations, why?*
- The effect is the same, you can access `1000.jpg` (or png, gif, jpg) via `a.ppy.sh/1000`
- **nginx is the most suitable software as a static resource server**

Use nginx to separate the avatar resources from the bancho server (separating static and dynamic resources) for optimal processing performance.


### Dev on windows

![dev](http://miya.ink/dev.png)

Add hosts:
```hosts
127.0.0.1 osu.ppy.sh
127.0.0.1 c.ppy.sh
127.0.0.1 c1.ppy.sh
127.0.0.1 c2.ppy.sh
127.0.0.1 c3.ppy.sh
127.0.0.1 c4.ppy.sh
127.0.0.1 c5.ppy.sh
127.0.0.1 c6.ppy.sh
127.0.0.1 ce.ppy.sh
127.0.0.1 a.ppy.sh
127.0.0.1 i.ppy.sh
```
Start **Nginx**: http://nginx.org/download/nginx-1.18.0.zip

- With [nginx configuration file and ssl certificate](https://github.com/Pure-Peace/Peace/blob/main/nginx/readme.md).
- Need to install certificate to "Trusted Certification Authority" first.

Install **PostgreSQL** and initialize **Peace** database:
```
cd sql
```
```
./init_database.bat
```


## Geo-ip

After installing this module, turn it on in the **.toml** configuration file to get the user's geolocation information. Also peace provides geo-ip api to allow your other applications to use this service as well.

[>> Go to Readme and setup!](https://github.com/Pure-Peace/Peace/blob/main/geoip/readme.md)

### Example:

Access:
http://127.0.0.1:8080/geoip/219.76.152.150

Request 1ms, Result:

```json
{
    "ip_address": "219.76.152.150",
    "latitude": 22.3833,
    "longitude": 114.2,
    "continent_code": "AS",
    "continent_name": "Asia",
    "country_code": "HK",
    "country_name": "Hong Kong",
    "region_code": "NST",
    "region_name": "Sha Tin",
    "city_name": "Sha Tin Wai",
    "timezone": "Asia/Hong_Kong",

    "message": null,
    "status_code": 1
}
```




---

Finally, Run **Peace**:


## Run

**Debug**

- The compilation speed is faster, but the binary file size is larger. And the performance is much lower than release compilation and cannot be used for performance testing.
```
cargo run
```

**Release**

- Longer compilation time, but best performance, suitable for deployment.
- You can edit **Cargo.toml** to enable **lto**, which will increase compilation time, but with better performance and smaller files
```
cargo run --release
```

```
cargo build --release
```

The compiled binary can be moved to any directory containing **config** and run, this means you can compile on the PC, and then send binary files to the server for deployment.

**Run with environment**

- Run **Peace** with the specified configuration file:
- The configuration file must be located in the config directory (`environment`.toml)
- Default: development
```
cargo run <environment(.toml)>
```
**Examples:**
```
cargo run prodction
```
```
cargo run development
```
```
./Peace prodction
```

## Nginx and SSL certificate configuration

- Read [nginx/readme.md](https://github.com/Pure-Peace/Peace/blob/main/nginx/readme.md)

## MIT
