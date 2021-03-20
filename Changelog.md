# Version 2.5.5
* Support new dev version of A32NX (APU/Electrical button syncage)
* Fixed an issue where the aircraft would be floating or through the ground on initial connection
* C152 - attempted to fix heading gyro desync
* WT CJ4 - fixed various desync

# Version 2.5.4
* Fixed an error message when NAV is activated on the CJ4
* Fixed an issue where brakes would still be depressed for others after releasing them
* Fixed multiple autopilot definitions conflicts on the WT CJ4
* Fixed off by one logic for the course and altimeter on the G1000s
* Fixed an issue where avionic presses would be triggered multiple times for clients

* Added C208B seatbelt signs/other INOP switches as an optional feature

# Version 2.5.3

## New Features
* Added an error message when the community package is not loaded in the simulator

## Fixes
* Fixed a script error in the frontend UI that would pop up when the external IP could not be fetched
* Fixed an issue where the number of clients connected would not update
* Fixed an issue where the UI would be delayed in loading the saved config

* Fixed an issue where the game would crash when transferring controls
* Fixed an issue where clients would get teleported to various locations when transferring controls
* Attempted to fix an issue where controls, switches, and avionics would stop working (gauge crashed)
* Attempted to fix an issue where some switches/key presses would get randomly dropped
  
* Fixed an issue where the selected altitude would reset when climbing and nearing the selected altitude
* Fixed pitch hold/autopilot leveler not syncing as intended
* Fixed the FLC, HDG, NAV, VNAV, VS on the WT CJ4 where it would not sync properly
* Fixed nav radios not syncing properly

## Misc
* Completely refactored the JS side of things

## Synced
* A32NX Coffee
* A32NX EFB Textboxes (ensure you have the same A32NX version)
* Time (on initial connection)
* Fuel
* Speed when winds are mismatched
* Altitude when scenery is mismatched

# Version 2.5.1
## Fixes
* Fixed an issue where aircraft movement and throttle would not be synced (removed fuel syncage).
* Small optimization of net code

# Version 2.5.0

## New Features
* Network Stats! See how much network YourControls is using and if there is any packet loss detected.
* Cloud Host Did port forwarding and Cloud Server not work with you? Well this new option gives you the opportunity to get connected without any other setup! Currently the server resides in the USA so latency is the biggest issue if you choose to use this connection method.
Note: Currently, the number of connections to the server is capped at 100, and will be increased as we upgrade our server infrastructure. You should always use Cloud P2P and Direct first if you can.
* A32NX clicking the priority button on the joystick will now forcibly take control.
* The person connecting to the server no longer has to select a definition file! The server will send their copy over to the client hassle-free
* New clients will no longer start as an observer
* You can now set a keybinding to transfer controls to, and take controls from the 1st person on the connection list! Go into the controls menu, and bind a key combo to LAUNCH BAR SWITCH TOGGLE.
* Streamer Mode - Hides your IP after connecting.
* New connections as observer - New connections will not be able to manipulate switches.

## Changes
* Reworked interpolation - syncs position/rotation data more reliably
* UPnP is now it's own setting. Check the `Log.txt` if you need to see if UPnP worked or not and why.
* The default port has now been changed to 25071.

## Bug Fixes
* Clicking the CDI button on the G1000/G3000 will no longer throw it off sync
* Fixed an issue where MCDU scratchpad inputs would be out of order. For example, typing KBOS would sync KOBS sometimes.
* FD drift has been fixed (only if winds are the same for all people)
* Fixed an issue where transferring controls would lead to an unrecoverable dive

## Synced
* Indicated Airspeed
* FBW A32NX EFB, Printer, APU, New Radios
* Engine N1/N2/ITT/torque
* Engine oil temp/oil pressure
* More precise radio syncing (you can now increment by 0.05MHz)
* Fuel

**Huge thanks to @rthom91 for testing and syncing all of these new aircraft!**

## New Aircraft
* Experimental FBW A32NX
* Mrtommymxr DA40NGX
* Mrtommymxr DA62X
* SaltySimulations 747-8
* TheFrett Bonanza G36
* WorkingTitle CJ4
* Asobo Extra 330LT
* Asobo Boeing 747-8i
* Asobo Boeing 787-10
* Asobo Airbus A320 Neo
* Asobo Mudry Cap 10
* Asobo Cessna 152
* Asobo Cessna 172 Steam
* Asobo Cessna 208B
* Asobo CJ4
* Asobo CTLS
* Asobo Diamond DA40NG
* Asobo Diamond DA40TDI
* Asobo Robin DR400
* Asobo DV20
* Asobo Bonanza G36
* Asobo Baron G58
* Asobo KingAir 350
* Asobo Cessna Citation Longitude
* Asobo Cirrus SR22
* Asobo Pipistrel Virus SW1221
* Asobo Zlin Savage Cub
* Asobo Zlin Savage Shock
* Asobo Aveko VL-3 Sprint

## Known Issues
* Having differing weather will cause differences in indicated altitude and airspeed