# YourControls Changelog

## Version 2.6.2

### Profiles
* Added Headwind A330-900neo.
  - EFB not synced.
* Added RotorSimPilot Robinson R44 Raven II.
  - All pilots need to have flight model loaded with AirlandFS tool if transferring controls.

### Changes
* Fixed time sync on initial connect to host.
* Fixed H135 throttle collective.

## Version 2.6.1

### Profiles
* Added PMDG DC-6A/B (beta).
  - Note there are a few control knobs and multi-position switches that will physically move but won't sync the corresponding event(s) to client aircraft. We are working on how to send events that require multiple values.
  - For best sync possible, start Cold and Dark.
  - Ground power unit (GPU) sometimes despawns when connecting to host. To resync, simply toggle off and then on again with EFB (tablet).
  - Flaps sometimes fully extend for client when connecting to an already powered aircraft despite lever in same position.
  - Gyropilot is very sensitive and will fail for client aircraft in aggressive turns or weather. Recommend avoiding steep turns when using gyropilot turn knob and level out before resetting gyropilot switches.
  - Gyropilot for client pretends to follow GPS route but will drift if control handed over.
  - Gyropilot with altitude control ON will sometimes start porposing after control transfer but eventually stabilizes.
  - Windshield deice, once activated, client will always remain ON despite physical knob switched OFF. This is currently a workaround and will try to fix at later date.
  - Fuel is currently unsynced, but payload and passengers is. Fill up tanks accordingly before start.
  - Maintenance manager (EFB) is unsynced. Repair and service aircraft before start.
  - Beacon and Nav lights do not turn on despite switches ON. Hit 'L' on keyboard. (Asobo issue)
  - Since all switches are synced, AFE should only be used by one person at a time, or at least until his actions are finished.
  - Cargo/Exit doors and stairs use special events and will get out of sync if a change is made while co-pilot not connected.
* Added Carenado PA-34T Seneca V.
  - For best sync possible, start Cold and Dark.
  - Autopilot buttons use B-event toggle so 60% of the time, they work every time.
  - Propeller deice switch does not physically move for client but event still syncs.
* Added A32NX Experimental.
* Renamed and updated HypePerformanceGroup H135 to v1.4.3.
  - Recommend using L2: Basic flight model.
  - Starter switches are buggy when ramp starting. May have to switch them up and down a few times for client to properly start.
  - Throttle collective does not sync properly. Engines will remain in high-idle for client. (They can throttle up themselves but person-in-control overrides position/altitude)
* Quick patch of Aerosoft CRJ for 550-1000 family update. Full profile redo is planned for later.
  - Simulator fuel menu and payload now synced, EFB tablet is not.
* Added Asobo F/A-18E Super Hornet.
  - Many things don't sync because of B-events.
  - Cold start results in flap system issue. (Asobo issue)
* Added Asobo PC-6 Porter.
  - Many things don't sync because of B-events.
  - ADF swap is borked. Fast clicking swap will help correct frequencies.
* Added Asobo VoloCity helicopter.
* Added S-1S changes to Asobo_Pitts.
* Renamed XCub to X_NXCub.

### Changes
* Added support for aircraft slewing.
* Added support for TACAN channels.
* Added sync of G-force.
* Added sync of delta heading rate.
* Fixed A32NX sync of control stick, rudder pedals, and toe brakes.
* Fixed A32NX spoiler arm not always working.
* Fixed A32NX control transfer death spiral.
* Fixed missing control transfer hotkey (launch bar) to A32NX.
* Fixed warp crashing into ocean after control transfer.
* Fixed glitching around 360 degrees North.
* Fixed wheel spinning and launching forward on ground caused by faulty physics corrector.
* Fixed excessive aileron/rudder/FD drift with autopilot ON caused by AP not knowing how fast you are moving around the planet.
* Fixed magneto sync when pairing with someone using a physical yoke.
* Fixed throttle levers not syncing for modded aircraft using local vars as throttle position. (WT CJ4 and DA-62X)
* Fixed blue propeller and red mixture levers.

## Version 2.6.0

### Profiles
* Updated FBW A32NX to 0.8.0-dev. Temporarily removed outdated stable till next update.
  - AP buttons/knobs resynced. Managed speed unsyncable without overshoot (no variable). Either use selected speed or person in control set managed speed knob and remain in control for flight.
  - Added brake temp sync.
  - Strobe light doesn't like turning off. (FBW issue)
  - Printer sometimes causes sim crash. (FBW issue)
* Updated Salty 747 to v0.4.0+dev.
* Updated WorkingTitle CJ4 to v0.12.8.
* Updated JPLogistics C152 to v1.0.0-beta9.
* Added Heavy-Division 787 (78XH) profile.
* Added Mrtommymxr C172 profile.

### Changes
* All 30 ASOBO aircraft definition profiles updated for Sim Update 5 + brief updates for Sim Update 6.
* Added payload weight to all 30 ASOBO aircraft. Payload menu in sim is broken (SimConnect issue), however, weight values set by server host are still transferred to all clients even though clients won't "see" the change in value.
* Added payload weight to A32NX, Salty 747, WorkingTitle CJ4, and JPL C152. (same thing above applies)
* Added new Lvars for Garmin avionics since SU5.
* Added water rudder and gear handle to aircraft with floats and skis.
* Added missing pitch hold reference for VNAV flight director.
* Corrected all COM, NAV, ADF frequency overshoots. Boeing 787 STBY STEP buttons not fixable, please type frequency manually.
* Corrected COM1/2 event names and added COM3. All 3 radios should now work mostly as expected. (current B-event limitations)
* Corrected ADF event names and added ADF2.
* Corrected physics units from Degrees to Radians.
* Corrected(?) trim death dive after control transfer with AP on.
* Corrected gyro jumping and spinning.
* Removed glitchy engine statistics sync. May add back at later date.
* Removed yoke sync, replaced with flight control surfaces for more accurate external visuals. Yoke sync only showed 60% surface deflection for clients.
* Changed throttle levers to vars. Constant interp sync is unnecessary.
* Moved all Lvars for Garmin 330, 430, 530, Aera, and Vigilus to their own module definitions.

