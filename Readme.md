![https://github.com/Sequal32/yourcontrol](/assets/logo.png)
[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)
[![](https://img.shields.io/github/v/tag/Sequal32/yourcontrol?label=release&style=for-the-badge)](https://github.com/Sequal32/yourcontrol/releases/latest) [![](https://img.shields.io/github/downloads/Sequal32/yourcontrol/total?style=for-the-badge)](https://github.com/Sequal32/yourcontrol/releases/latest)

Shared Cockpit for Microsoft Flight Simulator 2020.

## Features
* Frequent and smooth position updates through linear interpolation
* Transferable controls
* Configurable aircraft files to sync most switches/knobs

Note: Currently only the C172 G1000 is implemented for this plugin. The entire GA fleet of the standard edition will be coming shortly. The airliners have 10x the amount code of the GA aircraft, so they will take a while to fully implemented.

## Install
1. Grab the latest [release](https://github.com/Sequal32/yourcontrol/releases/latest) and unzip to a directory of your choice.
2. Move the `YourControl` folder inside of `PLACE IN COMMUNITY PACKAGES` to...
   * Steam: C:\Users\[YOUR USERNAME]\AppData\Roaming\Microsoft Flight Simulator\Packages\Community
   * Microsoft Store: C:\Users\[YOUR USERNAME]\AppData\Local\Packages\Microsoft.FlightSimulator_<RANDOMLETTERS>\LocalCache\Packages\Community
   * Boxed: C:\Users\[YOUR USERNAME]\AppData\Local\MSFSPackages\Community

## Running
1. Launch FS2020, select the same aircraft, weather, and spawn location. Do NOT enable multiplayer.
2. Once everyone has spawned in, start up the included .exe file.
3. Navigate to the `Aircraft` tab and select the .yaml file associated with the aircraft you're flying (both server/clients should do this).
4.
    Enter a username, then...

    * Server (Designate 1 person to run):
      1. If your router does not support UPNP, [port forward](https://www.youtube.com/watch?v=usSpl0yJFnY) either `7777` or the specified port in the application. If you don't know if your router supports UPNP, you can attempt to connect without any additional steps, and then port forward if needed. If port forwarding is not an option, look into using [Hamachi](https://www.youtube.com/watch?v=bWbo0gcFqA8).
      2. Navigate to the server tab, **enter port and click start server**. You will have initial control of the aircraft.
      3. **Verify that the port was successfully forwarded**, and find your IP through this [website](https://www.yougetsignal.com/tools/open-ports/)
      
    * Clients: Navigate to the client tab and enter the **server's ip and port** and click connect.

1. Fly!
2. To transfer control, navigate to the `Connections` tab, find your partner's name and click `Give Control`.

## Troubleshooting
### Missing DLL
Install [Microsoft Visual C++ 2015 Redistributable Update 3 RC](https://www.microsoft.com/en-us/download/details.aspx?id=52685) to resolve this.
### Connection Timed Out
A connection to the server could not be established. Follow the steps for port forwarding and verifying your IP and forwarded port as described above.

### Discord
If you're seeking help for using this program, or would like to beta test more aircraft/features, join the [discord!](https://discord.gg/ywb7paY).

## Configuring
Config.json
```
{
    "update_rate": 30, // The update rate that aircraft data is sent in hz. Setting this any higher may be unstable.
    "conn_timeout": 10, // How many seconds to try to establish a connection with the server before timing out.
    "buffer_size": 3, // How many packets to buffer. Useful for unstable connections but increases latency.

    "port": 7777, // The default port shown in the app for client & server
    "ip": "", // The last entered ip
    "name": "", // The last entered username
    "last_config":"", // The last used aircraft config file
}
```
## Remarks
* The code's architecture is loosely defined, so looking at the code may prove challenging.
* Documentation for creating aircraft definition files is planned.

## Limitations
* G1000/FMC are not currently possible to fully sync (yet...)
* Some knobs are purely animation, and not represented by a local variable therefor cannot be synced (yet...)

## Data Files
If you're looking to create your own aircraft config file, refer to the provided files as examples and the [definitions](https://github.com/Sequal32/yourcontrol/tree/master/definitions) page.
