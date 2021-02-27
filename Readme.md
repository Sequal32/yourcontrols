![https://github.com/Sequal32/yourcontrol](/assets/logo.png)
[![](https://img.shields.io/static/v1?label=enjoying%20the%20mod?%20&style=for-the-badge&message=DONATE&logo=paypal&labelColor=orange&color=darkorange)](https://www.paypal.com/paypalme/ctam1207)
[![](https://img.shields.io/github/v/tag/Sequal32/yourcontrol?label=release&style=for-the-badge)](https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip) [![](https://img.shields.io/github/downloads/Sequal32/yourcontrolsinstaller/total?style=for-the-badge)](https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip) [![](https://img.shields.io/discord/764805300229636107?color=blue&label=discord&logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/p7Bzcv3Yjd)

Shared Cockpit for MSFS 
[**Changelog**](/Changelog.md)

Table of Contents
- [Supported Aircraft](#supported-aircraft)
- [Downloading](#downloading)
- [Running](#running)
- [Settings](#settings)
- [Support Me!](#support-me)
- [Troubleshooting](#troubleshooting)
  - [Discord](#discord)
  - [Missing DLL](#missing-dll)
  - [Most switches or avionics are not syncing](#most-switches-or-avionics-are-not-syncing)
  - [My copilot's aircraft is above me](#my-copilots-aircraft-is-above-me)
  - [My copilot's aircraft is in front or behind me](#my-copilots-aircraft-is-in-front-or-behind-me)
  - [Mismatching speeds/altitude](#mismatching-speedsaltitude)
- [Limitations](#limitations)

## Supported Aircraft
The following aircraft are supported:
* All 30 stock by ASOBO
* FlyByWire A32NX Stable, Dev, Experimental
* JPLogistics C152X
* Working Title CJ4
* MrTommymxr DA62X
* MrTommymxr DA40NGX 
* SaltySimulations 747

## Downloading
**Recommended:** Download and run the [installer](https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip).
  * If the installer does not open for you, you'll need to install [Webview2](https://go.microsoft.com/fwlink/p/?LinkId=2124703).

**Alternative:** A manual zip is available [here](https://github.com/sequal32/yourcontrols/releases/latest/download/YourControls.zip). Extract, and then drag the `YourControls` folder inside of the `community` folder into your Community packages. You can put the folder anywhere else and then launch `YourControls.exe`.

## Running
1. Ensure everybody has the same **navdata**, **scenery**, and **weather** installed.
2. Launch MSFS, select the same aircraft and spawn location. **Do NOT enable multiplayer unless you're on different servers.**
3. Once everyone has spawned in, start up the included .exe file. **Do NOT run as administrator.**
4. It is important that you do not touch anything (especially avionics!) until everybody is connected.
4.
    **Hoster (designate one person to run)**:

    Try all of these options in this order, until one works for you.

    **Cloud P2P**
    Cloud P2P utilizes a rendezvous server in order to connect two computers behind a router. Dependending on your router, this may fail and you'll have to use other connection methods. *This is the preferred method*.

    1. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying.
    2. Click `Start Server`
    3. Give the provided session code to the joiners.

    **Cloud Host**
    Cloud Host utilizes a hosted server that *relays* traffic between computers. Because of the high traffic this uses, the current connection limit is capped at 100.

    
    1. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying.
    2. Click `Start Server`
    3. Give the provided session code to the joiners.

    **Direct**
    1. If you have an [IPv6 address](https://test-ipv6.com/), you can simply give that along with the port to the joiners.
    2. **UDP** [port forward](https://www.youtube.com/watch?v=usSpl0yJFnY) either `25071` or the specified port in the application. If port forwarding is not an option, try enabling UPnP, or use Cloud P2P or Cloud Host.
    3. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying.
    4. Click `Start Server`
      
1. **Joiners**:
   If given a Session Code, click `Cloud Server`, paste code, and click `Connect`

   If given an IP, confirm with the hoster whether it is IPv4 or IPv6, enter port, and click `Connect` 

2. Fly!
3. To transfer control, navigate to the `Connections` tab, find your partner's name and click `Give Control`. You can also set a keybinding in the MSFS Controls menu to `LAUNCH BAR SWITCH TOGGLE` which will give and take control from the first person in the connection list.

Notes:
1. Both you and your copilot are recommended to **turn off crash physics** as there can be some desync issues that stresses your aircraft too much.
   
2. For the G1000/FMC/other avionics, only one person should be interacting with a given area at a time. For example, one person flies while the other fills out the flightplan (you should not be filing out the flightplan at the same time), or one person adjusts the transponder while another zooms out the map. This is to avoid desynchronization issues.
   
3. If you want to load a SimBrief Flightplan in the A32NX, you need to set the **same SimBrief username** in the AOC settings.
   
4. For aircraft and avionics that have setting saving functionality (A32NX, Working Title, etc...), the state of the aircraft may be different depending on those settings. You should verify that you and your copilot(s) have the same settings.

## Settings
- **Dark Mode**: Switches between light and dark theme for the application.
- **Streamer Mode**: Hides your IP after connecting.
- **Use UPnP**: Attempts to automatically port forward using UPnP. You can check if it was actually successful in the Log.txt file.
- **New connections start as observer**: New connections will be unable to manipulate switches.

## Support Me!
If you enjoy the mod, considering showing your gratitude with a donation! I've put a few hundred hours of my own time into making this program in order for everyone to have an opportunity to fly together in as many aircraft as possible. It'll also help me keep the servers up.

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://paypal.me/ctam1207)

## Troubleshooting
### Discord
<a href="https://discord.gg/SxYqf2n"><img src="https://discord.com/assets/e4923594e694a21542a489471ecffa50.svg" width="200"/></a>

If you're seeking help for this mod, or would like to give feedback and join the community, join the discord by clicking on the image above!

### Missing DLL
Install [Microsoft Visual C++ Redistributable](https://aka.ms/vs/16/release/vc_redist.x64.exe) to resolve this.

### Most switches or avionics are not syncing

Restart your simulator. If it still isn't working, manually delete the `YourControls` folder inside your community folder and reinstall YourControls.

### My copilot's aircraft is above me

This is commonly due to mismatching sceneries. Please ensure you have the same addon airports installed and that you're not at an airport that is handcrafted in one edition of the sim, but not in the other (Premium Deluxe vs Deluxe).

### My copilot's aircraft is in front or behind me

Disable multiplayer and make sure you *are not* in the same group. If you're using an online network such as VATSIM or IVAO, check out their documentation on how to enable observer mode for shared cockpit.

### Mismatching speeds/altitude

Unfortunately, weather cannot currently be synced between flight simulators. If the sea pressure, ISA, temperature, and other weather-related aspects are different, you will experience desync.

## Limitations
* Some knobs are purely animation, and not represented by a local variable therefor cannot be synced, such as guard switches (yet...)
* Avionics currently rely on syncing button presses only and not state. If the MCDUs are different in any way before connecting, they will desynchronize.
* Scenery, weather, and navdata cannot be synced. Please ensure you have the same settings prior to connecting.
* Ground services, and ATC cannot be synced.