# PEACE

*osu! server development framework 🚀*

<p align="center">
  <img src="/docs/media/peacev1.gif">
</p>

[![Rust](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

## [>> Documentation <<](https://peace.osu.icu/)



## *Description*

**Make it easy for developers to build their desired osu! server or project**

This framework provides a series of capabilities, including but not limited to: *osu! bancho, osu! lazer, api v2, pp calculation, bancho packet read & write...*

# Welcome to Peace

## Not "re-implementation"

Unlike other "re-implementation" projects for Bancho,
this project is starting completely from scratch and will focus on scalability and composability.
Moreover, implementing Bancho is just a small part of the process, as the ultimate goal is to achieve Lazer.

## Microservice architecture

The biggest feature of "Peace" is its use of a microservice architecture.
It consists of a group of services, which can be either remote or local.
You can split any service and start each service independently,
or you can combine all the services together to form a whole and start them separately.
Alternatively, you can split any service as needed and run them on different servers.

## Logic and State separation

Due to its distributed architecture,
Peace can implement logic and state separation,
allowing you to update the logic without restarting the service
and without losing user sessions and all states.
This means that you do not have to stop and restart the server every time you make an update.

## Language is not limited

Moreover, the language is not limited, which gives developers great freedom.
For example, you can use Python to write a flexible gateway service
and then integrate it into the gRPC service written in Rust to handle Bancho logic.
You can also write a Node.js service to replace the default Rust implementation.
You just need to implement the RPC interfaces defined in the `.proto` file,
and the switch can be seamlessly done.

## Multiple databases supports

Peace supports multiple databases,
including MySQL, PostgreSQL, and SQLite,
giving you the freedom to choose your preferred database.
With the support of SQLite, you can easily develop, debug, and run Peace locally.

## xxx

This project is still in the development stage,
and the architectural design is still in progress.
Many of the logic has not been implemented,
and the code structure may undergo major changes at any time.
It is not guaranteed that the code you download will be usable.

## *Current status*

*WIP*

## *Community*

[![Discord](https://discordapp.com/api/guilds/817149875635879997/widget.png?style=banner3)](https://discord.gg/6YKQMPpMrz)
