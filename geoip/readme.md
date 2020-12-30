# GeoIp

To get geolocation information (country region, latitude and longitude, etc.).

**Peace integrates this capability, and you can choose to enable the geo-ip http service locally for use by other applications.**

Example:
```
curl http://127.0.0.1:8080/geoip/219.76.152.150
```

Request ~ 1ms
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


Although it is undoubtedly the most convenient to use api services provided by others, considering the network speed and efficiency, I would like to set up geo-ip database services **locally** so that the speed of querying the geolocation information of an ip can be improved a lot.

maxmind provides **free** locally deployable geo-ip database **GeoLite2**, (mmdb and csv formats), And geo-ip api service.

Support for ipv4 and ipv6 info query.

maxmind website: [https://www.maxmind.com/en/home](https://www.maxmind.com/en/home)


## Next is a tutorial on how to deploy this service:


- To download the locally deployed GeoLite2 database, you first need to register an account with maxmind: [https://www.maxmind.com/en/geolite2/signup](https://www.maxmind.com/en/geolite2/signup)

## Register

It's very simple, take care to fill in the correct email address. 

After submitting your registration, maxmind will send an email to your email address, you need to click on the link in the email to complete the registration:

**Click on the link and set your password, registration is complete.**

After that **login to your account**, you can go to download the database!
The username is the registered email address, the password is the password you set

## Database Setup

**Download address (login required):** [https://www.maxmind.com/en/accounts/470006/geoip/downloads](https://www.maxmind.com/en/accounts/470006/geoip/downloads)


maxmind offers six free databases in mmdb and csv formats.

**Download this:**
- GeoLite2-City.mmdb

**We need**: GeoLite2-City.mmdb has precise geolocation info, including latitude and longitude, but **larger** (>60mb)
**Not this**: GeoLite2-Country.mmdb is accurate to the **country**, but **smaller** (<4mb)


### Unzip the downloaded zip, then configure the file path of **.mmdb** in the config/***.toml** file in config and you're ready to go!


```toml
# Geoip service config
# If enabled, we can get the player's geolocation info
# You need to download the .mmdb file from maxmind and specify the path here
[geoip]
enabled = true
mmdb_path = "geoip/GeoLite2-City.mmdb"
# If enabled, peace will load a route for querying geo-ip information for other applications to call.
web_api = true
```

Run Peace, now you can visit:
http://127.0.0.1:8080/geoip/219.76.152.150
to see the ip geolocation information!

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

## About mmdb

mmdb is one of maxmind's own binary database formats, which provides faster ip query speed.

To read/manipulate mmdb, maxmind provides APIs and libraries for most languages: [https://dev.maxmind.com/geoip/geoip2/downloadable/](https://dev.maxmind.com/geoip/geoip2/downloadable/)


## Update/upgrade database

Since ip information is often changing, it is necessary to update the database. maxmind very kindly provides a database upgrade tool, which can be used to easily upgrade the database to the latest: [https://dev.maxmind.com/geoip/geoipupdate/](https://dev. maxmind.com/geoip/geoipupdate/)

- Note that the free database update cycle is once every two weeks, and you need to pay to get the fastest update speed.


---


Well, with this, it's a breeze to build a local database of ip geolocation information~