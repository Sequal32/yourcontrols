![](/assets/logo.png)
[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com) [![forthebadge](https://forthebadge.com/images/badges/fo-real.svg)](https://forthebadge.com)

A simple shared cockpit solution for MSFS2020.

## Setup
1. Grab the latest [release](https://github.com/Sequal32/yourcontrol/releases/latest) and unzip to a directory of your choice.
1. Launch FS2020 and make sure you and your partner(s) spawn in close to each other in the **same aircraft state** (all spawn on runway or all spawn on ramp)
1. Start up the included .exe file.
    * One human should enter `s`, and **port forward** either `7777` or the port in `config.json` (which gets created on first run) and expect to output `Server Started`
    * Others should enter the 1st human's ip, and expect to output `Client Connected`
2. Fly!
3. To transfer control, assign a key to `Toggle Water Rudder` then activate the key binding to either
   * **Relieve control** if you're currently flying
   * **Accept control** once the human flying relieves controls
## Remarks
* Not all switchable switches are currently syncronized due to the current state of SimConnect
* The code is currently not organized in any way, so there's a bunch of spagettiness to it