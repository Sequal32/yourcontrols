# YourControls Changelog

## Version 2.8.0

### Changes

- Brake pedals are now shared. (THE CAR TINA!)
- Improved sync of autopilot modes on Garmin avionics.
  - Not perfect, but better.
- Increased base engine support to 4.
- Updated glider support for SU14.
- Updated rotorcraft support for SU14.

### Profiles

- Added: Bell 222B by Cowan Simulation (v1.0.2).
- Added: Boeing 787-9 Dreamliner by Horizon Simulations (v1.0.1).
- Added: Guimbal Cabri G2 by Asobo Studio (v0.1.20).
- Added: Juice Goose UTV by Parallel 42 (v1.1.0).
- Added: MD Helicopters 530F by Shrike Simulations (v1.0.1).
- Readded: Airbus H125 by RotorSimPilot & Airland (v1.3.9).
- Updated: Airbus A320neo by FlyByWire (v0.11.1).
- Updated: Airbus A330-900neo by Headwind Simulations (v0.5.8).
- Updated: Airbus H135 by HypePerformanceGroup (Build 447).
- Updated: Bell 407 by Nemeth Designs (v0.3.6).
- Updated: Cessna 310R by Milviz/Blackbird (v2.0.4).
- Updated: Cirrus SR22T G6 by Asobo/Working Title (v0.2.13).
- Updated: Douglas DC-3 by Aeroplane Heaven (v1.2.5).
- Updated: Hughes H4 Hercules by Bluemesh (v1.2.5).
- Updated: Latecoere 631 by Bluemesh (v1.0.8).

### Aircraft Fixes

- Asobo 747: Taxi light flashing.
- Asobo 747/787: TO/GA switch event.
- Asobo CJ4: TO/GA switch event.
- Asobo CJ4: Wing/Eng deice switch event.
- Asobo TBM930: Right seat PFD altimeter.
- Asobo Longitude: Auto throttle arm.
- Asobo Longitude: Deice switch inaccuracies.
- Asobo Longitude: Spoiler lever range.
- Asobo Longitude: TO/GA switch event.
- CowanSim: Collective twists are now shared.
- CowanSim: Gyrocompass slave switches.
- CowanSim: Hydraulic and deice switches.
- DCD Concorde: APU bleed and generator events.
- DCD Concorde: Disabled XPNDR number buttons.
- DCD Concorde: Hydraulic and engine bleed switches.
- Deejing RV-7: G5 power button.
- FBW A320: Enabled saved NAV frequencies.
- FBW A320: Landing lights RETRACT.
- FBW A320: Potential causes of bad FPS.
- FBW A320: Potential gear desync.
- FBW A320: Strobe no longer on when OFF.
- FlightFX Vision Jet: Auto throttle arm.
- FSS E170/E175: Yoke and pedals for v0.9.19.
- G1000/G3000: CDI/NAV mode skipping.
- G3000: Saved barometer pressure desync.
- G3X/G1000/G3000: Added mach airspeed sync.
- Got Friends Discus-2c: FES power switch.
- HPG H135: Engine starter consistency.
- HPG H135: Numerous tablet desyncs.
- Headwind A330: Cabin and cargo doors.
- Headwind A330: Enabled saved NAV frequencies.
- Headwind A330: Potential causes of bad FPS.
- Headwind A330: Potential gear desync.
- Headwind A330: Strobe no longer on when OFF.
- Milviz 310R: G5 power buttons.
- Nemeth B407: Logo light switch.
- PMDG 737: Added tablet Home events.
- PMDG 737: Potential causes of bad FPS.
- PMDG 737: Tablet brightness.

### New Issues

- G1000/G3000: Flight director disabled to prevent breaking autopilot.

## Version 2.7.13

### Changes

- Added: Separate FlyByWire A32NX profile for use with SimBridge.

## Version 2.7.12

### Profiles

- Updated: Tecnam P2006T by FlightSim Studio (v1.0.6).

### Aircraft Fixes

- Asobo 747/787: Baro minimums. Radio minimums unsyncable.
- Asobo 787: Floppy yoke with autopilot on.
- Asobo 787: Inverted elevator trim.

## Version 2.7.11

- Cloud IPv6 should work again.

### Profiles

- Added: Boeing 787-8 Dreamliner by Kurorin (v2.1.1).
- Updated: Airbus A320 Family by LatinVFR.
- Updated: Cessna 310R by Milviz/Blackbird (v1.9.6).
- Updated: Embraer E170/E175 by FlightSim Studio (v0.9.17).
- Updated: Piper Arrow III by Just Flight (v0.10.8).
- Updated: Piper Turbo Arrow III/IV by Just Flight (v0.5.8).
- Updated: Piper Warrior II by Just Flight (v0.3.8).

