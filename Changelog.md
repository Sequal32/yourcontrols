# YourControls Changelog

## Version 2.6.4

#### Profiles
* Added SimSkunkWorks TF-104G Starfighter.

#### Changes
* Added missing support for non-G950 Asobo PC-6 Porter.
* Fixed Porter profile error.

## Version 2.6.3

#### Changes
* Attempt to fix cloud connection issues by changing the domain name to an IP address.

## Version 2.6.2

#### Profiles
* Added Headwind A330-900neo.
  - EFB not synced.
* Added RotorSimPilot Robinson R44 Raven II.
  - All pilots need to have flight model loaded with AirlandFS tool if transferring controls.

#### Changes
* Fixed time sync on initial connect to host.
* Fixed H135 throttle collective.
* Attempted to fix random warping on control transfer.
* Attempted to fix low FPS on long flights even without using YourControls.
* More instantaneous control transfer.
* New option to disable verbose logging sent/received packets to reduce log file size.
* Fixed an issue where the hostname/IP would disappear upon a failed connection.

## Version 2.6.1

#### Profiles
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

#### Changes
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

#### Profiles
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

#### Changes
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

#### Changes
* Updated VCockpit.js for Sim Update V
* Support latest switch changes to the A32NX Development version