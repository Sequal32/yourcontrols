Thanks for checking out YourControls!

To setup:
1. Ensure everybody has the same navdata, scenery, and weather installed.
2. Launch MSFS, select the same aircraft and spawn location. Do NOT enable multiplayer unless you're on different servers.
3. Once everyone has spawned in, start up the included .exe file. Do NOT run as administrator.
4.
    Hoster (designate one person to run):

    Try all of these options in this order, until one works for you.

    Cloud P2P
    Cloud P2P utilizes a rendezvous server in order to connect two computers behind a router. Dependending on your router, this may fail and you'll have to use other connection methods. *This is the preferred method*.

    1. Click `Start Server`
    2. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying.
    3. Give the provided session code to the joiners.

    Cloud Host
    Cloud Host utilizes a hosted server that *relays* traffic between computers. Because of the high traffic this uses, the current connection limit is capped at 100.

    1. Click `Start Server`
    2. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying.
    3. Give the provided session code to the joiners.

    Direct
    1. If you have a IPv6 address, you can simply give that along with the port to the joiners.
    2. UDP port forward either `25071` or the specified port in the application.
    3. In `Settings`, under the header `Active Aircraft`, select the .yaml file associated with the aircraft you're flying.
    4. Click `Start Server`
      
1. Joiners:
   If given a Session Code, click `Cloud Server`, paste code, and click `Connect`

   If given an IP, confirm with the hoster whether it is IPv4 or IPv6, enter port, and click `Connect` 

2. Fly!
3. To transfer control, navigate to the `Connections` tab, find your partner's name and click `Give Control`.

Notes:
1. Both you and your copilot are recommended to turn off crash physics as there can be some desync issues that stresses your aircraft too much.
   
2. For the G1000/FMC/similar systems, only one person should be interacting with a given area at a time. For example, one person flies while the other fills out the flightplan (you should not be filing out the flightplan at the same time), or one person adjusts the transponder while another zooms out the map. This is to avoid desynchronization issues.

If you enjoy the mod, considering showing your gratitude with a donation! 
I've put around a hundred hours of my own time into making this program in order for everyone to have an opportunity to fly together in as many aircraft as possible.
https://paypal.me/ctam1207

If you're missing vcruntime140.dll or any other DLLs, install Microsoft Visual C++ 2015 Redistributable Update 3 RC.
https://www.microsoft.com/en-us/download/details.aspx?id=52685

For more information, check out the Official documentation.
https://docs.yourcontrols.one

For early testing of new features and support, join the discord.
https://discord.gg/SxYqf2n