### Aircraft Fixes

- Asobo 787: Throttle levers.
- Asobo 787: APU generator switches.
- Asobo 747/787: Hydraulic switches.
- FBW A320: Removed outdated condition for ECAM buttons.

#### Issues for Just Flight

- Arrow/Turbo Arrow: Heading bug no longer changes with AUTOPILOT HEADING LOCK DIR, unsyncable.
- Warrior: Works normally with AUTOPILOT HEADING LOCK DIR, syncable.

## Version 2.7.10

### Profiles

- Added: Boeing 314 Clipper by PILOT'S (v1.6.2).
- Added: Boeing PT–17 Stearman by DC Designs (v1.0.6).
- Added: Dornier Do J Wal by Microsoft & Oliver Moser (v0.1.1).
- Added: Latecoere 631 by Microsoft & Bluemesh (v1.0.6).
- Updated: Asobo Boeing 747 and 787 to AAU2.

### Aircraft Fixes

- General: All doors and windows re-done, no more delayed sync.
- ORBX P-750: Missing fuel valve sync preventing cold start.

#### Issues for Asobo/Working Title

- Asobo 747: COM STANDBY FREQUENCY does not update after a complete frequency is entered.
- Asobo 787: Radio panels are completely broken.
  - When solo, frequencies visually do not change when switching frequencies with native ATC menu.
  - When multicrew, frequencies visually do not change when simvars are correctly syncing.
  - STBY STEP buttons do nothing. COM STANDBY FREQUENCY does not change when stepping.

## Version 2.7.9

### Profiles

- Added: Beechcraft Baron B58 by Black Square (v0.1.0).
- Added: Beechcraft Bonanza A36 by Black Square (v0.1.1).
- Added: Beechcraft King Air 350i by Black Square (v0.1.3).
- Added: Cessna 208B Grand Caravan by Black Square (v0.1.4).
- Added: Tecnam P2006T MkII by FlightSim Studio (v1.0.5).
- Added: Velocity XL by Black Square (v0.1.2).

## Version 2.7.8

### Program

- Added: Clients can now put themselves into observer mode.
- Added: Instructor mode. New connections are placed in observer mode automatically.
- Fixed: Developer console flooded with errors leading to slideshow FPS.
- Removed: Update rate selection. Application now sends data as fast as possible.

### Profiles

- Added: Cessna R182 II Skylane RG by Carenado (v1.0.2).
- Added: DG Aviation DG1001E by Asobo Studio (v0.1.39).
- Added: DG Aviation LS8-18 by Microsoft & FSS AG (v0.8.4).
- Added: Noorduyn Norseman by Big Radials (v1.0.3).
- Added: Piper PA-28-181 Archer II by Carenado (v1.0.1).
- Added: SeaRey Elite by FlightSim Studio (v1.0.3).
- Updated: Cessna 170B by Carenado (v2.0.0).
- Updated: Cessna 172SP by WBSim (v1.0.7).
- Updated: Cessna 337H Skymaster by Carenado (v2.0.1).
- Updated: Cessna T182T Skylane by Carenado (v2.0.2).
- Updated: Cirrus Vision Jet by FlightFX (v2.0.2).
- Updated: Honda HA-420 HondaJet FlightFX (v2.0.0).
- Updated: Mooney M20R Ovation by Carenado (v2.0.1).
- Updated: Pilatus PC-6 by Milviz/BlackBird (v1.1.8).
- Updated: Pilatus PC-12 by Carenado (v2.0.0).
- Updated: Piper PA-28R Arrow III by Carenado (v2.0.0).
- Updated: Piper PA-34T Seneca V by Carenado (v2.0.1).
- Updated: Piper PA-44 Seminole by Carenado (v2.0.0).
- Updated: Tecnam P92 by AeroSachs & Erasam (v1.5.0/v2.0.1).

### Aircraft Fixes

- General: Added GPS OBS toggle setting.
- General: Added GPS OBS course setting.
- Asobo A320: Disabled XPNDR number buttons.
- Carenado 337H: Cowl flap switches.
- Carenado PA-34T: Landing/Pulse light switch.
- Carenado PC-12: Most menu desync on EX500.
- FBW A320: Disabled broken sync of passengers.
- FBW A320: Disabled XPNDR number buttons.
- Milviz PC-6: All tablet desync and spam.

