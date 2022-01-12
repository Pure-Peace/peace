## Bancho benchmarks

There are many open source bancho server implementations on github, so I wanted to test the performance of these servers.

## Players

- [Peace](https://github.com/Pure-Peace/Peace) (async rust, actix-web, postgresql)
- [gulag](https://github.com/cmyui/gulag) (async python, cmyui, mysql)
- [pep.py](https://github.com/kafuu-osu/pep.py) (python & cython, tornado, mysql)
- [ruri](https://github.com/rumoi/ruri) (c++, mysql)
  
No contestants
- [shiro](https://github.com/Marc3842h/shiro) (c++, mysql) (i spent a lot of time trying, but in the end the compiled program does not run...)
- [sora](https://github.com/Chimu-moe/Sora) (c#, mysql) (it doesn't compile, it seems to be caused by something missing in the osu.game i downloaded)
- [shiori](https://github.com/osu-katakuna/shiori) (node.js, express, mysql) (i can't find the sql file used to create the database)

## How to test

I just tested it on my own windows 10 PC for convenience. I mean, these test results may not be rigorous. And the performance of the program on windows is different from that on linux (So i will use wrk for concurrency testing on linux  systems sometime, maybe..?)


Finally, i used [locust](https://www.locust.io/) for testing. locust is a load testing tool written in python, i chose it mainly because of the data visualization, very easy. 

However, since locust is written in python, it will have some performance limitations. This will make it difficult for locust to simulate very high concurrency.

##### The test will use locust to simulate 100 users and 1000 users sending some requests at the same time.

I divide **requests** into **two types**, one is **light**: *in-game chat (just sending packets, no database operations involved)* and **the other is heavy**: *login, which has a longer process and involves database reading and writing.*

A few words about the differences in platforms and load testing procedures: on linux, actix-web (rust's web program framework) can run up to 10000 rps (using wrk to test), while on windows, using locust testing results in only 1500 rps.



## Start testing

- All participants turned off log printing to get the highest performance.
- All participants did not use ssl and front servers (nginx): Accessed directly (modified some of the gulag code for this purpose).


## Heavy

### 1. Peace

#### Bancho Login - (locust with 100 users)

The memory usage of the peace was around 6 mb when it was first started, and reached 14 mb when tested with locust generating 100 users. cpu usage peaked at 14.6% (up to).
![peace_cpu_100c](http://miya.ink/bench/peace_login/cpu_100c.png)

Test results, in the case of 100 users have been concurrent requests, peace to complete a login request spent about 86 ms, rps stable above 1000.
![peace_100c](http://miya.ink/bench/peace_login/login_100c.png)

#### Bancho Login - (locust with 1000 users)

1000 users, average cpu usage is around 11%, memory usage also increases.
![peace_cpu_1000c](http://miya.ink/bench/peace_login/cpu_1000c.png)

The rps dropped by 100 to 963
![peace_1000c](http://miya.ink/bench/peace_login/login_1000c.png)

Overall, Peace had the best performance.

Every login of a user will be recorded in the database of peace, and when a user logs out peace will update the logout time of the user, as well as the length  of time online. This requires many database operations (i have written this part of the operation as a database trigger that runs in the database.).


### 2. gulag

#### Bancho Login - (locust with 100 users)

The memory usage of the gulag was around 30 mb when it was first started, and reached 42 mb when tested with locust generating 100 users. cpu usage peaked at 18.8%.
![gulag_cpu_100c](http://miya.ink/bench/gulag_login/cpu_100c.png)

Test results, in the case of 100 users have been concurrent requests, peace to complete a login request spent about 599 ms, rps stable above 150.

In fact, that's a good score for python.
![gulag_100c](http://miya.ink/bench/gulag_login/login_100c.png)

#### Bancho Login - (locust with 1000 users)

1000 users, average cpu usage is around 20%, memory usage also increases (20mb).
![gulag_cpu_1000c](http://miya.ink/bench/gulag_login/cpu_1000c.png)

rps is still holding above 150, good job gulag!
![gulag_1000c](http://miya.ink/bench/gulag_login/login_1000c.png)

gulag's performance in this test was second only to Peace, this performance difference is largely the result of the programming language.

Before testing, I modified the code of gulag and cmyui to turn off log printing. And made gulag log out the previous user before each login is processed (consistent with peace).

Overall, gulag is doing quite well.


### 3. pep.py

This pep.py is derived from osu!thailand and has some changes (such as adding login records so that each login will insert a piece of data in mysql, same as Peace) that have reduced performance.

#### Bancho Login - (locust with 100 users)

pep.py has a very high memory and cpu usage (and mysql also has a high cpu usage)
![pep_cpu_100c](http://miya.ink/bench/pep_login/cpu_100c.png)

rps is only 16, processing time of over 5.9 seconds per request.

It can be seen that pep.py cannot withstand concurrency and has poor performance (even though it uses cython for acceleration, but many io is still not asynchronous (gulag is asynchronous io)).

![pep_100c](http://miya.ink/bench/pep_login/login_100c.png)

#### Bancho Login - (locust with 1000 users)

locust generates 1000 users for concurrent requests, which is a concurrency that pep.py can't handle at all. 

#### Most of the requests fail.
![pep_100c](http://miya.ink/bench/pep_login/login_1000c.png)


### 4. ruri

In fact, since ruri is implemented using c++, I was very optimistic about its performance at the beginning.

But the test results proved that ruri could not withstand the concurrency.

#### Bancho Login - (locust with *10* users)

The memory usage of ruri is very, very low (3.4 mb), even lower than the rust implementation.

![ruri_cpu_10c](http://miya.ink/bench/ruri_login/ruri_cpu.png)

But ruri can't handle concurrency, even if locust only generates 10 users.

All requests have failed.
![ruri_10c](http://miya.ink/bench/ruri_login/login_10c.png)



### After the heavy request test, the next is the light request test, which is the bancho message (involving only io)

- **Testing process:** First, we use python to login once to get osu-token, and then use locust to generate users (all using the same osu-token) to keep sending public message packets for testing.

- All servers turned off automatic speech bans.


**Public message packets:** 

```
\x01\x00\x00\x14\x00\x00\x00\x0b\x00\x0b\x06\xe4\xbd\xa0\xe5\xa5\xbd\x0b\x04#osu\x00\x00\x00\x00
```


### 1. Peace

#### Bancho Public Message - (locust with 100 users)


![peace_cpu_100c](http://miya.ink/bench/peace_message/cpu_100c.png)

Stress-free, rust completes these asynchronous iOs very easily.

rps is almost at its maximum (1500rps). (this maximum rps is the result of a test using locust to request an api that does nothing.)

![peace_100c](http://miya.ink/bench/peace_message/message_100c.png)

#### Bancho Public Message - (locust with 1000 users)

Memory usage has improved.

![peace_cpu_1000c](http://miya.ink/bench/peace_message/cpu_1000c.png)

![peace_1000c](http://miya.ink/bench/peace_message/message_1000c.png)


### 2. gulag

#### Bancho Public Message - (locust with 100 users)


![gulag_cpu_100c](http://miya.ink/bench/gulag_message/cpu_100c.png)

gulag's asynchronous io makes it easy to complete this test with good performance.

![gulag_100c](http://miya.ink/bench/gulag_message/message_100c.png)

#### Bancho Public Message - (locust with 1000 users)

Cpu usage has improved.

![gulag_cpu_1000c](http://miya.ink/bench/gulag_message/cpu_1000c.png)

Concurrency of 1000 users is not a problem.

![gulag_1000c](http://miya.ink/bench/gulag_message/message_1000c.png)

### 3. pep.py

#### Bancho Public Message - (locust with 100 users)

Memory usage is 10 mb higher than gulag, cpu usage is also higher.

![pep_cpu_100c](http://miya.ink/bench/pep_message/cpu.png)

pep.py's rps can basically stabilize above 700, not bad (much better than the results of the login test)

![pep_100c](http://miya.ink/bench/pep_message/message_100c.png)

#### Bancho Public Message - (locust with 1000 users)


pep.py is unable to support 1000 users concurrently and has a lot of errors.

![pep_1000c](http://miya.ink/bench/pep_message/message_1000c.png)


### 4.ruri

ruri cannot complete the test because concurrency is not supported.


---


### END

Test results are for reference only, if you have a better means of testing, welcome to issue.

2020-12-21 PurePeace
