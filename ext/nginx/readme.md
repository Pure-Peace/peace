## Peace Nginx

### Generate local dev cert

Requires openssl installation

```
cd cert
```

- windows

```
./gencert.bat
```

Then Double click `cert.cer`, install into `Trusted Organizations`

- linux
```
./gencert.sh
```

#### For windows dev

### 1. Download nginx windows:
http://nginx.org/download/nginx-1.18.0.zip


### 2. Extract nginx files here and set nginx into environments:
- You can also manually add nginx to the environment variables
- The `.bat` needs to be run with administrator privileges
```
./set_nginx_into_path.bat
```

### 3. Configure the nginx conf file (including avatar server)

- Create avatar dir including `default.jpg` (or `png`, `jpg`, `gif`)
- Change `avatar_dir` in `bancho.conf`
- Move `nginx.conf` and `bancho.conf` into `nginx/conf` dir

### 4. Start nginx
```
./start_nginx.bat
```

### linux

include `bancho.conf` in your own `nginx.conf`

And do `3. Configure the nginx conf file (including avatar server)`

Finally, run nginx