## Version 2.7.7

### Profiles

- Added: Airbus A320neo by LatinVFR (v1.0.2).
- Added: Bell 206L3 by Cowan Simulation (v1.0.2).
- Updated: Airbus A318ceo by LatinVFR (v2.0.0).
- Updated: Airbus A319ceo by LatinVFR (v2.0.0).
- Updated: Airbus A321neo by LatinVFR (v2.0.1).
- Updated: Airbus A320neo by FlyByWire (v0.10.0).
- Updated: Airbus A330-900neo by Headwind Simulations (v0.4.1).
- Updated: Airbus H125 by Cowan Simulation (v1.0.2).
- Updated: Bell 206B3 by Cowan Simulation (v1.0.3).
- Updated: Cessna 414AW Chancellor by Flysimware (v4.3.3).
- Updated: Discus-2c by Got Friends (v2.0.7).
- Updated: Edgley EA-7 by Got Friends (v1.0.7).
- Updated: Grumman G-21A by Microsoft & iniBuilds (v1.1.5).
- Updated: Grumman JRF-6B Goose by Big Radials (v1.0.8).
- Updated: Hughes H4 by Microsoft & Bluemesh (v1.1.8).
- Updated: MD Helicopters 500E by Cowan Simulation (v1.0.2).
- Updated: Piper PA-28 Warrior II by JustFlight (v0.5.6).
- Updated: Piper PA-28R Arrow III by JustFlight (v0.10.6).
- Updated: Piper PA-28R Turbo Arrow by JustFlight (v0.3.6).
- Removed: FlyByWire A32NX Development/Experimental.
- Removed: Pushback tug (conflicts with other add-ons).
- Removed: Support for PMS50 GNS530 (deprecated).

### Aircraft Fixes

- General: Desync when pressing Garmin 330 numbers.
- General: Lots of excess unsettable local var cleanup.
- General: TACAN support updated per latest SDK.
- Asobo/WT Longitude: Backup ADI barometer.
- Microsoft PC-6: Dimmer brightness knobs.
- Microsoft Volocity: Throttle/VS switch.
- Big Radial JRF-6B: Added missing fuel valves.
- Big Radial JRF-6B: Propeller starter animation.
- Cowan H125: Corrected wrong fuel tank variable.
- FSS E175: Updated slider ignores to stop EFB spam.
- PMDG 737: Auto brake selectior knob desync.
- PMDG 737: CPT and FO yoke AP disconnect switches.
- PMDG 737: Disabled CPT and FO minimums selector knobs.
- PMDG 737: Landing and logo light switches w/pulse option.
- PMDG 737: Speed of parking brake set/release.
- SWS RV-10: Garmin G5 power button.

## Version 2.7.6

### Changes

- Added: Boeing 737NG Series by PMDG Simulations (v3.0.65).
- Added: Support for stored radio frequencies.

#### 737 Issues

- Recommend setting unmodified COLDDARK state before sharing.
- Persistent fuel state unsyncable. Simvars being overridden.
- Vertical speed window does not go negative value.
- Altimeter sync is disabled. Sim event is being overridden?
- APU/ENG bus switches may need to be switched twice for effect.
- COM3 does nothing interally. Not using native simvars. Won't sync.
- ADF standby frequency not using native simvar, active does and will sync.
- FLT ALT/LAND ALT knobs currently disabled to prevent desync.

## Version 2.7.5

### Profiles

- Added: Airbus A318ceo by LatinVFR (v1.0.2).
- Added: Airbus H125 by Cowan Simulation (v1.0.0).
- Added: Bell 206B3 by Cowan Simulation (v1.0.2).
- Added: Cessna 172SP Classic by WBSim (v1.0.6).
- Added: Embraer E175 by FlightSim Studio (v0.9.6-experimental).
  - Do not use standby freq/swap. Enter directly into active.
- Added: MD Helicopters 500E by Cowan Simulation (v1.0.1).
- Added: Van's RV-10 by SimWorks Studios (v1.0.3).
- Updated: Airbus A320neo (Dev/Exp) by FlyByWire to March 24, 2023.
- Updated: AAU1-Brief updates for Longitude, TBM, and CJ4.
- Updated: Airbus A330-900neo by Headwind Simulations (v0.3.3).
- Updated: Cessna 152 by JPLogistics & WBSim (v2.0.6).
- Updated: Cirrus Vision Jet G2 by FlightFX (v1.2.3).
- Updated: Robinson R44 by RotorSimPilot & Airland (v1.4).
- Readded: Douglas DC-6 by PMDG Simulations (v2.0.47).
  - Fuel and service repair still not synced.
