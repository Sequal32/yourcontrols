![https://github.com/Sequal32/yourcontrol](/assets/logo.png)
[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)
[![](https://img.shields.io/github/v/tag/Sequal32/yourcontrol?label=release&style=for-the-badge)](https://github.com/Sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip) [![](https://img.shields.io/github/downloads/Sequal32/yourcontrol/total?style=for-the-badge)](https://github.com/Sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip)

Shared Cockpit for Microsoft Flight Simulator 2020.

# Table of Contents
- [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Installing](#installing)
  - [Running](#running)
  - [Support Me!](#support-me)
  - [Troubleshooting](#troubleshooting)
    - [Discord](#discord)
    - [Missing DLL](#missing-dll)
    - [Connection Timed Out](#connection-timed-out)
  - [Remarks](#remarks)
  - [Limitations](#limitations)
  - [Data Files](#data-files)

## Features
* Frequent and smooth position updates through linear interpolation
* Transferable controls
* Configurable aircraft files to sync most switches/knobs
* Synchronzied button presses on FMCs/GXXXXs

The following aircraft have config files:
* FBW A32NX v0.4.X
* Cessna 152
* Cessna 172 G1000
* Diamond DA40
* Diamond DA62
* Pitts
* TBM 930
* XCub

The airliners have 10x the amount of code than the GA aircraft, so they will take a while to be fully implemented.

## Installing
1. Download the installer from [here](https://github.com/Sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip), unzip, and run the exe file.
   1. Don't click the Download Code button in the home page of the repository.
   2. Don't unzip/install into Program Files or any other system protected area, neither the installer or the program can run under elevated privilages.

## Running
1. Launch FS2020, select the same aircraft, weather, and spawn location. **Do NOT enable multiplayer unless you're on different servers.**
2. Once everyone has spawned in, start up the included .exe file. **Do NOT run as administrator.**
3. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying (both server/clients should do this).
4.
    Enter a username, then...

    * Server (Designate 1 person to run):
      1. If your router does not support UPNP, **TCP** [port forward](https://www.youtube.com/watch?v=usSpl0yJFnY) either `7777` or the specified port in the application. If you don't know if your router supports UPNP, you can attempt to connect without any additional steps, and then port forward if needed. If port forwarding is not an option, look into using [Hamachi](https://www.youtube.com/watch?v=bWbo0gcFqA8).
      2. **Verify that the port was successfully forwarded**, and find your IP through this [website](https://bytekitchen.de/tools/YCCT/)
         *This program additionally supports IPv6, so if your ISP supports it, port forwarding is not usually required.*
      3. Under the server tab, **enter port and click start server**. You will have initial control of the aircraft.
      
    * Clients: Enter the **server's ip and port** under the client section and click connect.

1. Fly!
2. To transfer control, navigate to the `Connections` tab, find your partner's name and click `Give Control`.

Notes:
1. Both you and your copilot are recommended to turn off crash physics as there can be some desync issues that stresses your aircraft too much.
   
2. For the G1000/FMC/similar systems, only one person should be interacting with a given area at a time. For example, one person flies while the other fills out the flightplan (you should not be filing out the flightplan at the same time), or one person adjusts the transponder while another zooms out the map. This is to avoid desynchronization issues.
   
3. Because this mod modifies some of the default aircraft files, other mods that build their mods on top of these aircraft files may be incompatitble. The easiest solution is to disable YourControls in the content manager, or remove YourControls from the community packages.

## Support Me!
If you enjoy the mod, considering showing your gratitude with a donation! I've put around a hundred hours of my own time into making this program in order for everyone to have an opportunity to fly together in as many aircraft as possible.

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://paypal.me/ctam1207)

## Troubleshooting
### Discord
If you're seeking help for using this program, or would like to beta test more aircraft/features, join the [discord!](https://discord.gg/SxYqf2n).

### Missing DLL
Install [Microsoft Visual C++ 2015 Redistributable Update 3 RC](https://www.microsoft.com/en-us/download/details.aspx?id=52685) to resolve this.

### Connection Timed Out
A connection to the server could not be established. Follow the steps for port forwarding and verifying your IP and forwarded port as described above.

## Remarks
* The code's architecture is loosely defined, so looking at the code may prove challenging.
* Documentation/tutorials for creating aircraft definition files is planned.

## Limitations
* Some knobs are purely animation, and not represented by a local variable therefor cannot be synced, such as the TBM830's oxygen (yet...)

## Data Files
If you're looking to create your own aircraft config file, refer to the provided files as examples and the [definitions](https://github.com/Sequal32/yourcontrol/tree/master/definitions) page.