## Version 2.5.18

* Updated VCockpit.js for Sim Update V
* Support latest switch changes to the A32NX Development version

## Version 2.5.17

* A32NX ADIRS, AP APPR, AP LOC, AP EXPED, AP 1/2, AP ATHR
* use_calculator on a var: event will now set the parameter as well
* a custom on_condition can be specified for toggleswitches that use local vars

## Version 2.5.16

* Synchronized A32NX Dev version parking brake, mcdu/dcdu screen brightnesses

## Version 2.5.15

* Fixed a crash that could occur when trying to write a REALLY BIG floating point number

## Version 2.5.14

* Fix A32NX Development seatbelt sign and annunciator lights switch
* Fix Longitude spoiler arm not syncing

## Version 2.5.13


* Fix A32NX Development batteries, spoilers, FCU, flaps, flight controls on ECAM not updating
* Fixed lights on the DV20


## Version 2.5.12

* Fixed issues with the FBW A32NX experimental throttles desynced, vertical speed mismatched, and rattling sounds
* Synced emergency light on the WT CJ4
* Synced doors on the MixMugz TBM930
* Attempted to fix WT CJ4 FD/Range desync
* Attempted to fix Airbus H135 starters desync

## Version 2.5.11

* Resynced A32NX spoilers
* Fix CJ4 autopilot altitude/heading selector issues
  
## Version 2.5.10

* Actually fixed A32NX flaps
* Added new WT CJ4 master light

## Version 2.5.9

* Added support for the Airbus H135
* Resynced flaps/spoilers in the A32NX Experimental

## Version 2.5.7

* Changed the window title to YourControls vX.X.X
* Fixed an issue where the heading indicator in the C152/172 would spin in circles
* Fixed an issue where the plane would jump around in altitude below 1000ft
* Added Aerosoft CRJ (DISCLAIMER: knobs do not sync well at all, it may be impossible to get the altitude selectors/radios synced)
* Added support for the Carenado M20R, JustFLight PA28, and mixMugz TBM930 (EFBs do not sync)
* Rewritten YourControls gauge to support unlimited local variables
* Support the following pulls/commits in the A32NX Dev/Experimental version:
  * [flybywiresim/a32nx##3794](https://github.com/flybywiresim/a32nx/pull/3794)
  * [flybywiresim/a32nx##3930](https://github.com/flybywiresim/a32nx/pull/3930)'
  * [flybywiresim/a32nx/autopilot](https://github.com/flybywiresim/a32nx/commit/8d09903343552b255be5f68a1ed4fff38af37568)

## Version 2.5.6

* Synced A32NX APU bleed
* Prepare A32NX for when [flybywiresim/a32nx##3782](https://github.com/flybywiresim/a32nx/pull/3782) is merged


## Version 2.5.5

* Support new dev version of A32NX (APU/Electrical button syncage)
* Fixed an issue where the aircraft would be floating or through the ground on initial connection
* C152 - attempted to fix heading gyro desync
* WT CJ4 - fixed various desync

## Version 2.5.4

* Fixed an error message when NAV is activated on the CJ4
* Fixed an issue where brakes would still be depressed for others after releasing them
* Fixed multiple autopilot definitions conflicts on the WT CJ4
* Fixed off by one logic for the course and altimeter on the G1000s
* Fixed an issue where avionic presses would be triggered multiple times for clients

* Added C208B seatbelt signs/other INOP switches as an optional feature

## Version 2.5.3

### New Features

* Added an error message when the community package is not loaded in the simulator

### Fixes

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

### Misc

* Completely refactored the JS side of things

### Synced
* A32NX Coffee
* A32NX EFB Textboxes (ensure you have the same A32NX version)
* Time (on initial connection)
* Fuel
* Speed when winds are mismatched
* Altitude when scenery is mismatched

## Version 2.5.1

### Fixes

* Fixed an issue where aircraft movement and throttle would not be synced (removed fuel syncage).
* Small optimization of net code

## Version 2.5.0

### New Features

* Network Stats! See how much network YourControls is using and if there is any packet loss detected.
* Cloud Host Did port forwarding and Cloud Server not work with you? Well this new option gives you the opportunity to get connected without any other setup! Currently the server resides in the USA so latency is the biggest issue if you choose to use this connection method.
Note: Currently, the number of connections to the server is capped at 100, and will be increased as we upgrade our server infrastructure. You should always use Cloud P2P and Direct first if you can.
* A32NX clicking the priority button on the joystick will now forcibly take control.
* The person connecting to the server no longer has to select a definition file! The server will send their copy over to the client hassle-free
* New clients will no longer start as an observer
* You can now set a keybinding to transfer controls to, and take controls from the 1st person on the connection list! Go into the controls menu, and bind a key combo to LAUNCH BAR SWITCH TOGGLE.
* Streamer Mode - Hides your IP after connecting.
* New connections as observer - New connections will not be able to manipulate switches.

### Changes

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