- Removed: Numerous profiles of discontinued enhancement mods.

### Aircraft Fixes

- FBW A320: AUX fuel tanks not filling through EFB.
- Asobo TBM930: Pilot and copilot reading lights.

## Version 2.7.4

### Profiles

- Added: Bell 407 by Microsoft & Nemeth Designs (v0.2.1).
- Added: Cirrus SF50 Vision Jet G2 by FlightFX (v1.2.0).
- Added: Curtiss JN4 Jenny by Microsoft & iniBuilds (v1.1.2).
- Added: Douglas DC-3 by Microsoft & Aeroplane Heaven (v1.0.4).
- Added: DHC-2 Beaver by Microsoft & Blackbird Sims (v1.0.2).
- Added: Grumman G-21A Goose by Microsoft & iniBuilds (v1.1.2).
- Added: Hughes H4 Hercules by Microsoft & Bluemesh (v1.1.3).
- Added: PAC P-750 XSTOL by ORBX (v1.0.1).
- Updated: Aeroplane Heaven C140 (v1.5.0).
- Updated: Aerosoft DHC-6 Twin Otter (v1.1.1).
- Updated: Carenado PC-12 (v1.1.0).
- Updated: DC Designs Concorde (v1.0.6).
- Updated: FlyByWire A32NX (Stable) (v0.9.1).
- Updated: FlyByWire A32NX (Dev/Exp) to December 24, 2022.
- Updated: Flysimware C414AW Chancellor (v3.2.3).
- Updated: Headwind A330-900neo (v0.3.1).
- Updated: LatinVFR A319ceo (v1.0.7).
- Updated: LatinVFR A321neo (v1.2.10).
- Updated: Microsoft Bonanza V35 (v1.6.0).
- Updated: Microsoft Junkers Ju 52 (v0.1.9).
- Updated: Milviz 310R (v1.1.9).
- Updated: Milviz PC-6 (v1.1.2).
- Updated: Salty Simulations 747-8i (Dev) to December 4, 2022.
- Updated: SimSkunkWorks TF-104G Starfighter (v3.3.1).
- Updated: SimWorks Studios Kodiak 100 II & III (v1.4.0).

### Changes

- General: Removed unsettable circuit failure variables.
- Carenado: Re-enabled tablet button sync of pilot models.
- Aerosoft DHC-6: Fixed cabin general, reading, emergency, and FO ceiling lights.
- Aerosoft DHC-6: Fixed cockpit caution, entrance, and skydiver day/night lights.
- Big Radials JRF-6B: Fixed manual fuel pump lever.
- FlyByWire A32NX: Attempted to fix glitching SPD selection.
- FlyByWire A32NX: Fixed desync of ISIS SPD/ALT bugs.
- Flysimware C414: Fixed tablet fuel tank spam.
- Milviz C310: Fixed tablet dirt level spam.
- Milviz C310: Fixed tablet fuel tank spam.
- Milviz PC-6: Fixed trapdoor open/close.
- Parallel 42 Kitfox: Fixed pilot & copilot doors.
- SimSkunkWorks TF-104G: Fixed missing fuel sync for center & tip tanks.
- Fixed typo with COM RECEIVE ALL.
- Added support for NAV3 and NAV4.
- Added support for rotorcraft.

## Version 2.7.3

### Profiles

- Added: Savoia-Marchetti S.55 & S.55X by Microsoft (v1.5.3).
- Added: Tecnam P92 Echo by Erasam (v1.5).
- Corrected aircraft-dependent profiles for latest name changes. (e.g. Heavy Division 787)

### Aircraft Fixes

- Asobo Longitude: Added more payload to support Dakfly mod.
- FlyByWire A32NX: Reverted some changes that broke AP knobs in last update.

### Issues

