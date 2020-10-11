![https://github.com/Sequal32/yourcontrol](/assets/logo.png)
[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)
[![](https://img.shields.io/github/v/tag/Sequal32/yourcontrol?label=release&style=for-the-badge)](https://github.com/Sequal32/yourcontrol/releases/latest) [![](https://img.shields.io/github/downloads/Sequal32/yourcontrol/total?style=for-the-badge)](https://github.com/Sequal32/yourcontrol/releases/latest)

A simple shared cockpit solution for Microsoft Flight Simulator.

## Features
* Frequent and smooth position updates through linear interpolation
* Transferable controls and fully configurable switches to synchronize

Note: Currently only the C172 G1000 is implemented for this plugin. Documentation for implementing for aircraft is planned and will be provided in the [Data Files](#Data-Files) section.

## Install
1. Grab the latest [release](https://github.com/Sequal32/yourcontrol/releases/latest) and unzip to a directory of your choice.
2. Move the `YourControl` folder inside of `PLACE IN COMMUNITY PACKAGES` to...
   * Steam: C:\Users\[YOUR USERNAME]\AppData\Roaming\Microsoft Flight Simulator\Packages\Community
   * Microsoft Store: C:\Users\[YOUR USERNAME]\AppData\Local\Packages\Microsoft.FlightSimulator_<RANDOMLETTERS>\LocalCache\Packages\Community
   * Boxed: C:\Users\[YOUR USERNAME]\AppData\Local\MSFSPackages\Community

## Running
1. Launch FS2020 and make sure you and your partner(s) spawn in *close* to each other in the **same aircraft state** (all spawn on runway or all spawn on ramp)
2. Start up the included .exe file.
3. Navigate to the `Aircraft` tab and select the .yaml file associated with the aircraft you're flying.
4.
    Enter a username, then...
    * Server:
      1. If your router does not support UPNP, [port forward](https://www.noip.com/support/knowledgebase/general-port-forwarding-guide/) either `7777` or the specified port in the application. If you don't know if your router supports UPNP, you can attempt to connect without any additional steps, and then port forward if needed.
      2. Navigate to the server tab, enter port and click start server. You will have initial control of the aircraft.
    * Clients: Navigate to the client tab and enter the **server's ip and port** and click connect.
5. Fly!
6. To transfer control, navigate to the `Connections` tab, find your partner's name and click `Give Control`.

## Troubleshooting
### Missing DLL
Install [Microsoft Visual C++ 2015 Redistributable Update 3 RC](https://www.microsoft.com/en-us/download/details.aspx?id=52685) to resolve this.

### Discord
I've created a [discord](https://discord.gg/ywb7paY) for anybody seeking support for this program.
`ywb7paY`

## Configuring
Config.json
```
{
  "port": 7777, // The default port shown in the app for client & server
  "ip": "", // The last entered ip
  "last_config":"", // The last used aircraft config file
  "buffer_size": 3, // How many packets to buffer. Useful for unstable connections.
  "conn_timeout": 10, // How many seconds to try to establish a connection with the server before timing out.
  "update_rate": 30, // The update rate that aircraft data is sent in hz. Setting this any higher may be unstable.
}
```
## Remarks
* The code's architecture is loosely defined, so looking at the code may prove challenging.
* Documentation for creating aircraft definition files is planned.

## Data Files
If you're looking to create your own aircraft config file, refer to the provided files as examples and the [definitions](https://github.com/Sequal32/yourcontrol/tree/master/definitions) page.