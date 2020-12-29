![https://github.com/Sequal32/yourcontrol](/assets/logo.png)
[![](https://img.shields.io/static/v1?label=enjoying%20the%20mod?%20&style=for-the-badge&message=DONATE&logo=paypal&labelColor=orange&color=darkorange)](https://www.paypal.com/paypalme/ctam1207)
[![](https://img.shields.io/github/v/tag/Sequal32/yourcontrol?label=release&style=for-the-badge)](https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip) [![](https://img.shields.io/github/downloads/Sequal32/yourcontrol/total?style=for-the-badge)](https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip) [![](https://img.shields.io/discord/764805300229636107?color=blue&label=discord&logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/p7Bzcv3Yjd)

Shared Cockpit for MSFS
- [Downloading](#downloading)
- [Running](#running)
- [Support Me!](#support-me)
- [Troubleshooting](#troubleshooting)
  - [Discord](#discord)
  - [Missing DLL](#missing-dll)
  - [Connection Timed Out](#connection-timed-out)
  - [Client can see host manipulate switches but not vice-versa](#client-can-see-host-manipulate-switches-but-not-vice-versa)
- [Limitations](#limitations)

The following aircraft are currently supported:
* FBW A32NX Stable + Dev
* Cessna 152
* Cessna 172 G1000
* DR400
* Diamond DA40
* Diamond DA62
* Icon A5
* Pitts
* TBM 930
* XCub

Other airliners have 10x the amount of code than the GA aircraft, so they will take a while to be fully implemented.

## Downloading
Download and run the [installer](https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip).
  * If the installer does not open for you, you'll need to install [Webview2](https://go.microsoft.com/fwlink/p/?LinkId=2124703).

## Running
1. Ensure everybody has the same navdata, scenery, and weather installed.
2. Launch MSFS, select the same aircraft and spawn location. **Do NOT enable multiplayer unless you're on different servers.**
3. Once everyone has spawned in, start up the included .exe file. **Do NOT run as administrator.**
4. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying (both server/clients should do this).
5. When a person connects, make sure to click the `Observer` button next to their name if you want them to be able to manipulate switches.
6.
    **Hoster (designate one person to run)**:

    Try all of these options in this order, until one works for you. `Cloud Server` does not have a 100% success rate. If you fail to connect using this method, try having another person host, but ultimately you'll have to fall back on the other methods.

    **Cloud Server**

    1. Click `Start Server`
    2. Give the provided session code to the joiners.

    **UPnP**
     1. Click `Start Server`
     2. Enter any port, or leave the default 7777 if unused.
     3. Give your `External IP`, and the port to the joiners.
      *If an error occurs, your router may be incompatible with UPnP, or does not have it enabled. You'll need to login into your router to enable it. More information below.*

    **Direct**
    1. If you have a [IPv6 address](https://test-ipv6.com/), you can simply give that along with the port to the joiners.
    2. **UDP** [port forward](https://www.youtube.com/watch?v=usSpl0yJFnY) either `7777` or the specified port in the application. If port forwarding is not an option, look into using [Hamachi](https://www.youtube.com/watch?v=bWbo0gcFqA8).
    3. Click start server.
      
1. **Joiners**:
   If given a Session Code, click `Cloud Server`, paste code, and click `Connect`

   If given an IP, confirm with the hoster whether it is IPv4 or IPv6, enter port, and click `Connect` 

2. Fly!
3. To transfer control, navigate to the `Connections` tab, find your partner's name and click `Give Control`.

Notes:
1. Both you and your copilot are recommended to turn off crash physics as there can be some desync issues that stresses your aircraft too much.
   
2. For the G1000/FMC/similar systems, only one person should be interacting with a given area at a time. For example, one person flies while the other fills out the flightplan (you should not be filing out the flightplan at the same time), or one person adjusts the transponder while another zooms out the map. This is to avoid desynchronization issues.

## Support Me!
If you enjoy the mod, considering showing your gratitude with a donation! I've put around a hundred hours of my own time into making this program in order for everyone to have an opportunity to fly together in as many aircraft as possible.

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://paypal.me/ctam1207)

## Troubleshooting
### Discord
<a href="https://discord.gg/SxYqf2n"><img src="https://discord.com/assets/e4923594e694a21542a489471ecffa50.svg" width="200"/></a>

If you're seeking help for this mod, or would like to beta test more aircraft/features, join the discord by clicking on the image above!

### Missing DLL
Install [Microsoft Visual C++ Redistributable](https://aka.ms/vs/16/release/vc_redist.x64.exe) to resolve this.

### Connection Timed Out
A connection to the server could not be established. Follow the steps for port forwarding and verifying your IP and forwarded port as described above.

### Client can see host manipulate switches but not vice-versa
The host needs to click on the `Observer` button under the client's name in the connection list.

## Limitations
* Some knobs are purely animation, and not represented by a local variable therefor cannot be synced, such as the TBM830's oxygen (yet...)
* Avionics currently rely on syncing button presses only and not state. If the MCDUs are different in any way before connecting, they will desyncronize.