![image](https://cdn.discordapp.com/attachments/765289974232252466/1028444695069737030/fbwkeepoff.png)

## Version 2.7.2

### Profiles

- Added: BN-2 Islander by BlackBox Simulation (v2.1.2).
- Added: BN-3 Trislander by BlackBox Simulation (v1.1.6).
- Added: Cessna C195 Businessliner by Microsoft (v1.4.2).
- Added: Cessna C510 Mustang by Cockspur (v1.0.3).
- Updated: FlyByWire A32NX (Dev/Exp) to October 2nd, 2022.

### Aircraft Fixes

- FlyByWire A32NX: Improved sync of ground services on tablet.

## Version 2.7.1

### Profiles

- Complete profile re-organization.
- Added: Airbus A319ceo by LatinVFR (v1.0.4).
- Added: Airbus A321neo by LatinVFR (v1.2.5).
- Added: Beechcraft Bonanza V35 by Microsoft (v1.5.2).
- Added: Beechcraft Model 17 Staggerwing by Microsoft (v1.8.0).
- Added: Beechcraft Model 18 by Microsoft (v1.9.0).
- Added: Cessna CT182T Skylane by Carenado (v1.5.0).
- Added: Cessna C337 Skymaster by Carenado (v1.1.2).
- Added: Discus2c by GotFriends (v2.0.4).
- Added: Edgley EA-7 Optica by GotFriends (v1.0.3).
- Added: Hondajet by FlightFX (v1.0.6).
- Added: Junkers F 13 by Microsoft (v0.1.1).
- Added: Junkers Ju 52 by Microsoft (v0.1.8).
- Added: Pilatus PC-12 by Carenado (v1.0.3).
- Added: Piper PA-28R Arrow III by Carenado (v1.5.0).
- Added: Piper PA-44 Seminole by Carenado (v1.8.0).
- Added: Schleicher AS33Me by MADolo Simulations (1.51).
- Updated: Flysimware C414AW Chancellor (v3.2.0).
- Updated: SimWorks Studios RV-14 (v1.3.2).

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

- Added: A32NX Stable by FlyByWire (v0.8.1).
- Added: Bell 47G by FlyInside (v1.71).
- Added: Boeing 247D by Wing42 (v1.0.1).
- Added: C140 by Aeroplane Heaven (v1.4.0a).
- Added: C170B by Carenado (v1.3.0).
- Added: C208B EX Improvement mod by Magraina (v2203.2.2).
- Added: C310R by Milviz (1.0.0).
- Added: C414AW Chancellor by Flysimware (v2.7.0).
- Added: Concorde by DC Designs (v1.0.4).
- Added: DHC6 Twin Otter by Aerosoft (v1.0.6.0).
- Added: Electra-10A by Aeroplane Heaven (v1.33.7).
- Added: FreedomFox & Fox2 (KitFox STi) by Parallel 42 (v1.0.0).
- Added: G115 Tutor T.1 by IRIS Simulations (v2.3.7).
- Added: H125 by RotorSimPilot (v1.3.9).
- Added: J160/J170 Jabiru by IRIS Simulations (v1.5.6).
- Added: JRF-6B Goose by Big Radials (v1.0.4).
- Added: Kodiak 100 by SimWorks Studios (v1.2.2).
- Added: PA-28 Warrior II by JustFlight (v0.3.5).
- Added: PA-28R Turbo Arrow III/IV by JustFlight (v0.5.5).
- Added: PC-6 Turbo Porter by Milviz (v1.0.9).
- Added: RV-14 & 14A by SimWorks Studios (v1.2.0).
- Added: RV-7 & 7A mod by Deejing (v1.0.8).
- Added: Stream by LightSim (v1.3).
- Added: TF-104G Starfighter by SimSkunkWorks (v3.1.2).
- Added: Basic support for GNS530 by PMS50 (v1.0.50).
- Added: Basic support for GTN750 by PMS50 (v2.1.41).
- Added: Basic support for GTNXi 750 by TDS Sim Software (v1.0.1.8).
- Added: Basic support for G1000 NXi by Working Title (v1.0.1).
- Updated: ASOBO aircraft for Sim Update 9 (v1.25.7.0).
- Updated: Carenado M20R (v1.5.1).
- Updated: Carenado Seneca V (v1.4.0).
- Updated: FlyByWire A32NX to May 18, 2022.
- Updated: Headwind A339X (v0.200).
- Updated: HypePerformanceGroup H135 (v1.4.5).
- Updated: JustFlight Arrow III (v0.10.5).
- Updated: Mrtommymxr C172 (v0.3).
- Updated: Mrtommymxr DA62X (v0.7).
- Updated: Mrtommymxr DA40NGX (v0.8.6).
- Updated: RotorSimPilot R44 (v1.2.8).
- Updated: SaltySimulations 747 (v0.5.1).
- Updated: Working Title CJ4 (v0.12.13).
- Removed: Aerosoft CRJ (being re-evaluated).
- Removed: Frett G36 (deprecated).
- Removed: PMDG DC-6 (being re-created).

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
- Headwind A339X: ATC ground services.
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