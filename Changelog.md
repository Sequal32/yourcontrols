# YourControls Changelog

## Version 2.7.3

### Profiles

- Added Savoia-Marchetti S.55 & S.55X by Microsoft (v1.5.3).
- Added Tecnam P92 Echo by Erasam (v1.5).
- Corrected aircraft-dependent profiles for latest name changes. (e.g. Heavy Division 787)

### Aircraft Fixes

- Asobo Longitude: Added more payload to support Dakfly mod.
- FlyByWire A32NX: Reverted some changes that broke AP knobs in last update.

### Issues
![image](https://cdn.discordapp.com/attachments/765289974232252466/1028444695069737030/fbwkeepoff.png)

## Version 2.7.2

### Profiles

- Added BN-2 Islander by BlackBox Simulation (v2.1.2).
- Added BN-3 Trislander by BlackBox Simulation (v1.1.6).
- Added Cessna C195 Businessliner by Microsoft (v1.4.2).
- Added Cessna C510 Mustang by Cockspur (v1.0.3).
- Updated FlyByWire A32NX (Dev/Exp) to October 2nd, 2022.

### Aircraft Fixes

- FlyByWire A32NX: Improved sync of ground services on tablet.

## Version 2.7.1

### Profiles

- Complete profile re-organization.
- Added Airbus A319neo by LatinVFR (v1.0.4).
- Added Airbus A321neo by LatinVFR (v1.2.5).
- Added Beechcraft Bonanza V35 by Microsoft (v1.5.2).
- Added Beechcraft Model 17 Staggerwing by Microsoft (v1.8.0).
- Added Beechcraft Model 18 by Microsoft (v1.9.0).
- Added Cessna CT182T Skylane by Carenado (v1.5.0).
- Added Cessna C337 Skymaster by Carenado (v1.1.2).
- Added Discus2c by GotFriends (v2.0.4).
- Added Edgley EA-7 Optica by GotFriends (v1.0.3).
- Added Hondajet by FlightFX (v1.0.6).
  - WT G3000 currently not supported, setting route won't sync.
- Added Junkers F 13 by Microsoft (v0.1.1).
- Added Junkers Ju 52 by Microsoft (v0.1.8).
- Added Pilatus PC-12 by Carenado (v1.0.3).
- Added Piper PA-28R Arrow III by Carenado (v1.5.0).
- Added Piper PA-44 Seminole by Carenado (v1.8.0).
- Added Schleicher AS33Me by MADolo Simulations (1.51).
- Updated Flysimware C414AW Chancellor to v3.2.0.
- Updated SimWorks Studios RV-14 to v1.3.2.

### Aircraft Fixes

- FlyByWire A32NX: Removed possible duplicate pushback command causing no more sync.
- Carenado M20R: Desync with vertical speed gauge.
- Carenado Seneca V: Landing pulse lights not turning off.

## Version 2.7.0

### Program Fixes

- Performance degradation with community module on solo flights longer than 2 hours.
- H-event duplication after ending and making new server with someone previously connected.
- An issue preventing FlyByWire A32NX compatiblity with Cloud Host.
- An issue with definitions not being received by clients through Cloud Host.
- Attempted to fix an issue which prevented those behind CGN (Carrier-grade NAT) to connect through Cloud Host.
- Systems on LAN can now connect through Cloud P2P.
- Added message for when cloud hoster connection is lost. First client in list becomes new hoster.
- Added hide clickability for IP addresses and session code.
- Added IPv6 support for Cloud P2P and Cloud Host.
- Hosting via the `Direct` method will now listen on both IPv6 and IPv4 addresses.
- Current session ID moved to the `Server` tab.

### Profiles

- Added A32NX Stable by FlyByWire (v0.8.1).
- Added Bell 47G by FlyInside (v1.71).
- Added Boeing 247D by Wing42 (v1.0.1).
- Added C140 by Aeroplane Heaven (v1.4.0a).
- Added C170B by Carenado (v1.3.0).
- Added C208B EX Improvement mod by Magraina (v2203.2.2).
- Added C310R by Milviz (1.0.0).
- Added C414AW Chancellor by Flysimware (v2.7.0).
- Added Concorde by DC Designs (v1.0.4).
- Added DHC6 Twin Otter by Aerosoft (v1.0.6.0).
- Added Electra-10A by Aeroplane Heaven (v1.33.7).
- Added FreedomFox & Fox2 (KitFox STi) by Parallel 42 (v1.0.0).
- Added G115 Tutor T.1 by IRIS Simulations (v2.3.7).
- Added H125 by RotorSimPilot (v1.3.9).
- Added J160/J170 Jabiru by IRIS Simulations (v1.5.6).
- Added JRF-6B Goose by Big Radials (v1.0.4).
- Added Kodiak 100 by SimWorks Studios (v1.2.2).
- Added PA-28 Warrior II by JustFlight (v0.3.5).
- Added PA-28R Turbo Arrow III/IV by JustFlight (v0.5.5).
- Added PC-6 Turbo Porter by Milviz (v1.0.9).
- Added RV-14 & 14A by SimWorks Studios (v1.2.0).
- Added RV-7 & 7A mod by Deejing (v1.0.8).
- Added Stream by LightSim (v1.3).
- Added TF-104G Starfighter by SimSkunkWorks (v3.1.2).
- Added basic support for GNS530 by PMS50 (v1.0.50).
- Added basic support for GTN750 by PMS50 (v2.1.41).
- Added basic support for GTNXi 750 by TDS Sim Software (v1.0.1.8).
- Added basic support for G1000 NXi by Working Title (v1.0.1).
- Updated ASOBO aircraft for Sim Update 9 (v1.25.7.0).
- Updated Carenado M20R to v1.5.1.
- Updated Carenado Seneca V to v1.4.0.
- Updated FlyByWire A32NX to May 18, 2022.
  - EFB not updated.
- Updated Headwind A330-900 to v0.200.
- Updated HypePerformanceGroup H135 to v1.4.5.
- Updated JustFlight Arrow III to v0.10.5.
- Updated Mrtommymxr C172 to v0.3.
- Updated Mrtommymxr DA62X to v0.7.
- Updated Mrtommymxr DA40NGX to v0.8.6.
- Updated RotorSimPilot R44 to v1.2.8.
- Updated SaltySimulations 747 to v0.5.1.
- Updated Working Title CJ4 to v0.12.13.
- Removed Aerosoft CRJ (being re-evaluated).
- Removed Frett G36 (deprecated).
- Removed PMDG DC-6 (being re-created).

### General Fixes

- All autopilot and radio button/knob desync on aircraft utilizing legacy simvars.
- Avionics master switches on all aircraft experiencing intermittent issues.
- Master caution and warning acknowledge events since Sim Update 7.
- Autopilot automatically leveling off when using V/S and FLC.
- Inaccurate G1000 COM/NAV volume level percentage.
- NAV/ADF volume knobs jumping back and forth.
- ADF frequency not always swapping.
- Parking brake event name spelling error.
- Attitude indicator bar calibration.
- Annunciator test light switch.
- Engine bleed air toggle event.
- External power toggle event.
- Propeller condition lever.
- Added pushback tug support.

### Aircraft Fixes

- Asobo 747: ATC ground services.
- Asobo 787: ATC ground services.
- Asobo A320neo: ATC ground services.
- Asobo A320neo: WX brightness knob.
- Asobo C172: Kohlsman index for G1000 PFD.
- Asobo C208: External power BUS and STBY alternator.
- Asobo Cap10: Missing external lights.
- Asobo Cap10: Aerobatic trim flap switch.
- Asobo CJ4: ATC ground services.
- Asobo CJ4: Pulse light pushbutton.
- Asobo KingAir: Added 3rd interior cabin light.
- Asobo Longitude: ATC ground services.
- Asobo Longitude: SVT terrain toggle desync.
- Asobo Porter: Profile not executing.
- Asobo Porter: Support for non-G950 variant.
- Asobo Porter: Master battery and avionics switches.
- Asobo Porter: Landing lights.
- Asobo SW121: Missing panel potentiometer.
- Asobo TBM930: SVT terrain toggle desync.
- A32NX: AP1 to AP2 toggle with a temporary workaround.
- A32NX: Autobrake level setting (#7067).
- A32NX: A/THR turning off and back on.
- A32NX: Corrected 16K value for flap handle.
- A32NX: Gear lever for new hydraulic system (#6893).
- A32NX: Ground services including exit and cargo doors (#7229).
- A32NX: LS pushbutton desync.
- A32NX: Mach airspeed knob.
- Carenado M20R: Interior and exterior lights.
- Carenado Seneca V: Added TDS GTN support.
- Headwind A339: ATC ground services.
- HPG H135: Barometer desync with MFD knobs.
- HPG H135: Excessive data spam while in hover mode.
- HPG H135: Tablet autopilot button desync.
- RSP H125: Added TDS GTN support.
- RSP R44: Added TDS GTN support.
- Salty 747: ATC ground services.
- Salty 747: Flight director switches.
- Salty 747: Sync of LNAV/VNAV modes.
- Working Title CJ4: Vertical pitch reference indicator.

### New issues

- B-events/variables: This is a new system introduced by Asobo with Sim Update 5. All switches and levers now using "B:" no longer visually move but any associated "A:" or "L:" secondary vars will still sync, causing proper effect. This is because "B:" is not settable through SimConnect. Asobo currently shows no intent on fixing this.
- Touchscreen interaction on 3rd-party avionics packages currently does not sync. This is a 3rd-party WASM limitation. Full support for such packages may or may not be possible.
- JustFlight PA-28 Bundle: Avionics power trouble may occur during cold start if GPS type on tablet are not the same for all connected. Each person should cycle type before powering.
- Milviz C310R: Some actions on tablet only sync once everyone connected has tablet visible.
- FlyByWire current fuel state unsyncable.

## Version 2.6.3

### Changes

- Attempt to fix cloud connection issues by changing the domain name to an IP address.

## Version 2.6.2

### Profiles

- Added Headwind A330-900neo.
  - EFB not synced.
- Added RotorSimPilot Robinson R44 Raven II.
  - All pilots need to have flight model loaded with AirlandFS tool if transferring controls.

### Changes

- Fixed time sync on initial connect to host.
- Fixed H135 throttle collective.
- Attempted to fix random warping on control transfer.
- Attempted to fix low FPS on long flights even without using YourControls.
- More instantaneous control transfer.
- New option to disable verbose logging sent/received packets to reduce log file size.
- Fixed an issue where the hostname/IP would disappear upon a failed connection.

## Version 2.6.1

### Profiles

- Added PMDG DC-6A/B (beta).
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
- Added Carenado PA-34T Seneca V.
  - For best sync possible, start Cold and Dark.
  - Autopilot buttons use B-event toggle so 60% of the time, they work every time.
  - Propeller deice switch does not physically move for client but event still syncs.
- Added A32NX Experimental.
- Renamed and updated HypePerformanceGroup H135 to v1.4.3.
  - Recommend using L2: Basic flight model.
  - Starter switches are buggy when ramp starting. May have to switch them up and down a few times for client to properly start.
  - Throttle collective does not sync properly. Engines will remain in high-idle for client. (They can throttle up themselves but person-in-control overrides position/altitude)
- Quick patch of Aerosoft CRJ for 550-1000 family update. Full profile redo is planned for later.
  - Simulator fuel menu and payload now synced, EFB tablet is not.
- Added Asobo F/A-18E Super Hornet.
  - Many things don't sync because of B-events.
  - Cold start results in flap system issue. (Asobo issue)
- Added Asobo PC-6 Porter.
  - Many things don't sync because of B-events.
  - ADF swap is borked. Fast clicking swap will help correct frequencies.
- Added Asobo VoloCity helicopter.
- Added S-1S changes to Asobo_Pitts.
- Renamed XCub to X_NXCub.

### Changes

- Added support for aircraft slewing.
- Added support for TACAN channels.
- Added sync of G-force.
- Added sync of delta heading rate.
- Fixed A32NX sync of control stick, rudder pedals, and toe brakes.
- Fixed A32NX spoiler arm not always working.
- Fixed A32NX control transfer death spiral.
- Fixed missing control transfer hotkey (launch bar) to A32NX.
- Fixed warp crashing into ocean after control transfer.
- Fixed glitching around 360 degrees North.
- Fixed wheel spinning and launching forward on ground caused by faulty physics corrector.
- Fixed excessive aileron/rudder/FD drift with autopilot ON caused by AP not knowing how fast you are moving around the planet.
- Fixed magneto sync when pairing with someone using a physical yoke.
- Fixed throttle levers not syncing for modded aircraft using local vars as throttle position. (WT CJ4 and DA-62X)
- Fixed blue propeller and red mixture levers.

## Version 2.6.0

### Profiles

- Updated FBW A32NX to 0.8.0-dev. Temporarily removed outdated stable till next update.
  - AP buttons/knobs resynced. Managed speed unsyncable without overshoot (no variable). Either use selected speed or person in control set managed speed knob and remain in control for flight.
  - Added brake temp sync.
  - Strobe light doesn't like turning off. (FBW issue)
  - Printer sometimes causes sim crash. (FBW issue)
- Updated Salty 747 to v0.4.0+dev.
- Updated WorkingTitle CJ4 to v0.12.8.
- Updated JPLogistics C152 to v1.0.0-beta9.
- Added Heavy-Division 787 (78XH) profile.
- Added Mrtommymxr C172 profile.

### Changes

- All 30 ASOBO aircraft definition profiles updated for Sim Update 5 + brief updates for Sim Update 6.
- Added payload weight to all 30 ASOBO aircraft. Payload menu in sim is broken (SimConnect issue), however, weight values set by server host are still transferred to all clients even though clients won't "see" the change in value.
- Added payload weight to A32NX, Salty 747, WorkingTitle CJ4, and JPL C152. (same thing above applies)
- Added new Lvars for Garmin avionics since SU5.
- Added water rudder and gear handle to aircraft with floats and skis.
- Added missing pitch hold reference for VNAV flight director.
- Corrected all COM, NAV, ADF frequency overshoots. Boeing 787 STBY STEP buttons not fixable, please type frequency manually.
- Corrected COM1/2 event names and added COM3. All 3 radios should now work mostly as expected. (current B-event limitations)
- Corrected ADF event names and added ADF2.
- Corrected physics units from Degrees to Radians.
- Corrected(?) trim death dive after control transfer with AP on.
- Corrected gyro jumping and spinning.
- Removed glitchy engine statistics sync. May add back at later date.
- Removed yoke sync, replaced with flight control surfaces for more accurate external visuals. Yoke sync only showed 60% surface deflection for clients.
- Changed throttle levers to vars. Constant interp sync is unnecessary.
- Moved all Lvars for Garmin 330, 430, 530, Aera, and Vigilus to their own module definitions.

## Version 2.5.18

### Changes

- Updated VCockpit.js for Sim Update V
- Support latest switch changes to the A32NX Development version
