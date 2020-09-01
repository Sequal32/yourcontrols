![](/assets/logo.png)
[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)

A simple shared cockpit solution for MSFS2020.

## Setup
1. Grab the latest [release](https://github.com/Sequal32/yourcontrol/releases/latest) and unzip to a directory of your choice.
1. Launch FS2020 and make sure you and your partner(s) spawn in close to each other in the **same aircraft state** (all spawn on runway or all spawn on ramp)
1. Start up the included .exe file.
    * Server: **Port forward** either `7777` or the specified port in the application. Navigate to the server tab, enter port and click start server. You will have initial control of the aircraft.
    * Clients: Navigate to the client tab and enter the **server's ip and port** and click connect.
1. Fly!
1. To transfer control, assign a key to `Toggle Water Rudder` then activate the key binding or click the button in the application to either
   * **Relieve control** if you're currently flying
   * **Accept control** when the person flying relieves control
## Configuring
Config.json
```json
{
  "port": 7777, // The default port shown in the app for client & server
  "update_rate": 15, // The update rate in hz
  "conn_timeout": 10.0 // When control should be taken back after a packet hasn't been received for X amount of seconds
}
```

<details>
    <summary>Screenshots</summary>
    <img src="assets/app.png">
</details>

## Remarks
* Not all switchable switches are currently syncronized due to the current state of SimConnect
* The `.dat` files can be freely modified, but they must match a specification otherwise the application will not work correctly.
* The code's architecture is loosely defined, so looking at the code may prove challenging.